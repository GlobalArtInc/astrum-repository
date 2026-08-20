

import React from 'react';

import './GASearchPagination.scss';

export interface GASearchPaginationProps {
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
 * Pagination component for GA search results.
 * Uses "Load More" pattern instead of page numbers.
 */
export function GASearchPagination({
  hasMore,
  loading,
  loadedCount,
  totalCount,
  onLoadMore,
}: GASearchPaginationProps): JSX.Element | null {
  if (!hasMore && loadedCount >= totalCount) {
    return null;
  }

  return (
    <div className="ga-search-pagination">
      <div className="ga-search-pagination__info">
        Showing {loadedCount} of {totalCount} results
      </div>

      {hasMore && (
        <button
          type="button"
          className="ga-search-pagination__button"
          onClick={onLoadMore}
          disabled={loading}
        >
          {loading ? 'Loading...' : 'Load More'}
        </button>
      )}
    </div>
  );
}

export default GASearchPagination;

