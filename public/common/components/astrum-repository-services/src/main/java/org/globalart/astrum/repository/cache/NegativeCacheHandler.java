
package org.globalart.astrum.repository.cache;

import java.util.Set;

import javax.annotation.Nonnull;

import org.globalart.astrum.common.node.NodeAccess;
import org.globalart.astrum.repository.MissingFacetException;
import org.globalart.astrum.repository.http.HttpMethods;
import org.globalart.astrum.repository.http.HttpStatus;
import org.globalart.astrum.repository.httpclient.HttpClientFacet;
import org.globalart.astrum.repository.httpclient.RemoteConnectionStatus;
import org.globalart.astrum.repository.httpclient.RemoteConnectionStatusType;
import org.globalart.astrum.repository.proxy.ProxyFacet;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.Response;
import org.globalart.astrum.repository.view.Status;

import com.google.common.collect.ImmutableSet;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.context.annotation.Primary;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.repository.proxy.ProxyFacetSupport.MISSING_BLOB_SKIP_NEGATIVE_CACHE;
import static org.globalart.astrum.repository.replication.PullReplicationSupport.IS_REPLICATION_REQUEST;

/**
 * Handler that caches 404 responses.
 * When context invocation returns 404, it caches the 404 status to avoid future invocations (if cached status is
 * present).
 *
 * <p>
 * In HA deployments the local content store is shared across nodes (Postgres metadata + shared blob store). If a
 * peer node has populated that store since this node cached its 404, the entry here is stale and returning the
 * cached 404 would produce divergent responses across the cluster. On clustered deployments this handler consults
 * the proxy facet before honoring a cached 404; if the path already has content locally, the stale NFC entry is
 * invalidated and the request is allowed to proceed so the downstream proxy handler can serve the cluster-cached
 * asset. The check is skipped entirely on non-clustered deployments, where no peer can have populated content
 * behind this node's back — this keeps NFC's shielding path near-zero-cost for CE and single-node Pro deployments
 * hammered with 404 traffic.
 *
 * @since 3.0
 */
@Primary
@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class NegativeCacheHandler
    implements Handler
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private static final Set<String> NFC_CACHEABLE_ACTIONS = ImmutableSet.of(HttpMethods.GET, HttpMethods.HEAD);

  private final NodeAccess nodeAccess;

  @Autowired
  public NegativeCacheHandler(final NodeAccess nodeAccess) {
    this.nodeAccess = checkNotNull(nodeAccess);
  }

  @Nonnull
  @Override
  public Response handle(@Nonnull final Context context) throws Exception {
    String action = context.getRequest().getAction();
    if (!NFC_CACHEABLE_ACTIONS.contains(action)) {
      return context.proceed();
    }

    NegativeCacheFacet negativeCache = context.getRepository().facet(NegativeCacheFacet.class);
    NegativeCacheKey key = negativeCache.getCacheKey(context);

    log.debug("Handling NotFoundCache for {}", key);

    if (context.getAttributes() != null && Boolean.TRUE.equals(
        context.getAttributes().get(IS_REPLICATION_REQUEST, Boolean.class, false))) {
      log.debug("Skipping NotFoundCache for {} as this is a replication request.", key);
      Response response = context.proceed();
      if (response.getStatus().isSuccessful()) {
        negativeCache.invalidate(key);
      }
      return response;
    }

    Response response;
    Status status = negativeCache.get(key);
    if (status == null) {
      response = context.proceed();

      if (isNotFound(response) && !isRemoteBlocked(context) && !skipNegativeCacheForMissingBlob(context)) {
        negativeCache.put(key, response.getStatus());
        log.debug("Couldn't find {} - adding to NFC", key);
      }
      else if (response.getStatus().isSuccessful()) {
        negativeCache.invalidate(key);
        log.debug("Found {} - invalidating NFC", key);
      }
    }
    else if (hasLocalContent(context)) {
      // A peer node has populated the shared local store since this node cached the 404. Drop the stale NFC entry
      // and let the request proceed so the downstream handler serves the cluster-cached asset.
      negativeCache.invalidate(key);
      log.debug("Local content present for {} - invalidating stale NFC entry and proceeding", key);
      response = context.proceed();
    }
    else {
      response = buildResponse(status, context);

      log.debug("Found {} in negative cache, returning {}", key, response);
    }
    return response;
  }

  private boolean hasLocalContent(final Context context) {
    // Skip the storage probe on non-clustered deployments: no peer can have populated content behind our back,
    // so this check would always return false and only add DB load to every NFC-hit request. The NFC fast path
    // (near-zero-cost cached-404 response) is what CE and single-node Pro deployments rely on for handling
    // high-404 workloads.
    if (!nodeAccess.isClustered()) {
      return false;
    }
    try {
      return context.getRepository().facet(ProxyFacet.class).hasContentFor(context);
    }
    catch (MissingFacetException e) {
      // Non-proxy repositories (hosted, group) have no ProxyFacet to probe. Fall through to serving the
      // cached 404 as before. Group NFC divergence is a related but separate problem — a peer could
      // populate content into one of the group's members while this node's group NFC still holds a 404;
      // fixing that would require probing each member. Left for a follow-up.
      return false;
    }
  }

  protected Response buildResponse(final Status status, final Context context) {
    return new Response.Builder()
        .status(status)
        .build();
  }

  private boolean isNotFound(final Response response) {
    return HttpStatus.NOT_FOUND == response.getStatus().getCode();
  }

  private boolean isRemoteBlocked(final Context context) {
    RemoteConnectionStatus remoteConnectionStatus = context.getRepository().facet(HttpClientFacet.class).getStatus();
    return remoteConnectionStatus.getType() == RemoteConnectionStatusType.AUTO_BLOCKED_UNAVAILABLE ||
        remoteConnectionStatus.getType() == RemoteConnectionStatusType.BLOCKED;
  }

  private boolean skipNegativeCacheForMissingBlob(final Context context) {
    if (context.getAttributes() == null) {
      return false;
    }
    return Boolean.TRUE.equals(context.getAttributes().get(MISSING_BLOB_SKIP_NEGATIVE_CACHE));
  }
}
