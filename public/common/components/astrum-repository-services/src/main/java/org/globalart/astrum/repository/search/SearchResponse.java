
package org.globalart.astrum.repository.search;

import java.util.List;

/**
 * @since 3.38
 */
public class SearchResponse
{
  private List<ComponentSearchResult> searchResults;

  private String continuationToken;

  private Long totalHits;

  public String getContinuationToken() {
    return continuationToken;
  }

  public List<ComponentSearchResult> getSearchResults() {
    return searchResults;
  }

  public Long getTotalHits() {
    return totalHits;
  }

  public void setContinuationToken(final String continuationToken) {
    this.continuationToken = continuationToken;
  }

  public void setSearchResults(final List<ComponentSearchResult> searchResults) {
    this.searchResults = searchResults;
  }

  public void setTotalHits(final Long totalHits) {
    this.totalHits = totalHits;
  }
}
