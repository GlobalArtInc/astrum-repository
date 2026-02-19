
package org.globalart.astrum.repository.rest;

import org.globalart.astrum.repository.rest.api.ComponentXO;
import org.globalart.astrum.repository.rest.internal.resources.SearchResource;
import org.globalart.astrum.repository.search.ComponentSearchResult;

/**
 * Extension point for the {@link SearchResource} class
 *
 * @since 3.8
 */
public interface SearchResourceExtension
{
  /**
   * Update the {@link ComponentXO} with data from the {@link ComponentSearchResult}
   *
   * @since 3.38
   */
  ComponentXO updateComponentXO(ComponentXO componentXO, ComponentSearchResult hit);
}
