
package org.globalart.astrum.repository.proxy;

import java.io.IOException;
import java.io.UncheckedIOException;

import javax.annotation.Nonnull;
import javax.annotation.Nullable;

import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.blobstore.api.BlobStoreWarmingUpException;
import org.globalart.astrum.common.io.CooperationException;
import org.globalart.astrum.common.node.NodeAccess;
import org.globalart.astrum.common.stateguard.InvalidStateException;
import org.globalart.astrum.repository.MissingBlobException;
import org.globalart.astrum.repository.http.HttpResponses;
import org.globalart.astrum.repository.httpclient.RemoteBlockedIOException;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.Headers;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.Response;
import org.globalart.astrum.repository.view.Status;
import org.globalart.astrum.repository.view.payloads.StringPayload;

import org.springframework.context.annotation.Primary;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static java.lang.Boolean.TRUE;
import static org.globalart.astrum.repository.http.HttpMethods.GET;
import static org.globalart.astrum.repository.http.HttpMethods.HEAD;
import static org.globalart.astrum.repository.proxy.ProxyFacetSupport.PROXY_REMOTE_FETCH_SKIP_MARKER;
import static org.globalart.astrum.repository.proxy.ProxyFacetSupport.PROXY_TELEMETRY_BLOCKING_MARKER;
import static org.globalart.astrum.repository.proxy.ThrottlerInterceptor.PAYMENT_REQUIRED_MESSAGE;

/**
 * A format-neutral proxy handler which delegates to an instance of {@link ProxyFacet} for content.
 *
 * @since 3.0
 */
