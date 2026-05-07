
package org.globalart.astrum.webresources;

import java.util.Collection;

import javax.annotation.Nullable;

/**
 * Provides access to {@link WebResource} instances.
 *
 * @since 2.8
 */
public interface WebResourceService
{
  /**
   * Returns all discovered web-resource paths.
   */
  Collection<String> getPaths();

  /**
   * Returns all discovered web-resources.
   */
  Collection<WebResource> getResources();

  /**
   * Get a web-resource by path.
   *
   * @return Web-resource for path or null if not bound.
   */
  @Nullable
  WebResource getResource(String path);
}
