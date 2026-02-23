
package org.globalart.astrum.repository.search.index;

import org.globalart.astrum.repository.Facet;
import org.globalart.astrum.repository.Repository;

/**
 * Search index {@link Facet}.
 *
 * @since 3.25
 */
@Facet.Exposed
public interface SearchIndexFacet
    extends Facet
{
  /**
   * Rebuilds the index of the {@link Repository} this facet is attached to.
   */
  void rebuildIndex();
}
