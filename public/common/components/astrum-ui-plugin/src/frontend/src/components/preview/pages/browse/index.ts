

/**
 * Browse Module - Public API
 *
 * This is the main entry point for the Browse module.
 * Import types, API functions, and shared components from here.
 *
 * @example
 * import { BrowseNode, fetchBrowseNodes, NodeIcon } from '../browse';
 */

// =============================================================================
// TYPES
// =============================================================================
export * from './browse.types';

// =============================================================================
// API
// =============================================================================
export * from './browse.api';

// =============================================================================
// SHARED COMPONENTS
// =============================================================================
export { NodeIcon } from './shared/NodeIcon';
export { FormatBadge } from './shared/FormatBadge';
export { StatusIndicator } from './shared/StatusIndicator';

// =============================================================================
// IN-REPOSITORY SEARCH
// =============================================================================
export { InRepositorySearch } from './InRepositorySearch';
export { SearchResultItem } from './SearchResultItem';

// =============================================================================
// MAIN COMPONENTS (to be added by agents)
// =============================================================================
// export { RepositoryList } from './repository-list/RepositoryList';
// export { BrowseTree } from './tree/BrowseTree';
// export { DetailPanel } from './detail/DetailPanel';
// export { DeleteDialog, CopyUrlButton } from './actions';
// export { default as BrowsePage } from './BrowsePage';
