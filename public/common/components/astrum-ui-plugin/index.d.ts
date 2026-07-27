

// Public TypeScript type exports for '@globalart/astrum-ui-plugin'.
// The runtime entry (target/dist/index.js) is JavaScript and cannot carry
// TypeScript-only `export type` declarations, so consumers that need types
// would otherwise have to reach into the internal source tree. This file
// re-exports the types for a stable, package-rooted import path.
export type {
  AuditCategory,
  AuditEvent,
  AuditLogResponse,
  AuditFilters,
} from './src/frontend/src/utils/audit/audit.types';
