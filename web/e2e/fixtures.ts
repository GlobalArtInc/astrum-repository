import { type APIRequestContext, test as base, expect, type Page } from '@playwright/test';
import { type Running, startServer } from './server';

export const ADMIN = 'admin';

let current: { file: string; running: Running } | null = null;

async function serverFor(file: string): Promise<Running> {
  if (current?.file === file) {
    return current.running;
  }
  if (current) {
    await current.running.stop();
    current = null;
  }
  const running = await startServer({ port: 0 });
  current = { file, running };
  return running;
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

export const test = base.extend<{ server: Server }, { serverPerFile: undefined }>({
  serverPerFile: [
    async ({}, use) => {
      await use(undefined);
      if (current) {
        await current.running.stop();
        current = null;
      }
    },
    { scope: 'worker', auto: true },
  ],
  baseURL: [
    async ({}, use, testInfo) => {
      const running = await serverFor(testInfo.file);
      await use(running.url);
    },
    { scope: 'test', timeout: 90_000 },
  ],
  server: [
    async ({ request }, use, testInfo) => {
      const { url, adminPassword } = await serverFor(testInfo.file);
      await use({
        url,
        adminPassword,
        setAnonymous: (enabled) => setAnonymous(request, url, adminPassword, enabled),
      });
      await setAnonymous(request, url, adminPassword, false);
    },
    { scope: 'test', timeout: 90_000 },
  ],
});

export async function signIn(page: Page, password: string, user = ADMIN): Promise<void> {
  await page.getByLabel('User name').fill(user);
  await page.getByLabel('Password', { exact: true }).fill(password);
  await page.getByRole('button', { name: 'Sign in' }).click();
}

export async function noHorizontalScroll(page: Page): Promise<void> {
  await expect
    .poll(() => page.evaluate(() => document.documentElement.scrollWidth - window.innerWidth))
    .toBeLessThanOrEqual(0);
}

export { expect };
