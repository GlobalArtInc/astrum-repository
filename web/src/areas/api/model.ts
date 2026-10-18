export const READ = 'astrum:settings:read';
export const DOCUMENT = '/service/rest/swagger.json';
export const DOCS = 'https://links.globalart.dev/products/astrum/docs/rest-api';
export const METHODS = ['GET', 'POST', 'PUT', 'DELETE', 'PATCH', 'HEAD', 'OPTIONS'] as const;
export const ALL = '*';

export interface Schema {
  $ref?: string;
  type?: string;
  format?: string;
  items?: Schema;
  properties?: Record<string, Schema>;
  additionalProperties?: Schema | boolean;
  enum?: unknown[];
  example?: unknown;
  default?: unknown;
  allOf?: Schema[];
  oneOf?: Schema[];
  anyOf?: Schema[];
}

export interface Parameter {
  name: string;
  in: 'path' | 'query' | 'header' | 'cookie';
  required?: boolean;
  schema?: Schema;
  example?: unknown;
}

export interface MediaType {
  schema?: Schema;
}

export interface Operation {
  tags?: string[];
  summary?: string;
  operationId?: string;
  parameters?: Parameter[];
  requestBody?: { content?: Record<string, MediaType>; required?: boolean };
  responses?: Record<string, { content?: Record<string, MediaType> }>;
}

export interface OpenApi {
  openapi?: string;
  servers?: Array<{ url: string }>;
  paths?: Record<string, Record<string, Operation | Parameter[] | unknown>>;
  components?: { schemas?: Record<string, Schema> };
}

export interface Endpoint {
  key: string;
  method: string;
  path: string;
  fullPath: string;
  tag: string | null;
  summary: string;
  operation: Operation;
}

export interface Group {
  tag: string | null;
  endpoints: Endpoint[];
}

const HTTP = new Set(['get', 'put', 'post', 'delete', 'patch', 'head', 'options']);

export function base(document: OpenApi): string {
  return (document.servers?.[0]?.url ?? '').replace(/\/$/, '');
}

export function endpointKey(method: string, path: string): string {
  return `${method.toUpperCase()} ${path}`;
}

export function endpoints(document: OpenApi): Endpoint[] {
  const prefix = base(document);
  const found: Endpoint[] = [];
  for (const [path, item] of Object.entries(document.paths ?? {})) {
    const shared = Array.isArray(item.parameters) ? (item.parameters as Parameter[]) : [];
    for (const [method, value] of Object.entries(item)) {
      if (!HTTP.has(method) || value === null || typeof value !== 'object') {
        continue;
      }
      const operation = value as Operation;
      const own = operation.parameters ?? [];
      const parameters = [
        ...shared.filter((parameter) => !own.some((other) => other.name === parameter.name)),
        ...own,
      ];
      found.push({
        key: endpointKey(method, path),
        method: method.toUpperCase(),
        path,
        fullPath: `${prefix}${path}`,
        tag: operation.tags?.[0] ?? null,
        summary: operation.summary ?? '',
        operation: { ...operation, parameters },
      });
    }
  }
  return found;
}

export function tagsOf(list: Endpoint[]): string[] {
  const tags = new Set<string>();
  for (const endpoint of list) {
    if (endpoint.tag) {
      tags.add(endpoint.tag);
    }
  }
  return [...tags].sort(compareText);
}

function compareText(a: string, b: string): number {
  return a.localeCompare(b, 'en', { sensitivity: 'base' });
}

export interface Filters {
  query: string;
  method: string;
  tag: string;
}

export function filterEndpoints(list: Endpoint[], filters: Filters): Endpoint[] {
  const query = filters.query.trim().toLowerCase();
  return list.filter(
    (endpoint) =>
      (filters.method === ALL || endpoint.method === filters.method) &&
      (filters.tag === ALL || endpoint.tag === filters.tag) &&
      (query === '' ||
        endpoint.fullPath.toLowerCase().includes(query) ||
        endpoint.summary.toLowerCase().includes(query)),
  );
}

