import { screen } from '@testing-library/react';
import SwaggerUI from 'swagger-ui-dist/swagger-ui-es-bundle.js';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { renderWithApp } from '../../test/render';
import ApiPage from './ApiPage';
import { withSession } from './model';

vi.mock('swagger-ui-dist/swagger-ui-es-bundle.js', () => ({ default: vi.fn() }));

afterEach(() => {
  vi.mocked(SwaggerUI).mockClear();
  document.cookie = 'astrum_csrf=; expires=Thu, 01 Jan 1970 00:00:00 GMT; path=/';
});

describe('the API page', () => {
  it('mounts Swagger UI on the API description without calling outside', () => {
    renderWithApp(null, { path: '/api', routes: [{ path: '/api', element: <ApiPage /> }] });
    expect(screen.getByRole('heading', { level: 1, name: 'API' })).toBeInTheDocument();
    const region = screen.getByRole('region', { name: 'API' });
    expect(SwaggerUI).toHaveBeenCalledTimes(1);
    const options = vi.mocked(SwaggerUI).mock.calls[0]?.[0];
    expect(options?.domNode).toBe(region);
    expect({ ...options, domNode: undefined }).toMatchObject({
      url: '/service/rest/swagger.json',
      deepLinking: true,
      validatorUrl: null,
      requestInterceptor: withSession,
    });
  });

  it('sends the session headers with the requests it makes', () => {
    document.cookie = 'astrum_csrf=token-1; path=/';
    const read = withSession({ url: '/service/rest/v1/status', method: 'GET', headers: {} });
    expect(read.headers).toEqual({ 'X-Astrum-UI': 'true' });
    const write = withSession({ url: '/service/rest/v1/tasks/x/run', method: 'post', headers: {} });
    expect(write.headers).toEqual({ 'X-Astrum-UI': 'true', 'X-Astrum-CSRF': 'token-1' });
  });
});
