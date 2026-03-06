export interface FieldError {
  id: string;
  message: string;
}

export class ApiError extends Error {
  readonly status: number;
  readonly id: string | undefined;
  readonly fieldErrors: FieldError[];

  constructor(status: number, message: string, options: { id?: string; fieldErrors?: FieldError[] } = {}) {
    super(message);
    this.name = 'ApiError';
    this.status = status;
    this.id = options.id;
    this.fieldErrors = options.fieldErrors ?? [];
  }

  get isNetwork(): boolean {
    return this.status === 0;
  }
}

function isFieldError(value: unknown): value is FieldError {
  return (
    typeof value === 'object' &&
    value !== null &&
    typeof (value as FieldError).id === 'string' &&
    typeof (value as FieldError).message === 'string'
  );
}

export function errorFromBody(status: number, statusText: string, body: unknown): ApiError {
  if (Array.isArray(body)) {
    const fieldErrors = body.filter(isFieldError);
    const message = fieldErrors.map((error) => error.message).join('; ') || statusText;
    return new ApiError(status, message, { fieldErrors });
  }
  if (isFieldError(body)) {
    return new ApiError(status, body.message, { id: body.id });
  }
  if (typeof body === 'string' && body.trim() !== '') {
    return new ApiError(status, body.trim());
  }
  return new ApiError(status, statusText || `HTTP ${status}`);
}

export function fieldName(id: string): string {
  return id.replace(/^PARAMETER\s+/, '').trim();
}
