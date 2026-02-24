
package org.globalart.astrum.repository.search.query;

import java.util.List;

import org.globalart.astrum.repository.search.ComponentSearchResult;
import org.globalart.astrum.repository.search.SearchResponse;

/**
 * Generates search results consumable by the UI
 *
 * @since 3.14
 */
public interface SearchResultsGenerator
{
  List<ComponentSearchResult> getSearchResultList(SearchResponse response);
}