export function groupEndpoints(list: Endpoint[]): Group[] {
  const groups = new Map<string | null, Endpoint[]>();
  for (const endpoint of list) {
    const group = groups.get(endpoint.tag) ?? [];
    group.push(endpoint);
    groups.set(endpoint.tag, group);
  }
  return [...groups.entries()]
    .sort(([a], [b]) => (a === null ? 1 : b === null ? -1 : compareText(a, b)))
    .map(([tag, endpoints]) => ({
      tag,
      endpoints: endpoints.sort((a, b) =>
        a.path === b.path ? a.method.localeCompare(b.method) : a.path.localeCompare(b.path),
      ),
    }));
}

export function schemaName(schema: Schema | undefined): string | null {
  if (!schema) {
    return null;
  }
  if (schema.$ref) {
    return schema.$ref.split('/').pop() ?? schema.$ref;
  }
  if (schema.type === 'array') {
    const item = schemaName(schema.items);
    return item ? `${item}[]` : 'array';
  }
  return schema.type ?? null;
}

export function resolve(schema: Schema | undefined, schemas: Record<string, Schema>): Schema | undefined {
  if (!schema?.$ref) {
    return schema;
  }
  return schemas[schema.$ref.split('/').pop() ?? ''];
}

export function skeleton(schema: Schema | undefined, schemas: Record<string, Schema>, depth = 0): unknown {
  const resolved = resolve(schema, schemas);
  if (!resolved || depth > 5) {
    return null;
  }
  if (resolved.example !== undefined) {
    return resolved.example;
  }
  if (resolved.default !== undefined) {
    return resolved.default;
  }
  const combined = resolved.allOf ?? resolved.oneOf ?? resolved.anyOf;
  if (combined && combined.length > 0) {
    return skeleton(combined[0], schemas, depth + 1);
  }
  if (resolved.enum && resolved.enum.length > 0) {
    return resolved.enum[0];
  }
  switch (resolved.type) {
    case 'string':
      return '';
    case 'integer':
    case 'number':
      return 0;
    case 'boolean':
      return false;
    case 'array':
      return [];
    default: {
      const object: Record<string, unknown> = {};
      for (const [name, property] of Object.entries(resolved.properties ?? {})) {
        object[name] = skeleton(property, schemas, depth + 1);
      }
      return object;
    }
  }
}

export function bodySchema(operation: Operation): { type: string; schema?: Schema } | null {
  const content = operation.requestBody?.content;
  if (!content) {
    return null;
  }
  const type = Object.keys(content).find((name) => name.includes('json')) ?? Object.keys(content)[0];
  return type ? { type, schema: content[type]?.schema } : null;
}

export function initialValue(parameter: Parameter): string {
  const value = parameter.example ?? parameter.schema?.example ?? parameter.schema?.default;
  return value === undefined || value === null ? '' : String(value);
}

export function requestUrl(
  endpoint: Endpoint,
  values: Record<string, string>,
): { url: string; missing: string[] } {
  const missing: string[] = [];
  const parameters = endpoint.operation.parameters ?? [];
  let url = endpoint.fullPath;
  for (const parameter of parameters.filter((candidate) => candidate.in === 'path')) {
    const value = values[parameter.name] ?? '';
    if (value === '') {
      missing.push(parameter.name);
    }
    url = url.replace(`{${parameter.name}}`, encodeURIComponent(value));
  }
  const query = new URLSearchParams();
  for (const parameter of parameters.filter((candidate) => candidate.in === 'query')) {
    const value = values[parameter.name] ?? '';
    if (value !== '') {
      query.set(parameter.name, value);
    } else if (parameter.required) {
      missing.push(parameter.name);
    }
  }
  const text = query.toString();
  return { url: text ? `${url}?${text}` : url, missing };
}
