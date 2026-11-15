export interface FieldError {
  id: string;
  message: string;
}

interface ApiErrorOptions {
  id?: string;
  code?: string;
  retryAfter?: number;
  fieldErrors?: FieldError[];
}

interface CodedError extends FieldError {
  code?: string;
}

export class ApiError extends Error {
  readonly status: number;
  readonly id: string | undefined;
  readonly code: string | undefined;
  readonly retryAfter: number | undefined;
  readonly fieldErrors: FieldError[];

  constructor(status: number, message: string, options: ApiErrorOptions = {}) {
    super(message);
    this.name = 'ApiError';
    this.status = status;
    this.id = options.id;
    this.code = options.code;
    this.retryAfter = options.retryAfter;
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

export function parseRetryAfter(value: string | null): number | undefined {
  const text = value?.trim() ?? '';
  if (!/^\d+$/.test(text)) {
    return undefined;
  }
  const seconds = Number(text);
  return seconds > 0 ? seconds : undefined;
}

export function errorFromBody(
  status: number,
  statusText: string,
  body: unknown,
  retryAfter?: number,
): ApiError {
  if (Array.isArray(body)) {
    const fieldErrors = body.filter(isFieldError);
    const message = fieldErrors.map((error) => error.message).join('; ') || statusText;
    return new ApiError(status, message, { fieldErrors, retryAfter });
  }
  if (isFieldError(body)) {
    const code = (body as CodedError).code;
    return new ApiError(status, body.message, {
      id: body.id,
      code: typeof code === 'string' ? code : undefined,
      retryAfter,
    });
  }
  if (typeof body === 'string' && body.trim() !== '') {
    return new ApiError(status, body.trim(), { retryAfter });
  }
  return new ApiError(status, statusText || `HTTP ${status}`, { retryAfter });
}

export function fieldName(id: string): string {
  return id.replace(/^PARAMETER\s+/, '').trim();
}
