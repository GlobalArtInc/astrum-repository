

/**
 * REST API Utilities
 *
 * Shared utilities for REST API interactions in the Preview UI.
 * Part of the ExtDirect to REST migration.
 *
 * Usage:
 *   import { restClient, parseApiError, ENDPOINTS } from '@/utils/api';
 *
 *   try {
 *     const data = await restClient.get<MyType>(ENDPOINTS.PRIVILEGES);
 *   } catch (err) {
 *     const apiError = parseApiError(err);
 *     console.error(apiError.message);
 *     if (apiError.fieldErrors) {
 *       // Handle field-level validation errors
 *     }
 *   }
 */

// Types
export type {
  ApiError,
  FieldError,
  RestErrorResponse,
  PaginationParams,
  PaginatedResponse,
  ApiState,
  HttpMethod,
  RequestConfig,
} from './types';

// Error handling
export {
  parseApiError,
  getErrorMessage,
  getFieldError,
  hasFieldErrors,
  fieldErrorsToMap,
  isAuthError,
  isPermissionError,
  isNotFoundError,
  isConflictError,
  isValidationError,
  isServerError,
  isNetworkError,
} from './error-handler';

// REST client
export {
  restClient,
  urlBuilder,
  API_BASE,
  API_V1,
  API_INTERNAL,
  API_INTERNAL_UI,
  ENDPOINTS,
  encodeRepositoryItemId,
  decodeRepositoryItemId,
} from './rest-client';

export { restClient as default } from './rest-client';
