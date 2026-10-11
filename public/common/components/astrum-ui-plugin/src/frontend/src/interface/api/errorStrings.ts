import localize from '../localize';

/** The texts the REST client shows for a failed request when the server sends no message of its own. */
export const API_ERROR_STRINGS = localize('react-ui', {
  CONSTRAINTS: {
    NAME: 'Only letters, digits, hyphens (-), underscores (_), and dots (.) are allowed and may not start with underscore or dot.',
    CASE: 'Case must be the correct value.',
    PORT_NUMBER: 'Port number must be within the valid range.',
    HOSTNAME: 'Hostname must be valid.',
    NOT_NULL: 'Must not be null.',
  },
  UNEXPECTED: 'An unexpected error occurred',
  TIMEOUT: 'Request timed out. Please check your connection and try again.',
  NETWORK: 'Unable to connect to server. Please check your network connection.',
  SESSION_EXPIRED: 'Your session has expired. Please log in again.',
  FORBIDDEN: 'You do not have permission to perform this action. Contact your administrator if you need access.',
  NOT_FOUND: 'The requested resource was not found.',
  METHOD_NOT_ALLOWED: 'This operation is not supported for the current resource state.',
  CONFLICT: 'This resource already exists or conflicts with another resource.',
  SERVER_ERROR: (status: number) => `Server error (${status}). Please try again later.`,
  WITH_REFERENCE: (message: string, requestId: string) => `${message} Reference: ${requestId}`,
  REQUEST_FAILED: 'Unable to complete request. Please try again.',
  INVALID_VALUE: 'Invalid value',
  VALIDATION_ERROR: (message: string) => `Validation error: ${message}`,
  VALIDATION_ERRORS: (count: number) => `Please fix ${count} validation errors`,
  INVALID_REQUEST: 'Invalid request. Please check your input.',
}, 'PREVIEW.API_ERRORS');
