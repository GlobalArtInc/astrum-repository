

/**
 * RubyGems Search Module
 *
 * Public exports for RubyGems search components.
 */

// Types
export type {
  RubyGemsSearchFilters,
  RubyGemsResult,
  RubyGemsSearchResponse,
  RubyGemsVersion,
  RubyGemsDetail,
  RubyGemsSearchState,
} from './rubygems.types';

// Main page components
export { RubyGemsSearchPage } from './RubyGemsSearchPage';
export type { RubyGemsSearchPageProps } from './RubyGemsSearchPage';

export { RubyGemsDetailPage } from './RubyGemsDetailPage';
export type { RubyGemsDetailPageProps } from './RubyGemsDetailPage';

// Sub-components
export { RubyGemsSearchFilters } from './RubyGemsSearchFilters';
export type { RubyGemsSearchFiltersProps } from './RubyGemsSearchFilters';

export { RubyGemsSearchResults } from './RubyGemsSearchResults';
export type { RubyGemsSearchResultsProps } from './RubyGemsSearchResults';

export { RubyGemsResultRow } from './RubyGemsResultRow';
export type { RubyGemsResultRowProps } from './RubyGemsResultRow';

// Hook
export { useRubyGemsSearch } from './useRubyGemsSearch';
export type { UseRubyGemsSearchReturn } from './useRubyGemsSearch';

// Mock data (for development only)
export { mockRubyGemsResults, mockRubyGemsSearchApi, mockRubyGemsDetailApi } from './mockData';