@Primary
@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class ProxyHandler
    implements Handler
{
  private static final String TELEMETRY_READONLY_MODE =
      "Astrum Repository is now in read-only mode because Baseline Telemetry data could not be uploaded within the grace period. Check your network configuration and verify that the server can reach the telemetry service. If the problem persists, contact GlobalArt Support. To retry the upload now, run the Upload Retry task.";

  protected final Logger log = LoggerFactory.getLogger(getClass());

  @Autowired
  private NodeAccess nodeAccess;

  @Nonnull
  @Override
  public Response handle(@Nonnull final Context context) throws Exception { // NOSONAR
    final Response response = buildMethodNotAllowedResponse(context);
    if (response != null) {
      return response;
    }

    try {
      Payload payload = proxyFacet(context).get(context);
      if (payload != null) {
        return buildPayloadResponse(context, payload);
      }
      if (isMarkerSet(context, PROXY_REMOTE_FETCH_SKIP_MARKER)) {
        return buildPaymentRequiredResponse(context);
      }
      if (isMarkerSet(context, PROXY_TELEMETRY_BLOCKING_MARKER)) {
        return buildTelemetryBlockingResponse(context);
      }
      return buildNotFoundResponse(context);
    }
    catch (BypassHttpErrorException e) {
      return buildHttpErrorResponse(e);
    }
    catch (ProxyServiceException e) {
      return HttpResponses.serviceUnavailable();
    }
    catch (CooperationException e) { // NOSONAR
      return HttpResponses.serviceUnavailable(e.getMessage());
    }
    catch (RemoteBlockedIOException e) {
      return HttpResponses.notFound(e.getMessage());
    }
    catch (BlobStoreWarmingUpException e) {
      // Blob store connection pool is still initializing (temporary, retry-able)
      log.info("Blob store '{}' warming up for {}, returning 503 to trigger client retry",
          e.getBlobStoreName(), context.getRequest().getPath());
      return HttpResponses.serviceUnavailable("Blob store warming up, please retry in a moment");
    }
    catch (MissingBlobException e) {
      // CRITICAL: Blob exists in metadata but missing from storage (data corruption, not retry-able)
      log.error("BLOB DATA LOSS: Blob {} missing from storage for {} - data corruption",
          e.getBlobRef(), context.getRequest().getPath());
      throw e; // Propagate as 500 error
    }
    catch (InvalidStateException e) {
      // Generic invalid state (e.g., stopped repository - not retry-able)
      log.warn("Invalid state for {}: {}",
          context.getRequest().getPath(), e.getMessage());
      throw e; // Propagate as 500 error
    }
    catch (IOException | UncheckedIOException e) {
      // Some format facets wrap RemoteBlockedIOException in UncheckedIOException (e.g.
      // when the block signal has to escape a method that cannot declare throws IOException,
      // as PyPI does in cachePackageRootMetadataAndRetrieveLink). Unwrap once so those callers
      // still get the "Remote Auto Blocked until …" 404 that a raw RemoteBlockedIOException
      // produces, instead of a generic 502 that drops the block message.
      if (e instanceof UncheckedIOException && e.getCause() instanceof RemoteBlockedIOException) {
        return HttpResponses.notFound(e.getCause().getMessage());
      }
      return HttpResponses.badGateway();
    }
    catch (Exception e) {
      // Walk the cause chain looking for signal exceptions. Format facets often wrap
      // RemoteBlockedIOException (or IOException in general) in a plain RuntimeException so it
      // can escape a call site that can't declare throws IOException (see YumProxyFacet's
      // fetchMetadata for a canonical example). Without unwrapping, the block signal is lost
      // and the client sees a generic 500.
      Throwable cause = e.getCause();
      while (cause != null) {
        if (cause instanceof BlobStoreWarmingUpException) {
          log.info("Blob store '{}' warming up for {} (wrapped), returning 503 to trigger client retry",
              ((BlobStoreWarmingUpException) cause).getBlobStoreName(),
              context.getRequest().getPath());
          return HttpResponses.serviceUnavailable("Blob store warming up, please retry in a moment");
        }
        if (cause instanceof RemoteBlockedIOException) {
          return HttpResponses.notFound(cause.getMessage());
        }
        if (cause instanceof IOException) {
          return HttpResponses.badGateway();
        }
        cause = cause.getCause();
      }
      throw e;
    }
  }

  /**
   * Builds a not-allowed response if the specified method is unsupported under the specified context, null otherwise.
   */
  @Nullable
  protected Response buildMethodNotAllowedResponse(final Context context) {
    final String action = context.getRequest().getAction();
    if (!GET.equals(action) && !HEAD.equals(action)) {
      return HttpResponses.methodNotAllowed(action, GET, HEAD);
    }
    return null;
  }

  protected Response buildPayloadResponse(final Context context, final Payload payload) {
    return HttpResponses.ok(payload);
  }

  protected Response buildNotFoundResponse(final Context context) {
    return HttpResponses.notFound();
  }

  protected Response buildPaymentRequiredResponse(final Context context) {
    if ("nuget".equals(context.getRepository().getFormat().getValue())) {
      return HttpResponses.conflict(PAYMENT_REQUIRED_MESSAGE.concat(nodeAccess.getId()));
    }
    else {
      return HttpResponses.forbidden(PAYMENT_REQUIRED_MESSAGE.concat(nodeAccess.getId()));
    }
  }

  private Response buildTelemetryBlockingResponse(final Context context) {
    log.warn(TELEMETRY_READONLY_MODE);
    if ("nuget".equals(context.getRepository().getFormat().getValue())) {
      return HttpResponses.conflict(TELEMETRY_READONLY_MODE);
    }
    else {
      return HttpResponses.forbidden(TELEMETRY_READONLY_MODE);
    }
  }

  private static boolean isMarkerSet(final Context context, final String markerName) {
    return context.getAttributes() != null &&
        context.getAttributes().contains(markerName) &&
        TRUE.equals(context.getAttributes().get(markerName));
  }

  protected Response buildHttpErrorResponse(final BypassHttpErrorException proxyErrorsException) {
    return new Response.Builder()
        .payload(getPayload(proxyErrorsException.getBody(), proxyErrorsException.getContentType()))
        .status(new Status(false, proxyErrorsException.getStatusCode(), proxyErrorsException.getReason()))
        .headers(new Headers(proxyErrorsException.getHeaders()))
        .build();
  }

  private Payload getPayload(final String body, final String contentType) {
    return new StringPayload(body, contentType);
  }

  private ProxyFacet proxyFacet(final Context context) {
    return context.getRepository().facet(ProxyFacet.class);
  }
}
