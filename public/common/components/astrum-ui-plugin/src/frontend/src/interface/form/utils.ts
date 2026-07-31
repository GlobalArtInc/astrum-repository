

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
