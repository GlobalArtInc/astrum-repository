

/**
 * GA Search Results Module
 *
 * Public exports for the search results components.
 */

// Main page component
export { GASearchPage } from './GASearchPage';
export type { GASearchPageProps } from './GASearchPage';

// Sub-components (for customization)
export { GASearchInput } from './GASearchInput';
export type { GASearchInputProps } from './GASearchInput';

export { GASearchFilters } from './GASearchFilters';
export type { GASearchFiltersProps, FilterValues } from './GASearchFilters';

export { GASearchResults } from './GASearchResults';
export type { GASearchResultsProps } from './GASearchResults';

export { GAResultRow } from './GAResultRow';
export type { GAResultRowProps } from './GAResultRow';

export { GASearchPagination } from './GASearchPagination';
export type { GASearchPaginationProps } from './GASearchPagination';

// Hook
export { useGASearch } from './useGASearch';
export type { UseGASearchReturn, SearchParams } from './useGASearch';

// Mock data (for development only - remove when backend is ready)
export { mockResults, mockSearchApi, mockSuggestApi } from './mockData';

