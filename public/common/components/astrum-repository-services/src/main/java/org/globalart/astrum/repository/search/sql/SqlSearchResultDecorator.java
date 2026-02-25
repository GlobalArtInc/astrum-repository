
package org.globalart.astrum.repository.search.sql;

import org.globalart.astrum.repository.search.ComponentSearchResult;

/**
 * Allows for data to be contributed to the {@link ComponentSearchResult}. Implement this interface when you need to
 * contribute data depending on format, field or some other condition which is not true for every row in the search
 * table.
 */
public interface SqlSearchResultDecorator
{
  void updateComponent(ComponentSearchResult component, SearchResult searchResult);
}
