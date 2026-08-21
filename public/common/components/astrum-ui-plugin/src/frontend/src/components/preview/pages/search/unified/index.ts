

/**
 * Unified Search Module
 * 
 * Single search page that supports all formats with dynamic filters.
 */

// Types
export type {
  SearchFormat,
  FormatInfo,
  FilterType,
  FilterDefinition,
  FilterValues,
  FormatFilterConfig,
  SearchResult,
  SearchResponse,
  SortField,
  SortDirection,
  UnifiedSearchState,
  SearchHeaderProps,
  SearchSidebarProps,
  SearchResultsProps,
  SearchResultCardProps,
} from './unified.types';

// Filter definitions
export {
  FORMATS,
  FORMAT_FILTERS,
  getFormatOptions,
  getFiltersForFormat,
  getPlaceholderForFormat,
  getApiFormat,
  buildQueryParams,
} from './searchFilters';

// Hooks
export { useUnifiedSearch } from './useUnifiedSearch';
export type { UseUnifiedSearchReturn } from './useUnifiedSearch';

// Agent 1: Repository data hook
export { useRepositories } from './useRepositories';
export type { Repository, UseRepositoriesResult } from './useRepositories';

// Agent 2: URL state sync hook
export { useSearchUrlState } from './useSearchUrlState';
export type { UrlSearchState, UseSearchUrlStateReturn } from './useSearchUrlState';

// Agent 3: Navigation hook
export { useSearchNavigation, consumeSearchReturnUrl } from './useSearchNavigation';
export type { DetailRoute, UseSearchNavigationReturn } from './useSearchNavigation';
export { SEARCH_RETURN_URL_KEY } from './useSearchNavigation';

// Components
export { default as SearchHeader } from './SearchHeader';

// Filter components
export { FilterSection, TextFilter, SelectFilter } from './filters';
export type { FilterSectionProps, TextFilterProps, SelectFilterProps, SelectFilterOption } from './filters';
