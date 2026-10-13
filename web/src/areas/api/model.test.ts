import { describe, expect, it } from 'vitest';
import { DOC } from './fixture';
import { ALL, endpoints, filterEndpoints, groupEndpoints, requestUrl, skeleton, tagsOf } from './model';

describe('the API model', () => {
  const all = endpoints(DOC);

  it('lists every operation with its full path', () => {
    expect(all.map((endpoint) => endpoint.key)).toContain('DELETE /v1/blobstores/{name}');
    expect(all.find((endpoint) => endpoint.path === '/v1/status')?.fullPath).toBe('/service/rest/v1/status');
    expect(tagsOf(all)).toEqual(['assets', 'Blob store', 'Status']);
  });

  it('groups by tag ignoring case and sorts by path, then method, like the old list', () => {
    const groups = groupEndpoints(all);
    expect(groups.map((group) => group.tag)).toEqual(['assets', 'Blob store', 'Status']);
    expect(groups[1]?.endpoints.map((endpoint) => endpoint.key)).toEqual([
      'GET /v1/blobstores',
      'DELETE /v1/blobstores/{name}',
      'GET /v1/blobstores/{name}',
      'POST /v1/blobstores/file',
    ]);
  });

  it('filters by text, method and tag', () => {
    expect(filterEndpoints(all, { query: 'FILE', method: ALL, tag: ALL }).map((e) => e.key)).toEqual([
      'POST /v1/blobstores/file',
    ]);
    expect(filterEndpoints(all, { query: 'list', method: 'GET', tag: 'assets' }).map((e) => e.key)).toEqual([
      'GET /v1/assets',
    ]);
    expect(filterEndpoints(all, { query: '', method: 'DELETE', tag: ALL })).toHaveLength(1);
  });

  it('fills path and query parameters and reports missing ones', () => {
    const get = all.find((endpoint) => endpoint.key === 'GET /v1/blobstores/{name}');
    const assets = all.find((endpoint) => endpoint.key === 'GET /v1/assets');
    if (!get || !assets) {
      throw new Error('fixture endpoints are missing');
    }
    expect(requestUrl(get, { name: 'my store' })).toEqual({
      url: '/service/rest/v1/blobstores/my%20store',
      missing: [],
    });
    expect(requestUrl(assets, {}).missing).toEqual(['repository']);
    expect(requestUrl(assets, { repository: 'raw', continuationToken: '' }).url).toBe(
      '/service/rest/v1/assets?repository=raw',
    );
  });

  it('builds a body skeleton from the schema', () => {
    expect(skeleton({ $ref: '#/components/schemas/Store' }, DOC.components?.schemas ?? {})).toEqual({
      name: 'default',
      path: '',
      softQuota: { limit: 0 },
    });
  });
});
