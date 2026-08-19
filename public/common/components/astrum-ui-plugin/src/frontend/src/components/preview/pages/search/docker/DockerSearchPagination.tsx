

import React from 'react';

import './DockerSearchPagination.scss';

export interface DockerSearchPaginationProps {
  /** Whether more results are available */
  hasMore: boolean;
  /** Whether loading is in progress */
  loading: boolean;
  /** Current count of loaded results */
  loadedCount: number;
  /** Total count of available results */
  totalCount: number;
  /** Callback to load more results */
  onLoadMore: () => void;
}

/**
 * Pagination component for Docker search results.
 * Uses "Load More" pattern instead of page numbers.
 */
export function DockerSearchPagination({
  hasMore,
  loading,
  loadedCount,
  totalCount,
  onLoadMore,
}: DockerSearchPaginationProps): JSX.Element | null {
  if (!hasMore && loadedCount >= totalCount) {
    return null;
  }

  return (
    <div className="docker-search-pagination">
      <div className="docker-search-pagination__info">
        Showing {loadedCount} of {totalCount} results
      </div>

      {hasMore && (
        <button
          type="button"
          className="docker-search-pagination__button"
          onClick={onLoadMore}
          disabled={loading}
        >
          {loading ? 'Loading...' : 'Load More'}
        </button>
      )}
    </div>
  );
}

export default DockerSearchPagination;


