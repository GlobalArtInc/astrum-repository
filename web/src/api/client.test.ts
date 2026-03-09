import { afterEach, describe, expect, it, vi } from 'vitest';
import { jsonResponse } from '../test/http';
import { api, onUnauthorized, path, request, withQuery } from './client';
import { ApiError } from './errors';
import { matchesApiPrefix } from './query';

function mockFetch(response: Response | Error) {
  const fetch = vi.fn(async () => {
    if (response instanceof Error) {
      throw response;
    }
    return response;
  });
  vi.stubGlobal('fetch', fetch);
  return fetch;
}

function sent(fetch: ReturnType<typeof mockFetch>) {
  const [url, init] = fetch.mock.calls[0] as unknown as [string, RequestInit];
  return { url, init, headers: new Headers(init.headers) };
}

afterEach(() => {
  vi.unstubAllGlobals();
});

describe('the API client', () => {
  it('marks requests as coming from the UI and reads JSON', async () => {
    const fetch = mockFetch(jsonResponse({ ok: 1 }));
    expect(await api.get('/service/rest/v1/status')).toEqual({ ok: 1 });
    const { url, init, headers } = sent(fetch);
    expect(url).toBe('/service/rest/v1/status');
    expect(init.method).toBe('GET');
    expect(init.credentials).toBe('same-origin');
    expect(headers.get('X-Astrum-UI')).toBe('true');
    expect(headers.get('X-Astrum-CSRF')).toBeNull();
  });

  it('sends the anti-forgery token and JSON on unsafe methods', async () => {
    document.cookie = 'astrum_csrf=token-1';
    const fetch = mockFetch(new Response(null, { status: 204 }));
    expect(await api.post('/x', { name: 'a' })).toBeUndefined();
    const { init, headers } = sent(fetch);
    expect(headers.get('X-Astrum-CSRF')).toBe('token-1');
    expect(headers.get('Content-Type')).toBe('application/json');
    expect(init.body).toBe('{"name":"a"}');
  });

  it('passes form data untouched', async () => {
    const fetch = mockFetch(new Response('', { status: 200 }));
    const form = new FormData();
    form.set('file', 'x');
    await api.post('/upload', form);
    const { init, headers } = sent(fetch);
    expect(init.body).toBe(form);
    expect(headers.has('Content-Type')).toBe(false);
  });

  it('reads field errors of a 400', async () => {
    mockFetch(
      jsonResponse(
        [
          { id: 'PARAMETER name', message: 'must not be empty' },
          { id: '*', message: 'broken' },
        ],
        400,
      ),
    );
    const error = await request('/x').catch((failure: unknown) => failure);
    expect(error).toBeInstanceOf(ApiError);
    expect((error as ApiError).status).toBe(400);
    expect((error as ApiError).fieldErrors).toHaveLength(2);
    expect((error as ApiError).message).toBe('must not be empty; broken');
  });

  it('reads the message of a 404', async () => {
    mockFetch(jsonResponse({ id: '*', message: 'no such repository' }, 404));
    const error = (await request('/x').catch((failure: unknown) => failure)) as ApiError;
    expect(error.status).toBe(404);
    expect(error.message).toBe('no such repository');
  });

  it('tells listeners about a 401 unless asked to stay quiet', async () => {
    const listener = vi.fn();
    const stop = onUnauthorized(listener);
    mockFetch(jsonResponse({ id: '*', message: 'authentication is required' }, 401));
    await request('/x').catch(() => undefined);
    expect(listener).toHaveBeenCalledTimes(1);
    mockFetch(jsonResponse({ id: '*', message: 'authentication is required' }, 401));
    await request('/x', { quietUnauthorized: true }).catch(() => undefined);
    expect(listener).toHaveBeenCalledTimes(1);
    stop();
  });

  it('turns network failures into status 0', async () => {
    mockFetch(new TypeError('Failed to fetch'));
    const error = (await request('/x').catch((failure: unknown) => failure)) as ApiError;
    expect(error.status).toBe(0);
    expect(error.isNetwork).toBe(true);
  });

  it('builds paths and queries', () => {
    expect(path`/service/rest/v1/repositories/${'a b/c'}`).toBe('/service/rest/v1/repositories/a%20b%2Fc');
    expect(withQuery('/x', { q: 'react', format: undefined, page: 2, empty: '' })).toBe('/x?q=react&page=2');
    expect(withQuery('/x?a=1', { b: true })).toBe('/x?a=1&b=true');
    expect(withQuery('/x', {})).toBe('/x');
  });

  it('matches query keys by path prefix', () => {
    expect(
      matchesApiPrefix(['api', '/service/rest/v1/repositories?x'], '/service/rest/v1/repositories'),
    ).toBe(true);
    expect(matchesApiPrefix(['api', '/service/rest/v1/blobstores'], '/service/rest/v1/repositories')).toBe(
      false,
    );
    expect(matchesApiPrefix(['ui', '/service/rest/v1/repositories'], '/service/rest/v1/repositories')).toBe(
      false,
    );
  });
});
