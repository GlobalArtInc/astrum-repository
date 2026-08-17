

/**
 * GA Search Core Module
 * 
 * This module contains the authoritative contracts for GA-level search.
 * 
 * IMPORTANT: GA search exists ONLY in Preview UI.
 * Default UI (#browse/search/*) remains ExtJS-based and unchanged.
 * 
 * OWNERSHIP: Agent 0 (Tech Lead) owns this module.
 * Other agents import from here but do not modify.
 * 
 * Exports:
 * - Domain types (search.types.ts)
 * - API contracts (search.api.ts)
 * - Route definitions (search.routes.ts)
 * - UI mode detection (search.flags.ts)
 */

// Domain Types
export type {
  GAFormat,
  GAResult,
  GADetail,
  GARepository,
  GAVersion,
  GAAsset,
  GASearchRequest,
  GASearchResponse,
  GASuggestRequest,
  GASuggestResponse,
  GASuggestion,
  GADetailRequest,
  GAVersionAssetsRequest,
  GADetailTab,
  GASearchState,
  GADetailState,
  ComponentVersionSort,
  ComponentVersionsRequest,
  ComponentVersionsPage,
  ComponentVersionDetailRequest,
  ComponentVersionDetail,
} from './search.types';

// API Contracts
export {
  GA_SEARCH_API,
  buildSearchUrl,
  buildSuggestUrl,
  buildDetailUrl,
  buildVersionAssetsUrl,
  gaSearchApi,
  GASearchError,
} from './search.api';

// Route Definitions
export {
  GA_SEARCH_ROUTE_NAMES,
  DEFAULT_SEARCH_ROUTE_NAMES,
  GA_SEARCH_URLS,
  GA_SEARCH_PARAMS,
  PREVIEW_BASE_URL,
  buildSearchRoute,
  buildDetailRoute,
  buildDefaultSearchRoute,
  parseGaId,
  parseSearchParams,
  TAB_ROUTE_MAP,
  getTabFromRoute,
} from './search.routes';

// UI Mode Detection
export {
  PREVIEW_UI_PREFIX,
  isPreviewUI,
  isGASearchContext,
  toPreviewSearchUrl,
  toDefaultSearchUrl,
  GA_SEARCH_VISIBILITY,
  GA_SEARCH_FEATURE_FLAGS,
} from './search.flags';

// Search API (connects to existing /service/rest/v1/search)
export {
  searchMavenGA,
  searchComponents,
} from './searchApi';

// Component Versions API (paginated versions)
export { fetchComponentVersions } from './componentVersionsApi';

// Component Version Detail API (single version, Files tab)
export { fetchComponentVersionDetail, MAX_DETAIL_PAGES } from './componentVersionDetailApi';

// Shared Search Utilities (for all formats)
export {
  searchNpm,
  searchNuGet,
  searchDocker,
  searchGeneric,
  getFormatLabel,
  getFormatColor,
  buildDisplayName,
  buildComponentId,
  FORMAT_DISPLAY,
} from './searchUtils';

export type {
  RawSearchItem,
  RawAsset,
  RawSearchResponse,
  BaseSearchParams,
  NpmSearchParams,
  NuGetSearchParams,
  DockerSearchParams,
} from './searchUtils';

// UI Components (Agent 0)
export { SwitchToDefaultUI } from './SwitchToDefaultUI';
export { SearchBar } from './SearchBar';
export type { SearchBarProps, SearchFormat } from './SearchBar';
