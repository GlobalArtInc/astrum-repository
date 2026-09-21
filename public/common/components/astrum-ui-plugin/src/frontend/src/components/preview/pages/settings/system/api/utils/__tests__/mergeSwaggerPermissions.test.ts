

import type { ApiEndpointPermissionDto } from '../../types';
import {
  buildPermissionIndex,
  joinSwaggerBasePath,
  mergeSwaggerAndPermissions,
  permissionLookupKey,
  sliceSwaggerSpec,
} from '../mergeSwaggerPermissions';

describe('mergeSwaggerPermissions', () => {
  it('joinSwaggerBasePath avoids double slashes', () => {
    expect(joinSwaggerBasePath('/service/rest', '/v1/status')).toBe('/service/rest/v1/status');
  });

  it('permissionLookupKey normalizes', () => {
    expect(permissionLookupKey('get', '/service/rest/v1//status/')).toBe('GET|/service/rest/v1/status');
  });

  it('merge attaches permission rows by method and path', () => {
    const swagger = {
      swagger: '2.0',
      basePath: '/service/rest',
      paths: {
        '/v1/status': {
          get: { summary: 'Status', tags: ['System'] },
        },
      },
    };
    const perm: ApiEndpointPermissionDto[] = [
      {
        httpMethod: 'GET',
        pathPattern: '/service/rest/v1/status',
        permissions: [{ permission: 'astrum:*', logical: 'AND' }],
        description: 'd',
        tag: 'T',
        authenticated: true,
      },
    ];
    const rows = mergeSwaggerAndPermissions(swagger, perm);
    expect(rows).toHaveLength(1);
    expect(rows[0].fullPath).toBe('/service/rest/v1/status');
    expect(rows[0].permission?.permissions[0].permission).toBe('astrum:*');
  });

  it('buildPermissionIndex supports lookup', () => {
    const perm: ApiEndpointPermissionDto[] = [
      {
        httpMethod: 'POST',
        pathPattern: '/service/rest/v1/foo',
        permissions: [],
        description: null,
        tag: null,
        authenticated: true,
      },
    ];
    const idx = buildPermissionIndex(perm);
    expect(idx.get(permissionLookupKey('POST', '/service/rest/v1/foo'))).toBeDefined();
  });

  it('sliceSwaggerSpec keeps path parameters and single method', () => {
    const swagger = {
      swagger: '2.0',
      paths: {
        '/v1/r': {
          parameters: [{ name: 'q', in: 'query' }],
          get: { operationId: 'a' },
          post: { operationId: 'b' },
        },
      },
    };
    const slim = sliceSwaggerSpec(swagger as Record<string, unknown>, '/v1/r', 'get');
    const paths = slim.paths as Record<string, Record<string, unknown>>;
    expect(paths['/v1/r'].get).toBeDefined();
    expect(paths['/v1/r'].post).toBeUndefined();
    expect(paths['/v1/r'].parameters).toBeDefined();
  });
});
