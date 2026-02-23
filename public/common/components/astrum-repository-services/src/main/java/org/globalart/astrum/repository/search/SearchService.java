
package org.globalart.astrum.repository.search;

/**
 * This defines the db-agnostic interface for searching. All search requests should go through this.
 *
 * @since 3.38
 */
public interface SearchService
{
  /**
   * Search for components with the passed in {@link SearchRequest}
   *
   * @since 3.38
   */
  SearchResponse search(SearchRequest searchRequest);

  /**
   * Browse through all of the components returned
   *
   * @since 3.38
   */
  Iterable<ComponentSearchResult> browse(SearchRequest searchRequest);

  /**
   * Return the count of components matching the {@link SearchRequest}
   *
   * @since 3.38
   */
  long count(SearchRequest searchRequest);

  default void waitForCalm() {
    // not required for default
  }

  default void waitForReady() {
    // not required for default
  }

  /**
   * Synchronously flush any pending search-index updates for the given repository.
   * Lets a search caller observe the writes that have already happened in this
   * repository, eliminating the eventual-consistency window from the asynchronous
   * batched indexer. Default implementation is a no-op for backends that index
   * synchronously.
   */
  default void flushPendingIndex(final String repositoryName) {
    // not required for default
  }
}
