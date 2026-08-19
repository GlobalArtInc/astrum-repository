

/**
 * Go (Golang) Search Module
 *
 * Public exports for Go module search components.
 */

// Types
export type {
  GolangSearchFilters,
  GolangResult,
  GolangSearchResponse,
  GolangVersion,
  GolangDetail,
  GolangSearchState,
} from './golang.types';

// Main page components
export { GolangSearchPage } from './GolangSearchPage';
export type { GolangSearchPageProps } from './GolangSearchPage';

export { GolangDetailPage } from './GolangDetailPage';
export type { GolangDetailPageProps } from './GolangDetailPage';

// Sub-components
export { GolangSearchFilters } from './GolangSearchFilters';
export type { GolangSearchFiltersProps } from './GolangSearchFilters';

export { GolangSearchResults } from './GolangSearchResults';
export type { GolangSearchResultsProps } from './GolangSearchResults';

export { GolangResultRow } from './GolangResultRow';
export type { GolangResultRowProps } from './GolangResultRow';

// Hook
export { useGolangSearch } from './useGolangSearch';
export type { UseGolangSearchReturn } from './useGolangSearch';

// Mock data (for development only)
export { mockGolangResults, mockGolangSearchApi, mockGolangDetailApi } from './mockData';


