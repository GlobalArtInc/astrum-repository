

/**
 * Docker Search Module
 *
 * Public exports for the Docker search components.
 */

// Types
export type {
  DockerSearchFilters,
  DockerResult,
  DockerTag,
  DockerDetail,
  DockerSearchRequest,
  DockerSearchResponse,
  DockerSuggestion,
  DockerSuggestResponse,
  DockerSearchState,
  DockerDetailState,
} from './docker.types';

// Main page components
export { DockerSearchPage } from './DockerSearchPage';
export type { DockerSearchPageProps } from './DockerSearchPage';

export { DockerDetailPage } from './DockerDetailPage';
export type { DockerDetailPageProps } from './DockerDetailPage';

// Sub-components (for customization)
export { DockerSearchInput } from './DockerSearchInput';
export type { DockerSearchInputProps } from './DockerSearchInput';

export { DockerSearchFilters } from './DockerSearchFilters';
export type { DockerSearchFiltersProps, FilterValues } from './DockerSearchFilters';

export { DockerSearchResults } from './DockerSearchResults';
export type { DockerSearchResultsProps } from './DockerSearchResults';

export { DockerResultRow } from './DockerResultRow';
export type { DockerResultRowProps } from './DockerResultRow';

export { DockerSearchPagination } from './DockerSearchPagination';
export type { DockerSearchPaginationProps } from './DockerSearchPagination';

// Hook
export { useDockerSearch } from './useDockerSearch';
export type { UseDockerSearchReturn, DockerSearchParams } from './useDockerSearch';

// Mock data (for development only - remove when backend is ready)
export {
  mockResults,
  mockSuggestions,
  mockTags,
  mockDetail,
  mockSearchApi,
  mockSuggestApi,
  mockDetailApi,
} from './mockData';


