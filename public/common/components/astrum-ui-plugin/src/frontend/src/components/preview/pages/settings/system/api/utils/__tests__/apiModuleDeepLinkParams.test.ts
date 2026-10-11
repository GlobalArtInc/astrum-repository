

import type { MergedApiEndpoint } from '../mergeSwaggerPermissions';
import {
  extractQueryFromLocationHash,
  filterEndpointsByPermissionSubstring,
  findEndpointByDeepLink,
  parseApiModuleHashParams,
  parseEndpointDeepLinkParam,
} from '../apiModuleDeepLinkParams';

function row(method: string, path: string, perms: string[]): MergedApiEndpoint {
  return {
    httpMethod: method,
    swaggerPathKey: '/x',
    fullPath: path,
    tag: 'T',
    permission: {
      httpMethod: method,
      pathPattern: path,
      permissions: perms.map((permission) => ({ permission, logical: 'AND' as const })),
      description: null,
      tag: null,
      authenticated: true,
    },
  };
}

describe('apiModuleDeepLinkParams', () => {
  it('extracts query from hash', () => {
    expect(extractQueryFromLocationHash('#browse/api?user=admin')).toBe('user=admin');
    expect(extractQueryFromLocationHash('#/path')).toBe('');
  });

  it('parses hash params', () => {
    expect(parseApiModuleHashParams('#browse/api?user=a&role=r')).toEqual({
      user: 'a',
      role: 'r',
      permission: null,
      endpoint: null,
    });
    expect(parseApiModuleHashParams('#?permission=astrum:foo')).toEqual({
      user: null,
      role: null,
      permission: 'astrum:foo',
      endpoint: null,
    });
  });

  it('parses endpoint deep link', () => {
    expect(parseEndpointDeepLinkParam('GET:%2Fv1%2Frepositories')).toEqual({
      method: 'GET',
      fullPath: '/service/rest/v1/repositories',
    });
    expect(parseEndpointDeepLinkParam('POST:/service/rest/v1/foo')).toEqual({
      method: 'POST',
      fullPath: '/service/rest/v1/foo',
    });
    expect(parseEndpointDeepLinkParam('bad')).toBeNull();
  });

  it('filters by permission substring', () => {
    const endpoints = [
      row('GET', '/service/rest/a', ['astrum:repos:read']),
      row('DELETE', '/service/rest/b', ['astrum:other:delete']),
    ];
    const out = filterEndpointsByPermissionSubstring(endpoints, 'repos');
    expect(out).toHaveLength(1);
    expect(out[0].fullPath).toBe('/service/rest/a');
  });

  it('finds endpoint by deep link param', () => {
    const endpoints = [row('GET', '/service/rest/v1/repositories', ['x'])];
    const found = findEndpointByDeepLink(endpoints, 'GET:%2Fv1%2Frepositories');
    expect(found?.httpMethod).toBe('GET');
  });
});
