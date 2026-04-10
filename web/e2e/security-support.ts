import { type APIRequestContext, type Browser, expect, type Page } from '@playwright/test';
import { ADMIN, type Server, signIn } from './fixtures';

export function basic(user: string, password: string): string {
  return `Basic ${Buffer.from(`${user}:${password}`).toString('base64')}`;
}

export class AdminApi {
  constructor(
    private readonly request: APIRequestContext,
    private readonly server: Server,
  ) {}

  private headers(): Record<string, string> {
    return { Authorization: basic(ADMIN, this.server.adminPassword) };
  }

  async send(method: string, path: string, data?: unknown, ok: number[] = [200, 201, 204]) {
    const response = await this.request.fetch(`${this.server.url}${path}`, {
      method,
      headers: this.headers(),
      data,
    });
    expect(ok, `${method} ${path}: ${response.status()} ${await response.text()}`).toContain(
      response.status(),
    );
    return response;
  }

  async get<T>(path: string): Promise<T> {
    return (await (await this.send('GET', path)).json()) as T;
  }

  async rawRepository(name: string): Promise<void> {
    await this.send('POST', '/service/rest/v1/repositories/raw/hosted', {
      name,
      online: true,
      storage: { blobStoreName: 'default', strictContentTypeValidation: false, writePolicy: 'allow' },
      raw: { contentDisposition: 'INLINE' },
    });
  }

  async putFile(repository: string, path: string, text: string): Promise<void> {
    const response = await this.request.put(`${this.server.url}/repository/${repository}/${path}`, {
      headers: { ...this.headers(), 'Content-Type': 'text/plain' },
      data: text,
    });
    expect([201, 204], await response.text()).toContain(response.status());
  }

  async role(id: string, privileges: string[], roles: string[] = []): Promise<void> {
    await this.send('POST', '/service/rest/v1/security/roles', {
      id,
      name: id,
      description: id,
      privileges,
      roles,
    });
  }

  async user(userId: string, password: string, roles: string[]): Promise<void> {
    await this.send('POST', '/service/rest/v1/security/users', {
      userId,
      firstName: 'Test',
      lastName: userId,
      emailAddress: `${userId}@example.org`,
      password,
      status: 'active',
      roles,
    });
  }
}

export async function signInAs(page: Page, user: string, password: string): Promise<void> {
  await page.goto('/login');
  await signIn(page, password, user);
  await expect(page.getByRole('button', { name: /Account menu of/ })).toBeVisible();
}

export async function signOut(page: Page): Promise<void> {
  await page.getByRole('button', { name: /Account menu of/ }).click();
  await page.getByRole('menuitem', { name: 'Sign out' }).click();
  await expect(page.getByRole('button', { name: /Account menu of/ })).toHaveCount(0);
  await expect(page.getByRole('link', { name: 'Sign in' }).or(page.getByLabel('User name'))).toBeVisible();
}

export function toast(page: Page, text: string) {
  return page.getByRole('region', { name: /Notifications/ }).getByText(text, { exact: true });
}

export async function choose(page: Page, label: string | RegExp, option: string): Promise<void> {
  await page.getByRole('combobox', { name: label }).click();
  await page.getByRole('option', { name: option, exact: true }).click();
}

export async function stranger(browser: Browser, url: string) {
  const context = await browser.newContext({ baseURL: url });
  return { context, page: await context.newPage() };
}
