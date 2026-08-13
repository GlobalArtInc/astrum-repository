

// Public API for the Tags feature (Preview UI). Per frontend/architecture.md,
// only the page components and their data types are exported here; hooks,
// state machines, and API clients are internal implementation details and are
// imported directly by their consumers (and tests).

// Page components (Layer 3)
export { TagsPageRadix } from './TagsPageRadix';
export { default } from './TagsPageRadix';
export { TagDetailPage } from './TagDetailPage';

// Public data types
export type {
  TagDetail as TagDetailType,
  TagWithCount,
  TagSortField,
  SortDirection,
  TagsFilters,
  TaggedComponent,
} from './tags.types';
