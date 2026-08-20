

/**
 * NuGet Search Module
 * 
 * Agent 2 - NuGet Search Implementation
 * 
 * Exports:
 * - NuGetSearchPage - Main search page
 * - NuGetDetailPage - Package detail view
 * - NuGetSearchFilters - Filter component
 * - NuGetSearchResults - Results table
 * - useNuGetSearch - Search state hook
 */

// Pages
export { NuGetSearchPage } from './NuGetSearchPage';
export { NuGetDetailPage } from './NuGetDetailPage';

// Components
export { NuGetSearchFilters } from './NuGetSearchFilters';
export { NuGetSearchResults } from './NuGetSearchResults';

// Hooks
export { useNuGetSearch } from './useNuGetSearch';

// Types
export type {
  NuGetResult,
  NuGetDetail,
  NuGetVersion,
  NuGetSearchFilters as NuGetSearchFilterValues,
  NuGetSearchState,
  NuGetSearchResponse,
} from './nuget.types';

