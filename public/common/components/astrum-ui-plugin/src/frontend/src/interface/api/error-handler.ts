

/**
 * REST API Error Handler
 *
 * Parses REST API error responses and provides detailed, actionable error messages.
 * Replaces generic "Unknown error" messages with specific, helpful information.
 *
 * Usage:
 *   try {
 *     await Axios.post('/api/endpoint', data);
 *   } catch (err) {
 *     const apiError = parseApiError(err);
 *     setError(apiError.message);
 *     setFieldErrors(apiError.fieldErrors);
 *   }
 */

import { AxiosError } from 'axios';
import { ApiError, FieldError, RestErrorResponse } from './types';
import { API_ERROR_STRINGS } from './errorStrings';

// JAX-RS ConstraintViolationExceptionMapper prefixes the `id` field of each
// validation error with the leaf bean's simple name (`HelperBean.`) followed by
// the property path (often `attributes.<field>` for repository config). Neither
// segment is part of the frontend form's field naming, so we strip both before
// surfacing field-level errors. (ASTRUM-51599)
const HELPER_BEAN_PREFIX = 'HelperBean.';
const ATTRIBUTES_PREFIX = 'attributes.';

const CONSTRAINT_MESSAGES: Record<string, string> = {
  'org.globalart.astrum.validation.constraint.name': API_ERROR_STRINGS.CONSTRAINTS.NAME,
  'org.globalart.astrum.validation.constraint.case': API_ERROR_STRINGS.CONSTRAINTS.CASE,
  'org.globalart.astrum.validation.constraint.portnumber': API_ERROR_STRINGS.CONSTRAINTS.PORT_NUMBER,
  'org.globalart.astrum.validation.constraint.hostname': API_ERROR_STRINGS.CONSTRAINTS.HOSTNAME,
  'org.globalart.astrum.validation.constraint.notnull': API_ERROR_STRINGS.CONSTRAINTS.NOT_NULL,
};

function resolveConstraintMessage(message: string): string {
  const match = message.match(/^\{([^}]+)\}$/);
  if (match) {
    return CONSTRAINT_MESSAGES[match[1]] ?? message;
  }
  return message;
}

// WebApplicationMessageException callers on the backend commonly pass a JSON string
// literal as the message (e.g. `"\"Repository not found\""`), so the parsed `message`
// field itself is wrapped in a redundant pair of literal quote characters. Strip one
// layer so toasts show the plain text instead of `"Repository not found"`.
function unwrapQuotedMessage(message: string | undefined): string | undefined {
  if (message && message.length >= 2 && message.startsWith('"') && message.endsWith('"')) {
    return message.slice(1, -1);
  }
  return message;
}

/**
 * Parse any error into a structured ApiError
 */
export function parseApiError(error: unknown): ApiError {
  // Handle Axios errors
  if (isAxiosError(error)) {
    return parseAxiosError(error);
  }

  // Handle Error objects
  if (error instanceof Error) {
    return {
      status: 0,
      message: error.message || API_ERROR_STRINGS.UNEXPECTED,
      originalError: error,
    };
  }

  // Handle string errors
  if (typeof error === 'string') {
    return {
      status: 0,
      message: error,
    };
  }

  // Unknown error type
  return {
    status: 0,
    message: API_ERROR_STRINGS.UNEXPECTED,
    originalError: error,
  };
}

/**
 * Parse Axios error response
 */
function parseAxiosError(error: AxiosError<RestErrorResponse>): ApiError {
  const status = error.response?.status || 0;
  const data = error.response?.data;

  // Extract request ID from headers if available
  const requestId = error.response?.headers?.['x-request-id'] as string | undefined;

  // No response (network error, timeout, etc.)
  if (!error.response) {
    if (error.code === 'ECONNABORTED') {
      return {
        status: 0,
        message: API_ERROR_STRINGS.TIMEOUT,
        code: 'TIMEOUT',
        requestId,
        originalError: error,
      };
    }
    return {
      status: 0,
      message: API_ERROR_STRINGS.NETWORK,
      code: 'NETWORK_ERROR',
      requestId,
      originalError: error,
    };
  }

  // Parse response based on status code
  switch (status) {
    case 400:
      return parseBadRequest(status, data, requestId, error);

    case 401:
      return {
        status: 401,
        message: API_ERROR_STRINGS.SESSION_EXPIRED,
        code: 'UNAUTHORIZED',
        requestId,
        originalError: error,
      };

    case 403:
      return {
        status: 403,
        message: unwrapQuotedMessage(data?.message) || API_ERROR_STRINGS.FORBIDDEN,
        code: 'FORBIDDEN',
        requestId,
        originalError: error,
      };

    case 404:
      return {
        status: 404,
        message: unwrapQuotedMessage(data?.message) || API_ERROR_STRINGS.NOT_FOUND,
        code: 'NOT_FOUND',
        requestId,
        originalError: error,
      };

    case 405:
      return {
        status: 405,
        message: unwrapQuotedMessage(data?.message) || API_ERROR_STRINGS.METHOD_NOT_ALLOWED,
        code: 'METHOD_NOT_ALLOWED',
        requestId,
        originalError: error,
      };

    case 409:
      return {
        status: 409,
        message: unwrapQuotedMessage(data?.message) || API_ERROR_STRINGS.CONFLICT,
        code: 'CONFLICT',
        requestId,
        originalError: error,
      };

    case 422:
      return parseBadRequest(status, data, requestId, error);

    case 500:
    case 502:
    case 503:
    case 504: {
      // Extract backend error message if available (can be string or object with message)
      let backendMessage: string | undefined;
      if (typeof data === 'string' && data.trim()) {
        backendMessage = data;
      } else if (data?.message) {
        backendMessage = unwrapQuotedMessage(data.message);
      }
      
      const message = backendMessage 
        ? backendMessage 
        : API_ERROR_STRINGS.SERVER_ERROR(status);
      
      return {
        status,
        message: requestId ? API_ERROR_STRINGS.WITH_REFERENCE(message, requestId) : message,
        code: 'SERVER_ERROR',
        requestId,
        originalError: error,
      };
    }

    default:
      return {
        status,
        message: unwrapQuotedMessage(data?.message) || API_ERROR_STRINGS.REQUEST_FAILED,
        requestId,
        originalError: error,
      };
  }
}

