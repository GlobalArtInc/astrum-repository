
package org.globalart.astrum.repository.apt.datastore.internal.hosted;

import java.io.IOException;
import java.util.Optional;

import javax.annotation.Nonnull;
import org.globalart.astrum.repository.apt.datastore.AptContentFacet;
import org.globalart.astrum.repository.apt.datastore.internal.metadata.AptMetadataRebuildSchedulerFacet;
import org.globalart.astrum.repository.apt.internal.AptFacetHelper;
import org.globalart.astrum.repository.apt.internal.AptPackageParser;
import org.globalart.astrum.repository.apt.internal.debian.ControlFile;
import org.globalart.astrum.repository.apt.internal.debian.PackageInfo;
import org.globalart.astrum.repository.apt.internal.snapshot.AptSnapshotHandler;
import org.globalart.astrum.repository.http.HttpResponses;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.Response;
import org.globalart.astrum.repository.view.payloads.StreamPayload;
import org.globalart.astrum.repository.view.payloads.TempBlob;

import org.apache.commons.lang3.StringUtils;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static org.globalart.astrum.repository.apt.internal.ReleaseName.INRELEASE;
import static org.globalart.astrum.repository.http.HttpMethods.GET;
import static org.globalart.astrum.repository.http.HttpMethods.HEAD;
import static org.globalart.astrum.repository.http.HttpMethods.POST;
import org.springframework.stereotype.Component;

/**
 * Apt handlers
 *
 * @since 3.31
 */
@Component
public class AptHostedHandler
    implements Handler
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  @Nonnull
  @Override
  public Response handle(@Nonnull final Context context) throws Exception {
    String path = assetPath(context);
    String method = context.getRequest().getAction();
    AptContentFacet contentFacet = context.getRepository().facet(AptContentFacet.class);

    return switch (method) {
      case GET, HEAD -> doGet(context, path, contentFacet);
      case POST -> doPost(context, path, contentFacet);
      default -> HttpResponses.methodNotAllowed(method, GET, HEAD, POST);
    };
  }

  private Response doGet(
      final Context context,
      final String path,
      final AptContentFacet contentFacet)
  {
    if (isMetadataRebuildRequired(path, contentFacet)) {
      context.getRepository().facet(AptMetadataRebuildSchedulerFacet.class).maybeScheduleRebuild();
    }
    Optional<Content> content = contentFacet.get(path);
    return content.isPresent() ? HttpResponses.ok(content.get()) : HttpResponses.notFound(path);
  }

  private Response doPost(
      final Context context,
      final String path,
      final AptContentFacet contentFacet) throws IOException
  {
    final AptHostedFacet hostedFacet = context.getRepository().facet(AptHostedFacet.class);
    if ("rebuild-indexes".equals(path)) {
      context.getRepository().facet(AptMetadataRebuildSchedulerFacet.class).maybeScheduleRebuild();
      return HttpResponses.ok();
    }
    else if (StringUtils.isBlank(path)) {
      final Payload payload = context.getRequest().getPayload();
      try (TempBlob tempBlob = contentFacet.getTempBlob(payload)) {
        ControlFile controlFile = AptPackageParser
            .parsePackageInfo(tempBlob)
            .getControlFile();
        String assetPath = AptFacetHelper.buildAssetPath(controlFile);
        long payloadSize = payload.getSize();
        String contentType = payload.getContentType();

        hostedFacet.put(assetPath, new StreamPayload(tempBlob, payloadSize, contentType),
            new PackageInfo(controlFile));
      }
      return HttpResponses.created();
    }
    else {
      return HttpResponses.methodNotAllowed(POST, GET, HEAD);
    }
  }

  private boolean isMetadataRebuildRequired(final String path, final AptContentFacet contentFacet) {
    if (path.startsWith("dists")) {
      String inReleasePath = "dists/" + contentFacet.getDistribution() + "/" + INRELEASE;
      return contentFacet.get(inReleasePath).isEmpty();
    }
    return false;
  }

  private String assetPath(final Context context) {
    return context.getAttributes().require(AptSnapshotHandler.State.class).assetPath;
  }
}
