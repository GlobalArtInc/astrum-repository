import type { OpenApi } from './model';

export const DOC: OpenApi = {
  openapi: '3.0.1',
  servers: [{ url: '/service/rest' }],
  paths: {
    '/v1/status': { get: { tags: ['Status'], summary: 'List status', responses: { '200': {} } } },
    '/v1/blobstores': {
      get: {
        tags: ['Blob store'],
        summary: 'List blob stores',
        responses: {
          '200': {
            content: {
              'application/json': {
                schema: { type: 'array', items: { $ref: '#/components/schemas/Store' } },
              },
            },
          },
        },
      },
    },
    '/v1/blobstores/{name}': {
      delete: {
        tags: ['Blob store'],
        summary: 'Delete blob stores',
        parameters: [{ name: 'name', in: 'path', required: true, schema: { type: 'string' } }],
        responses: { '204': {} },
      },
      get: {
        tags: ['Blob store'],
        summary: 'Get blob stores',
        parameters: [{ name: 'name', in: 'path', required: true, schema: { type: 'string' } }],
        responses: { '200': {} },
      },
    },
    '/v1/assets': {
      get: {
        tags: ['assets'],
        summary: 'List assets',
        parameters: [
          { name: 'repository', in: 'query', required: true, schema: { type: 'string' } },
          { name: 'continuationToken', in: 'query', schema: { type: 'string' } },
        ],
        responses: { '200': {} },
      },
    },
    '/v1/blobstores/file': {
      post: {
        tags: ['Blob store'],
        summary: 'Create blob stores file',
        requestBody: {
          content: { 'application/json': { schema: { $ref: '#/components/schemas/Store' } } },
        },
        responses: { '204': {} },
      },
    },
  },
  components: {
    schemas: {
      Store: {
        type: 'object',
        properties: {
          name: { type: 'string', example: 'default' },
          path: { type: 'string' },
          softQuota: { type: 'object', properties: { limit: { type: 'integer' } } },
        },
      },
    },
  },
};
