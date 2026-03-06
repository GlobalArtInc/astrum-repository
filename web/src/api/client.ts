import { readCookie } from '../i18n/language';
import { ApiError, errorFromBody } from './errors';

export const REST = '/service/rest';
export const V1 = `${REST}/v1`;
export const INTERNAL = `${REST}/internal`;
export const CSRF_COOKIE = 'astrum_csrf';
export const CSRF_HEADER = 'X-Astrum-CSRF';
export const UI_HEADER = 'X-Astrum-UI';

const SAFE_METHODS = new Set(['GET', 'HEAD', 'OPTIONS']);
const unauthorizedListeners = new Set<() => void>();

export type Body = unknown;

export interface RequestOptions {
  method?: string;
  body?: Body;
  headers?: HeadersInit;
  signal?: AbortSignal;
  quietUnauthorized?: boolean;
}

export function onUnauthorized(listener: () => void): () => void {
  unauthorizedListeners.add(listener);
  return () => {
    unauthorizedListeners.delete(listener);
  };
}

function encodeBody(body: Body, headers: Headers): BodyInit | undefined {
  if (body === undefined) {
    return undefined;
  }
  if (
    body instanceof FormData ||
    body instanceof Blob ||
    body instanceof URLSearchParams ||
    typeof body === 'string'
  ) {
    return body;
  }
  headers.set('Content-Type', 'application/json');
  return JSON.stringify(body);
}

async function readBody(response: Response): Promise<unknown> {
  const type = response.headers.get('content-type') ?? '';
  const text = await response.text();
  if (type.includes('json') && text !== '') {
    try {
      return JSON.parse(text);
    } catch {
      return text;
    }
  }
  return text;
}

export async function request<T>(path: string, options: RequestOptions = {}): Promise<T> {
  const method = (options.method ?? 'GET').toUpperCase();
  const headers = new Headers(options.headers);
  headers.set(UI_HEADER, 'true');
  if (!headers.has('Accept')) {
    headers.set('Accept', 'application/json');
  }
  if (!SAFE_METHODS.has(method)) {
    const csrf = readCookie(CSRF_COOKIE);
    if (csrf) {
      headers.set(CSRF_HEADER, csrf);
    }
  }
  const body = encodeBody(options.body, headers);
  let response: Response;
  try {
    response = await fetch(path, {
      method,
      headers,
      body,
      credentials: 'same-origin',
      signal: options.signal,
    });
  } catch (error) {
    if (error instanceof DOMException && error.name === 'AbortError') {
      throw error;
    }
    throw new ApiError(0, error instanceof Error ? error.message : String(error));
  }
  if (response.status === 401 && !options.quietUnauthorized) {
    for (const listener of unauthorizedListeners) {
      listener();
    }
  }
  if (!response.ok) {
    throw errorFromBody(response.status, response.statusText, await readBody(response));
  }
  if (response.status === 204 || method === 'HEAD') {
    return undefined as T;
  }
  return (await readBody(response)) as T;
}

export const api = {
  get: <T>(path: string, options?: Omit<RequestOptions, 'method' | 'body'>) =>
    request<T>(path, { ...options, method: 'GET' }),
  post: <T>(path: string, body?: Body, options?: Omit<RequestOptions, 'method' | 'body'>) =>
    request<T>(path, { ...options, method: 'POST', body }),
  put: <T>(path: string, body?: Body, options?: Omit<RequestOptions, 'method' | 'body'>) =>
    request<T>(path, { ...options, method: 'PUT', body }),
  delete: <T = void>(path: string, options?: Omit<RequestOptions, 'method'>) =>
    request<T>(path, { ...options, method: 'DELETE' }),
};

export function path(strings: TemplateStringsArray, ...values: Array<string | number>): string {
  return strings.reduce(
    (built, part, index) =>
      built + part + (index < values.length ? encodeURIComponent(String(values[index])) : ''),
    '',
  );
}

export function withQuery(
  base: string,
  query: Record<string, string | number | boolean | null | undefined>,
): string {
  const params = new URLSearchParams();
  for (const [key, value] of Object.entries(query)) {
    if (value !== undefined && value !== null && value !== '') {
      params.set(key, String(value));
    }
  }
  const text = params.toString();
  return text ? `${base}${base.includes('?') ? '&' : '?'}${text}` : base;
}
