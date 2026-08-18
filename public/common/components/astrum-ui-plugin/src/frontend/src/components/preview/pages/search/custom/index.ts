

/**
 * Custom Search Module
 *
 * Provides a dynamic query builder UI for creating custom search filters.
 */

// Main page component
export { CustomSearchPage } from './CustomSearchPage';
export type { CustomSearchPageProps } from './CustomSearchPage';

// Builder components
export { CustomSearchBuilder } from './CustomSearchBuilder';
export type { CustomSearchBuilderProps } from './CustomSearchBuilder';

export { CustomFilterRow } from './CustomFilterRow';
export type { CustomFilterRowProps } from './CustomFilterRow';

// Results components
export { CustomSearchResults } from './CustomSearchResults';
export type { CustomSearchResultsProps } from './CustomSearchResults';

export { CustomSearchPagination } from './CustomSearchPagination';
export type { CustomSearchPaginationProps } from './CustomSearchPagination';

// Hook
export { useCustomSearch } from './useCustomSearch';
export type { UseCustomSearchReturn } from './useCustomSearch';

// Types
export type {
  CustomFilter,
  CustomSearchResult,
  CustomSearchResponse,
  CustomSearchState,
  FilterField,
  FilterOperator,
  FilterFieldOption,
  FilterOperatorOption,
} from './custom.types';

export {
  FILTER_FIELD_OPTIONS,
  FILTER_OPERATOR_OPTIONS,
  createEmptyFilter,
} from './custom.types';
