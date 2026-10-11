package org.globalart.astrum.repository.content.handlers;

import java.time.OffsetDateTime;
import java.util.Date;
import javax.annotation.Nullable;

import org.globalart.astrum.common.collect.AttributesMap;
import org.globalart.astrum.repository.ETagHeaderUtils;
import org.globalart.astrum.repository.capability.GlobalRepositorySettings;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.downloads.AssetDownloadCounter;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.http.HttpConditions;
import org.globalart.astrum.repository.http.HttpStatus;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Headers;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.Request;
import org.globalart.astrum.repository.view.Response;
import org.globalart.astrum.repository.view.Status;

import com.google.common.base.Strings;
import com.google.common.net.HttpHeaders;
import org.apache.http.client.utils.DateUtils;
import org.joda.time.DateTime;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.context.annotation.Primary;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.repository.date.DateTimeUtils.formatDateTime;
import static org.globalart.astrum.repository.http.HttpMethods.GET;
import static org.globalart.astrum.repository.http.HttpMethods.HEAD;

/**
 * Updates the asset last downloaded time and counts the download of the asset. Throttling of the last downloaded time
 * is enforced by AssetStore via a conditional SQL UPDATE; see ASTRUM-52006. Downloads are counted by
 * {@link AssetDownloadCounter} without throttling.
 *
 * A download is a GET request answered with the whole content, or with a range of it starting at the first byte.
 * HEAD requests, conditional requests answered with 304 and ranges starting further in do not count. This handler runs
 * inside the conditional request and partial fetch handlers of the recipes, so it sees the response before they turn it
 * into a 304 or 206 and checks the conditions and range of the request itself.
 */
