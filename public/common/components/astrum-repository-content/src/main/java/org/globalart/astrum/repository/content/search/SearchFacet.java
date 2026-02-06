
package org.globalart.astrum.repository.content.search;

import java.util.Collection;

import org.globalart.astrum.common.entity.EntityId;
import org.globalart.astrum.repository.Facet;
import org.globalart.astrum.repository.search.index.SearchIndexFacet;

/**
 * Search {@link Facet}, that indexes/de-indexes component metadata.
 *
 * @since 3.25
 */
@Facet.Exposed
public interface SearchFacet
    extends SearchIndexFacet
{
  /**
   * Adds the given components to the search index.
   *
   * @param componentIds the components to index
   */
  void index(Collection<EntityId> componentIds);

  /**
   * Removes the given components from the search index.
   *
   * @param componentIds the components to purge
   */
  void purge(Collection<EntityId> componentIds);
}
