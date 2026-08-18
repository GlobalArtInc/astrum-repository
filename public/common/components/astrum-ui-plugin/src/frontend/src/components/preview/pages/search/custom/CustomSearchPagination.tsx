

import React from 'react';

import './CustomSearchPagination.scss';

export interface CustomSearchPaginationProps {
  /** Whether more results are available */
  hasMore: boolean;
  /** Whether currently loading */
  loading: boolean;
  /** Number of results currently loaded */
  loadedCount: number;
  /** Total count of results */
  totalCount: number;
  /** Callback to load more results */
  onLoadMore: () => void;
}

/**
 * Pagination component for custom search results.
 * Shows a "Load More" button when more results are available.
 */
export function CustomSearchPagination({
  hasMore,
  loading,
  loadedCount,
  totalCount,
  onLoadMore,
}: CustomSearchPaginationProps): JSX.Element | null {
  // Don't render if no results or no more to load
  if (totalCount === 0 || !hasMore) {
    return null;
  }

  return (
    <div className="custom-search-pagination">
      <div className="custom-search-pagination__info">
        Showing {loadedCount} of {totalCount} results
      </div>
      <button
        type="button"
        className="custom-search-pagination__load-more"
        onClick={onLoadMore}
        disabled={loading}
      >
        {loading ? 'Loading...' : 'Load More'}
      </button>
    </div>
  );
}

export default CustomSearchPagination;