@Primary
@Component
public class LastDownloadedHandler
    implements org.globalart.astrum.repository.view.handlers.LastDownloadedHandler
{
  private static final String BYTES_UNIT = "bytes";

  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final LastDownloadedAttributeHandler lastDownloadedAttributeHandler;

  private final GlobalRepositorySettings globalRepositorySettings;

  private final AssetDownloadCounter assetDownloadCounter;

  @Autowired
  public LastDownloadedHandler(
      final LastDownloadedAttributeHandler lastDownloadedAttributeHandler,
      final GlobalRepositorySettings globalRepositorySettings,
      final AssetDownloadCounter assetDownloadCounter)
  {
    this.lastDownloadedAttributeHandler = checkNotNull(lastDownloadedAttributeHandler);
    this.globalRepositorySettings = checkNotNull(globalRepositorySettings);
    this.assetDownloadCounter = checkNotNull(assetDownloadCounter);
  }

  @Override
  public Response handle(final Context context) throws Exception {
    Response response = context.proceed();

    try {
      if (isSuccessfulRequestWithContent(context, response)) {
        Content content = (Content) response.getPayload();
        maybeUpdateLastDownloaded(content.getAttributes());
        maybeCountDownload(context, response, content);
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

  private void maybeCountDownload(final Context context, final Response response, final Content content) {
    Asset asset = content.getAttributes().get(Asset.class);
    if (asset != null && isDownload(context, response, content)) {
      assetDownloadCounter.recordDownload(context.getRepository().getFormat().getValue(), asset);
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

  private boolean isDownload(final Context context, final Response response, final Content content) {
    if (!GET.equals(context.getRequest().getAction())) {
      return false;
    }
    int status = response.getStatus().getCode();
    if (status == HttpStatus.PARTIAL_CONTENT) {
      return isRangeFromFirstByte(response.getHeaders().get(HttpHeaders.CONTENT_RANGE));
    }
    if (status != HttpStatus.OK) {
      return false;
    }
    Headers validators = validators(response, content);
    return !isNotModified(context, response, content, validators)
        && !isPartial(context.getRequest(), content, validators);
  }

  /**
   * The validators the client sees, set by the content headers handler once the response leaves this handler.
   */
  private static Headers validators(final Response response, final Content content) {
    Headers validators = new Headers();
    String etag = response.getHeaders().get(HttpHeaders.ETAG);
    if (etag == null) {
      String contentEtag = content.getAttributes().get(Content.CONTENT_ETAG, String.class);
      etag = contentEtag != null ? ETagHeaderUtils.quote(contentEtag) : null;
    }
    if (etag != null) {
      validators.set(HttpHeaders.ETAG, etag);
    }
    String lastModified = response.getHeaders().get(HttpHeaders.LAST_MODIFIED);
    if (lastModified == null) {
      DateTime contentLastModified = content.getAttributes().get(Content.CONTENT_LAST_MODIFIED, DateTime.class);
      lastModified = contentLastModified != null ? formatDateTime(contentLastModified) : null;
    }
    if (lastModified != null) {
      validators.set(HttpHeaders.LAST_MODIFIED, lastModified);
    }
    return validators;
  }

  /**
   * Whether the conditional request handler answers the request with 304 (or 412) instead of the content.
   */
  private static boolean isNotModified(
      final Context context,
      final Response response,
      final Content content,
      final Headers validators)
  {
    if (!context.getRequest().getAttributes().contains(HttpConditions.HTTP_CONDITIONS)) {
      return false;
    }
    Response clientResponse = new Response.Builder()
        .status(response.getStatus())
        .headers(validators)
        .payload(content)
        .build();
    return HttpConditions.maybeCreateConditionalResponse(context, clientResponse).isPresent();
  }

  /**
   * Whether the partial fetch handler answers the request with a range that does not start at the first byte, or
   * cannot be satisfied.
   */
  private static boolean isPartial(final Request request, final Content content, final Headers validators) {
    String range = request.getHeaders().get(HttpHeaders.RANGE);
    long size = content.getSize();
    if (range == null || size == Payload.UNKNOWN_SIZE) {
      return false;
    }
    String ifRange = request.getHeaders().get(HttpHeaders.IF_RANGE);
    if (ifRange != null && !ifRangeMatches(ifRange, validators)) {
      return false;
    }
    Long start = rangeStart(range, size);
    return start != null && start != 0L;
  }

  /**
   * Returns the first byte of the single range requested, {@code -1} when it cannot be satisfied, or {@code null} when
   * the whole content is sent anyway; mirrors the range parser of the partial fetch handler.
   */
  @Nullable
  private static Long rangeStart(final String range, final long size) {
    String prefix = BYTES_UNIT + "=";
    if (!range.startsWith(prefix) || range.length() <= prefix.length() || range.contains(",")) {
      return null;
    }
    String spec = range.substring(prefix.length());
    try {
      long start;
      long end = size - 1;
      if (spec.startsWith("-")) {
        long suffix = Long.parseLong(spec.substring(1));
        start = size - suffix;
      }
      else if (spec.endsWith("-")) {
        start = Long.parseLong(spec.substring(0, spec.length() - 1));
      }
      else if (spec.contains("-")) {
        String[] bounds = spec.split("-");
        start = Long.parseLong(bounds[0]);
        if (Long.parseLong(bounds[1]) < start) {
          return null;
        }
      }
      else {
        return null;
      }
      return start < 0 || start > end ? -1L : start;
    }
    catch (NumberFormatException | ArrayIndexOutOfBoundsException e) {
      return null;
    }
  }

  private static boolean ifRangeMatches(final String ifRange, final Headers validators) {
    if (ifRange.startsWith("\"")) {
      return ifRange.equals(validators.get(HttpHeaders.ETAG));
    }
    Date ifRangeDate = DateUtils.parseDate(ifRange);
    String lastModified = validators.get(HttpHeaders.LAST_MODIFIED);
    Date lastModifiedDate = Strings.isNullOrEmpty(lastModified) ? null : DateUtils.parseDate(lastModified);
    return ifRangeDate != null && lastModifiedDate != null && ifRangeDate.getTime() == lastModifiedDate.getTime();
  }

  private static boolean isRangeFromFirstByte(@Nullable final String contentRange) {
    return contentRange != null && contentRange.startsWith(BYTES_UNIT + " 0-");
  }
}
