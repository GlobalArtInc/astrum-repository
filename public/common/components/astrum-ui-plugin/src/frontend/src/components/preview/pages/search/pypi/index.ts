

/**
 * PyPI Search Module
 *
 * Public exports for PyPI (Python Package Index) search components.
 */

// Types
export type {
  PyPISearchFilters,
  PyPIResult,
  PyPISearchResponse,
  PyPIVersion,
  PyPIDetail,
  PyPISearchState,
} from './pypi.types';

// Main page components
export { PyPISearchPage } from './PyPISearchPage';
export type { PyPISearchPageProps } from './PyPISearchPage';

export { PyPIDetailPage } from './PyPIDetailPage';
export type { PyPIDetailPageProps } from './PyPIDetailPage';

// Sub-components
export { PyPISearchFilters } from './PyPISearchFilters';
export type { PyPISearchFiltersProps } from './PyPISearchFilters';

export { PyPISearchResults } from './PyPISearchResults';
export type { PyPISearchResultsProps } from './PyPISearchResults';

export { PyPIResultRow } from './PyPIResultRow';
export type { PyPIResultRowProps } from './PyPIResultRow';

// Hook
export { usePyPISearch } from './usePyPISearch';
export type { UsePyPISearchReturn } from './usePyPISearch';

// Mock data (for development only)
export { mockPyPIResults, mockPyPISearchApi, mockPyPIDetailApi } from './mockData';


