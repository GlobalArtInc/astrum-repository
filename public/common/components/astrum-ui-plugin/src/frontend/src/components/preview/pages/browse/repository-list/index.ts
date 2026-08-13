

/**
 * Repository List Module
 *
 * Browse functionality for viewing repositories in a table with sorting,
 * filtering, and IQ Server integration.
 */

// Main component
export { RepositoryList, default } from './RepositoryList';
export { RepositoryListTable } from './RepositoryListTable';
export { BrowseSidebar } from './BrowseSidebar';

// Sub-components
export { RepositoryStatusBadge } from './RepositoryStatusBadge';
export { HealthCheckCell } from '../../../shared/security/HealthCheckCell';
export { FirewallCell as IqPolicyViolationsCell } from '../../../shared/security/FirewallCell';

// Hooks
export {
  useRepositoryList,
  isIqServerEnabled,
  canUpdateHealthCheck,
  canReadFirewallStatus,
} from './useRepositoryList';

// Server-side filtering hook (enterprise-scalable)
export {
  useRepositoryListServer,
  arrayToFilterString,
  filterStringToArray,
} from './useRepositoryListServer';

// Types
export type {
  Repository,
  RepositoryStatus,
  RepositoryType,
  RepositoryListProps,
  RepositoryListState,
  RepositoryStatusBadgeProps,
  HealthCheckStatus,
  FirewallStatus,
  SortableField,
  SortDirection,
  SortConfig,
  RepositoryListStrings,
} from './repository-list.types';

