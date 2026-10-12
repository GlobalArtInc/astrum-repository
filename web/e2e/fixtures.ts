import { type APIRequestContext, test as base, expect, type Page } from '@playwright/test';

export const ADMIN = 'admin';

function required(name: string): string {
  const value = process.env[name];
  if (!value) {
    throw new Error(`${name} is not set; the global setup starts the server`);
  }
  return value;
}

export interface Server {
  url: string;
  adminPassword: string;
  setAnonymous(enabled: boolean): Promise<void>;
}

function basic(password: string): string {
  return `Basic ${Buffer.from(`${ADMIN}:${password}`).toString('base64')}`;
}

async function setAnonymous(request: APIRequestContext, url: string, password: string, enabled: boolean) {
  const response = await request.put(`${url}/service/rest/v1/security/anonymous`, {
    headers: { Authorization: basic(password) },
    data: { enabled, userId: 'anonymous', realmName: 'AstrumAuthorizingRealm' },
  });
  expect(response.ok(), await response.text()).toBe(true);
}

export const test = base.extend<{ server: Server }>({
  baseURL: async ({}, use) => {
    await use(required('ASTRUM_E2E_URL'));
  },
  server: async ({ request }, use) => {
    const url = required('ASTRUM_E2E_URL');
    const adminPassword = required('ASTRUM_E2E_ADMIN_PASSWORD');
    await use({
      url,
      adminPassword,
      setAnonymous: (enabled) => setAnonymous(request, url, adminPassword, enabled),
    });
    await setAnonymous(request, url, adminPassword, false);
  },
});

export async function signIn(page: Page, password: string, user = ADMIN): Promise<void> {
  await page.getByLabel('User name').fill(user);
  await page.getByLabel('Password', { exact: true }).fill(password);
  await page.getByRole('button', { name: 'Sign in' }).click();
}

export async function noHorizontalScroll(page: Page): Promise<void> {
  const overflow = await page.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);
  expect(overflow).toBeLessThanOrEqual(0);
}

export { expect };
