

/**
 * Audit event category for UI filtering and display.
 */
export type AuditCategory = 'security' | 'repository' | 'configuration' | 'protection';

/**
 * Audit event as returned from the backend REST API.
 */
export interface AuditEvent {
  id: number;
  domain: string;
  type: string;
  context: string;
  timestamp: string;  // ISO 8601
  initiator: string | null;
  nodeId: string;
  attributes: Record<string, any>;
}

/**
 * Paginated audit log response.
 */
export interface AuditLogResponse {
  items: AuditEvent[];
  pagination: {
    totalItems: number;
    totalPages: number;
    currentPage: number;
    itemsPerPage: number;
  };
}

/**
 * Filters for querying audit log.
 */
export interface AuditFilters {
  categories: AuditCategory[];
  domains: string[];
  eventTypes: string[];
  dateRange: 'last-24-hours' | 'last-7-days' | 'last-30-days' | 'last-90-days' | 'custom';
  customStartDate?: string;
  customEndDate?: string;
  initiator: string;
  initiators: string[];
  searchQuery: string;
  repositoryName?: string;
  repositoryType?: string;
}

/**
 * Enhanced audit event with computed UI fields.
 */
export interface AuditEventDisplay extends AuditEvent {
  category: AuditCategory;
  eventLabel: string;        // "User Created"
  summary: string;           // "User 'bob' created by admin"
  entityType: string;        // "User"
  entityName: string;        // "bob"
}
