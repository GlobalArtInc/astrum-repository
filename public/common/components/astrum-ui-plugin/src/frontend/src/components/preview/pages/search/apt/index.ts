

/**
 * Apt/Debian Search Module
 *
 * Public exports for Apt (Debian/Ubuntu) package search components.
 */

// Types
export type {
  AptSearchFilters,
  AptResult,
  AptSearchResponse,
  AptVersion,
  AptDetail,
  AptSearchState,
} from './apt.types';

// Main page components
export { AptSearchPage } from './AptSearchPage';
export type { AptSearchPageProps } from './AptSearchPage';

export { AptDetailPage } from './AptDetailPage';
export type { AptDetailPageProps } from './AptDetailPage';

// Sub-components
export { AptSearchFilters } from './AptSearchFilters';
export type { AptSearchFiltersProps } from './AptSearchFilters';

export { AptSearchResults } from './AptSearchResults';
export type { AptSearchResultsProps } from './AptSearchResults';

export { AptResultRow } from './AptResultRow';
export type { AptResultRowProps } from './AptResultRow';

// Hook
export { useAptSearch } from './useAptSearch';
export type { UseAptSearchReturn } from './useAptSearch';

// Mock data (for development only)
export { mockAptResults, mockAptSearchApi, mockAptDetailApi } from './mockData';


