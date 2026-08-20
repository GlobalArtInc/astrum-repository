

/**
 * npm Search Module
 *
 * Public exports for npm package search components.
 */

// Types
export type {
  NpmSearchFilters,
  NpmResult,
  NpmSearchResponse,
  NpmVersion,
  NpmDetail,
  NpmSearchState,
} from './npm.types';

// Main page components
export { NpmSearchPage } from './NpmSearchPage';
export type { NpmSearchPageProps } from './NpmSearchPage';

export { NpmDetailPage } from './NpmDetailPage';
export type { NpmDetailPageProps } from './NpmDetailPage';

// Sub-components
export { NpmSearchFilters } from './NpmSearchFilters';
export type { NpmSearchFiltersProps } from './NpmSearchFilters';

export { NpmSearchResults } from './NpmSearchResults';
export type { NpmSearchResultsProps } from './NpmSearchResults';

export { NpmResultRow } from './NpmResultRow';
export type { NpmResultRowProps } from './NpmResultRow';

// Hook
export { useNpmSearch } from './useNpmSearch';
export type { UseNpmSearchReturn } from './useNpmSearch';

// Mock data (for development only)
export { mockNpmResults, mockNpmSearchApi, mockNpmDetailApi } from './mockData';

