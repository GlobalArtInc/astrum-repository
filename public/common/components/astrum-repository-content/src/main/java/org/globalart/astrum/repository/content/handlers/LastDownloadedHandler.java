
package org.globalart.astrum.repository.content.handlers;

import java.time.OffsetDateTime;
import javax.annotation.Nullable;

import org.globalart.astrum.common.collect.AttributesMap;
import org.globalart.astrum.repository.capability.GlobalRepositorySettings;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Response;
import org.globalart.astrum.repository.view.Status;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.context.annotation.Primary;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.repository.http.HttpMethods.GET;
import static org.globalart.astrum.repository.http.HttpMethods.HEAD;

/**
 * Updates the asset last downloaded time. Throttling is enforced by AssetStore via a
 * conditional SQL UPDATE; see ASTRUM-52006.
 */
@Primary
@Component
public class LastDownloadedHandler
    implements org.globalart.astrum.repository.view.handlers.LastDownloadedHandler
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final LastDownloadedAttributeHandler lastDownloadedAttributeHandler;

  private final GlobalRepositorySettings globalRepositorySettings;

  @Autowired
  public LastDownloadedHandler(
      final LastDownloadedAttributeHandler lastDownloadedAttributeHandler,
      final GlobalRepositorySettings globalRepositorySettings)
  {
    this.lastDownloadedAttributeHandler = checkNotNull(lastDownloadedAttributeHandler);
    this.globalRepositorySettings = checkNotNull(globalRepositorySettings);
  }

  @Override
  public Response handle(final Context context) throws Exception {
    Response response = context.proceed();

    try {
      if (isSuccessfulRequestWithContent(context, response)) {
        Content content = (Content) response.getPayload();
        maybeUpdateLastDownloaded(content.getAttributes());
      }
    }
    catch (Exception e) {
      log.error("Failed to update last downloaded time for request {}", context.getRequest().getPath(), e);
    }

    return response;
  }

  protected void maybeUpdateLastDownloaded(final AttributesMap attributes) {
    maybeUpdateLastDownloaded(attributes.get(Asset.class));
  }

  protected void maybeUpdateLastDownloaded(@Nullable final Asset asset) {
    if (asset instanceof FluentAsset fluentAsset) {
      if (isStale(fluentAsset) && fluentAsset.tryMarkAsDownloaded()) {
        lastDownloadedAttributeHandler.writeLastDownloadedAttribute(fluentAsset);
      }
    }
    else if (asset != null) {
      log.debug("Cannot mark read-only asset {} as downloaded", asset.path());
    }
  }

  private boolean isStale(final FluentAsset asset) {
    OffsetDateTime threshold = OffsetDateTime.now().minus(globalRepositorySettings.getLastDownloadedInterval());
    return asset.lastDownloaded().map(last -> last.isBefore(threshold)).orElse(true);
  }

  private boolean isSuccessfulRequestWithContent(final Context context, final Response response) {
    return isGetOrHeadRequest(context)
        && isSuccessfulOrNotModified(response.getStatus())
        && response.getPayload() instanceof Content;
  }

  private boolean isGetOrHeadRequest(final Context context) {
    String action = context.getRequest().getAction();
    return GET.equals(action) || HEAD.equals(action);
  }

  private boolean isSuccessfulOrNotModified(final Status status) {
    return status.isSuccessful() || status.getCode() == 304;
  }
}
