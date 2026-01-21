
package org.globalart.astrum.blobstore.api;

import java.util.List;

public class PaginatedResult<T>
{
  private final List<T> results;

  private final String nextContinuationToken;

  public PaginatedResult(List<T> results, String nextContinuationToken) {
    this.results = results;
    this.nextContinuationToken = nextContinuationToken;
  }

  public List<T> getResults() {
    return results;
  }

  public String getNextContinuationToken() {
    return nextContinuationToken;
  }
}
