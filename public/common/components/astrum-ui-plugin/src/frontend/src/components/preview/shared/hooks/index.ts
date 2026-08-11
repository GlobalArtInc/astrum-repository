

/**
 * Shared Hooks
 *
 * Reusable React hooks for common patterns across the application.
 */

export {
  useUnsavedChangesWarning,
  clearDirtyState,
  hasUnsavedChanges,
} from './useUnsavedChangesWarning';

export { useIsCloud } from './useIsCloud';
export { useIsClustered } from './useIsClustered';
export { useHasFirewallLicense } from './useHasFirewallLicense';
export { useUnreadStatusFailure, resetUnreadStatusFailure, STATUS_BELL_ACK_STORAGE_KEY } from './useUnreadStatusFailure';
export type { UseUnreadStatusFailureReturn } from './useUnreadStatusFailure';

export { useBlobStoreInfo } from './useBlobStoreInfo';
export type { BlobStoreInfo, UseBlobStoreInfoResult } from './useBlobStoreInfo';

export { useRepositoryMetrics } from './useRepositoryMetrics';
export type { RepositoryMetrics, UseRepositoryMetricsResult } from './useRepositoryMetrics';

export { useDebounced } from './useDebounced';
