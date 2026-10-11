import { parseApiError } from '../api/error-handler';

/**
 * Check if validation errors object has any errors
 */
export function hasValidationErrors(errors: Record<string, string | null>): boolean {
  return Object.values(errors).some((error) => error !== null && error !== undefined);
}

/**
 * Extract error message from various error formats (Axios, Error, string, etc.)
 */
export function extractErrorMessage(error: unknown): string {
  if (typeof error === 'string') {
    return error;
  }

  // A failed request: the server's message (validation errors, a server error's text) rather than Axios's
  // "Request failed with status code …"
  if (error instanceof Error && (error as Error & { isAxiosError?: boolean }).isAxiosError
      && (error as Error & { response?: unknown }).response) {
    const { message } = parseApiError(error);
    if (message) {
      return message;
    }
  }

  if (error instanceof Error) {
    return error.message;
  }

  if (typeof error === 'object' && error !== null) {
    const err = error as Record<string, unknown>;

    // Axios error format
    if (err.response && typeof err.response === 'object') {
      const response = err.response as Record<string, unknown>;
      if (response.data && typeof response.data === 'object') {
        const data = response.data as Record<string, unknown>;
        if (typeof data.message === 'string') {
          return data.message;
        }
      }
    }

    // Standard message property
    if (typeof err.message === 'string') {
      return err.message;
    }
  }

  return 'An unexpected error occurred';
}

/**
 * Convert field name to path array for Ramda functions
 * Supports dot notation: 'user.name' -> ['user', 'name']
 */
export function toPathArray(name: string): string[] {
  return name.split('.');
}
