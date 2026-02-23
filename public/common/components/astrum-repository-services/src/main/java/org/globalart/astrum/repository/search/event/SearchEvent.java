
package org.globalart.astrum.repository.search.event;

import java.util.Collection;

import org.globalart.astrum.repository.search.query.SearchFilter;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Fired to denote a search has occurred.
 */
public class SearchEvent
{
  /**
   * The search criteria used in the search. Note, this is not the values that are searched, just the criteria.
   * repository, format, keyword, etc
   */
  private final Collection<SearchFilter> searchFilters;

  private final SearchEventSource source;

  public SearchEvent(
      final Collection<SearchFilter> searchFilters,
      final SearchEventSource source)
  {
    this.searchFilters = checkNotNull(searchFilters);
    this.source = checkNotNull(source);
  }

  public SearchEventSource getSource() {
    return source;
  }

  public Collection<SearchFilter> getSearchFilters() {
    return searchFilters;
  }
}
