

/**
 * Common API Types
 *
 * Shared type definitions for REST API interactions.
 * Part of the ExtDirect migration - standardizes error handling.
 */

/**
 * Structured API error with detailed information
 */
export interface ApiError {
  /** HTTP status code */
  status: number;

  /** Human-readable error message */
  message: string;

  /** Error code for programmatic handling */
  code?: string;

  /** Field-level validation errors */
  fieldErrors?: FieldError[];

  /** Request ID for debugging/support */
  requestId?: string;

  /** Original error for debugging */
  originalError?: unknown;
}

/**
 * Field-level validation error
 */
export interface FieldError {
  /** Field name (e.g., "name", "actions") */
  field: string;

  /** Error message for this field */
  message: string;

  /** Rejected value (if available) */
  rejectedValue?: unknown;
}

/**
 * Standard REST API error response format
 * Based on actual Astrum REST API responses
 */
export interface RestErrorResponse {
  /** Unique error ID */
  id?: string;

  /** Error message */
  message?: string;

  /** Nested validation errors (Hibernate Validator format) */
  errors?: Array<{
    id?: string;
    message?: string;
    field?: string;
  }>;
}

/**
 * Pagination parameters for list endpoints
 */
export interface PaginationParams {
  page?: number;
  size?: number;
  sort?: string;
  direction?: 'asc' | 'desc';
}

/**
 * Paginated response wrapper
 */
export interface PaginatedResponse<T> {
  items: T[];
  totalCount: number;
  page: number;
  pageSize: number;
  hasMore: boolean;
}

/**
 * Common API response states for hooks
 */
export interface ApiState<T> {
  data: T | null;
  loading: boolean;
  error: ApiError | null;
}

/**
 * HTTP methods supported
 */
export type HttpMethod = 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH';

/**
 * Request configuration
 */
export interface RequestConfig {
  /** Request timeout in milliseconds */
  timeout?: number;

  /** Additional headers */
  headers?: Record<string, string>;

  /** Query parameters */
  params?: Record<string, string | number | boolean>;

  /** AbortSignal to cancel the request (e.g. on unmount or strict-mode remount) */
  signal?: AbortSignal;

  /** Skip error handling (for custom handling) */
  skipErrorHandling?: boolean;
}
