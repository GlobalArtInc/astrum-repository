import type { APIRequestContext } from '@playwright/test';
import { ADMIN, expect } from './fixtures';

export type Json = Record<string, unknown>;

export class AdminApi {
  constructor(
    private readonly request: APIRequestContext,
    private readonly url: string,
    private readonly password: string,
  ) {}

  private headers(user = ADMIN, password = this.password) {
    return { Authorization: `Basic ${Buffer.from(`${user}:${password}`).toString('base64')}` };
  }

  async get<T = Json>(path: string): Promise<T> {
    const response = await this.request.get(`${this.url}/service/rest${path}`, { headers: this.headers() });
    expect(response.ok(), `GET ${path}: ${response.status()} ${await response.text()}`).toBe(true);
    return (await response.json()) as T;
  }

  async status(path: string): Promise<number> {
    const response = await this.request.get(`${this.url}/service/rest${path}`, { headers: this.headers() });
    return response.status();
  }

  async send(method: 'POST' | 'PUT' | 'DELETE', path: string, data?: unknown): Promise<void> {
    const response = await this.request.fetch(`${this.url}/service/rest${path}`, {
      method,
      headers: this.headers(),
      data,
    });
    expect(response.ok(), `${method} ${path}: ${response.status()} ${await response.text()}`).toBe(true);
  }

  async upload(repository: string, path: string, body: string): Promise<void> {
    const response = await this.request.put(`${this.url}/repository/${repository}/${path}`, {
      headers: { ...this.headers(), 'Content-Type': 'text/plain' },
      data: body,
    });
    expect(response.ok(), `${response.status()} ${await response.text()}`).toBe(true);
  }

  async repository(format: string, type: string, name: string): Promise<Json> {
    return this.get(`/v1/repositories/${format}/${type}/${name}`);
  }

  async createRepository(format: string, type: string, body: Json): Promise<void> {
    await this.send('POST', `/v1/repositories/${format}/${type}`, body);
  }

  async createUser(userId: string, password: string, permission: string): Promise<void> {
    const privilege = `${userId}-privilege`;
    await this.send('POST', '/v1/security/privileges/wildcard', {
      name: privilege,
      description: `Only ${permission}`,
      pattern: permission,
    });
    await this.send('POST', '/v1/security/roles', {
      id: `${userId}-role`,
      name: `${userId} role`,
      description: `Only ${permission}`,
      privileges: [privilege],
      roles: [],
    });
    await this.send('POST', '/v1/security/users', {
      userId,
      firstName: 'Read',
      lastName: 'Only',
      emailAddress: `${userId}@example.org`,
      password,
      status: 'active',
      roles: [`${userId}-role`],
    });
  }
}

export function canonical(value: unknown): unknown {
  if (Array.isArray(value)) {
    return value.map(canonical);
  }
  if (value && typeof value === 'object') {
    return Object.fromEntries(
      Object.keys(value as Json)
        .sort()
        .map((key) => [key, canonical((value as Json)[key])]),
    );
  }
  return value;
}

export function differences(before: unknown, after: unknown, path = ''): string[] {
  if (
    before &&
    after &&
    typeof before === 'object' &&
    typeof after === 'object' &&
    !Array.isArray(before) &&
    !Array.isArray(after)
  ) {
    const keys = new Set([...Object.keys(before as Json), ...Object.keys(after as Json)]);
    return [...keys].flatMap((key) =>
      differences((before as Json)[key], (after as Json)[key], path ? `${path}.${key}` : key),
    );
  }
  return JSON.stringify(before) === JSON.stringify(after) ? [] : [path];
}
