
package org.globalart.astrum.repository.proxy;

import java.io.IOException;
import java.net.URI;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.Facet;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Context;

/**
 * A format neutral proxy facet.
 *
 * @since 3.0
 */
@Facet.Exposed
public interface ProxyFacet
    extends Facet
{
  /**
   * Obtain the content which the user has requested, either by retrieving cached content, or by fetching new or
   * updated content from the upstream repository.
   */
  @Nullable
  Content get(Context context) throws IOException;

  /**
   * Returns the root of the remote repository.
   */
  URI getRemoteUrl();

  /**
   * Invalidates all entries from proxy cache, causing all subsequent requests to attempt to fetch new or updated
   * content, if any.
   */
  void invalidateProxyCaches();

  /**
   * Returns the configuration object used by the proxy facet
   */
  ProxyRepositoryConfiguration getConfiguration();

  /**
   * Returns {@code true} if this proxy repository has content for the request path available in the shared local
   * store. Does not initiate any remote fetch. Intended for HA coordination decisions where a peer node may have
   * populated the shared store since this node last observed the path (for example, allowing a stale negative-cache
   * entry to be bypassed once the asset is available locally).
   */
  boolean hasContentFor(Context context);
}