/**
 * Parse 400 Bad Request errors (validation errors)
 */
function parseBadRequest(
  httpStatus: number,
  data: RestErrorResponse | undefined | Array<{ id?: string; message?: string }>,
  requestId: string | undefined,
  error: AxiosError
): ApiError {
  const fieldErrors: FieldError[] = [];
  const messages: string[] = [];

  // Handle array of errors directly (Astrum validation format)
  // Example: [{"id":"PARAMETER path","message":"Path is required"}]
  // Example: [{"id":"*","message":"A file blob store must have a unique path..."}]
  // Example: [{"id":"HelperBean.attributes.docker.httpPort","message":"Port must be unique..."}]
  if (Array.isArray(data)) {
    for (const err of data) {
      if (err.message) {
        const resolvedMessage = resolveConstraintMessage(err.message);
        let fieldName = err.id && err.id !== '*'
          ? err.id.replace('PARAMETER ', '')
          : null;
        // Anchor the HelperBean. and attributes. strips to the start of the
        // string so an `id` that legitimately contains those tokens deeper in
        // the property path (e.g. a field literally named myHelperBean.foo)
        // isn't mangled mid-key.
        if (fieldName?.startsWith(HELPER_BEAN_PREFIX)) {
          fieldName = fieldName.slice(HELPER_BEAN_PREFIX.length);
        }
        if (fieldName?.startsWith(ATTRIBUTES_PREFIX)) {
          fieldName = fieldName.slice(ATTRIBUTES_PREFIX.length);
        }
        messages.push(resolvedMessage);
        if (fieldName) {
          fieldErrors.push({
            field: fieldName,
            message: resolvedMessage,
          });
        }
      }
    }
  }
  // Parse nested errors array (Hibernate Validator format)
  else if (data?.errors && Array.isArray(data.errors)) {
    for (const err of data.errors) {
      if (err.field || err.id) {
        const resolvedMessage = resolveConstraintMessage(err.message || API_ERROR_STRINGS.INVALID_VALUE);
        fieldErrors.push({
          field: err.field || err.id || 'unknown',
          message: resolvedMessage,
        });
        messages.push(resolvedMessage);
      }
    }
  }
  // Try to extract field from message if no explicit errors
  else if (data?.message) {
    messages.push(data.message);
    const fieldMatch = data.message.match(/(?:field|property|parameter)\s+['"]?(\w+)['"]?/i);
    if (fieldMatch) {
      fieldErrors.push({
        field: fieldMatch[1],
        message: data.message,
      });
    }
  }

  // Build user-friendly message - prefer the actual error messages from API
  let message: string;
  if (messages.length > 0) {
    message = messages.join('. ');
  } else if (fieldErrors.length > 0) {
    if (fieldErrors.length === 1) {
      message = API_ERROR_STRINGS.VALIDATION_ERROR(fieldErrors[0].message);
    } else {
      message = API_ERROR_STRINGS.VALIDATION_ERRORS(fieldErrors.length);
    }
  } else {
    message = API_ERROR_STRINGS.INVALID_REQUEST;
  }

  return {
    status: httpStatus,
    message,
    code: 'VALIDATION_ERROR',
    fieldErrors: fieldErrors.length > 0 ? fieldErrors : undefined,
    requestId,
    originalError: error,
  };
}

/**
 * Type guard for Axios errors
 */
function isAxiosError(error: unknown): error is AxiosError<RestErrorResponse> {
  return (
    typeof error === 'object' &&
    error !== null &&
    'isAxiosError' in error &&
    (error as AxiosError).isAxiosError === true
  );
}

/**
 * Get user-friendly message for display
 */
export function getErrorMessage(error: ApiError): string {
  return error.message;
}

/**
 * Get field-specific error message
 */
export function getFieldError(error: ApiError, fieldName: string): string | undefined {
  return error.fieldErrors?.find((fe) => fe.field === fieldName)?.message;
}

/**
 * Check if error has field-level validation errors
 */
export function hasFieldErrors(error: ApiError): boolean {
  return (error.fieldErrors?.length || 0) > 0;
}

/**
 * Convert field errors to a map for easy lookup
 */
export function fieldErrorsToMap(error: ApiError): Record<string, string> {
  const map: Record<string, string> = {};
  if (error.fieldErrors) {
    for (const fe of error.fieldErrors) {
      map[fe.field] = fe.message;
    }
  }
  return map;
}

/**
 * Check if error is a specific type
 */
export function isAuthError(error: ApiError): boolean {
  return error.status === 401;
}

export function isPermissionError(error: ApiError): boolean {
  return error.status === 403;
}

export function isNotFoundError(error: ApiError): boolean {
  return error.status === 404;
}

export function isConflictError(error: ApiError): boolean {
  return error.status === 409;
}

export function isValidationError(error: ApiError): boolean {
  return error.status === 400 || error.status === 422;
}

export function isServerError(error: ApiError): boolean {
  return error.status >= 500;
}

export function isNetworkError(error: ApiError): boolean {
  return error.status === 0;
}
