import { mkdirSync } from 'node:fs';
import { join } from 'node:path';
import type { Page } from '@playwright/test';
import { expect, signIn, test } from './fixtures';

test.describe.configure({ timeout: 120_000 });

const SHOTS = process.env.ASTRUM_SHOTS_DIR;
const SIZES = [
  { name: '1440', width: 1440, height: 900 },
  { name: '390', width: 390, height: 844 },
] as const;
const THEMES = ['dark', 'light'] as const;

async function signInAsAdmin(page: Page, password: string) {
  await page.goto('/login');
  await signIn(page, password);
  await expect(page.getByRole('button', { name: /Account menu of/ })).toBeVisible();
}

async function openOperation(page: Page, tag: string, path: string) {
  await page.getByPlaceholder('Filter by tag').fill(tag);
  const summary = page.locator(`.opblock-summary-path[data-path="${path}"]`).first();
  await summary.click();
  await page.getByRole('button', { name: 'Try it out' }).click();
  await page.getByRole('button', { name: 'Execute' }).click();
  return page.locator('.opblock.is-open .live-responses-table').first();
}

test('the API page shows the REST API in Swagger UI and runs it', async ({ page, server }) => {
  const outside: string[] = [];
  page.on('request', (request) => {
    if (!request.url().startsWith(server.url) && !request.url().startsWith('data:')) {
      outside.push(request.url());
    }
  });
  await signInAsAdmin(page, server.adminPassword);
  await page.getByRole('link', { name: 'API', exact: true }).first().click();
  await expect(page).toHaveURL(/\/api$/);
  await expect(page.getByRole('heading', { level: 1, name: 'API', exact: true })).toBeVisible();
  await expect(page.locator('.swagger-ui .opblock-tag').first()).toBeVisible();

  const status = await openOperation(page, 'Status', '/v1/status');
  await expect(status.locator('tbody .response-col_status').first()).toHaveText('200');

  await page.goto('/api');
  const users = await openOperation(page, 'Security management: users', '/v1/security/users');
  await expect(users).toContainText('"userId": "admin"');

  await page.goto('/#browse/api');
  await expect(page).toHaveURL(/\/api$/);
  expect(outside).toEqual([]);
});

test.describe('screenshots', () => {
  test.skip(!SHOTS, 'set ASTRUM_SHOTS_DIR to take screenshots');

  for (const size of SIZES) {
    for (const theme of THEMES) {
      test(`API screens at ${size.name} in ${theme}`, async ({ page, server }) => {
        const shoot = async (name: string) => {
          if (SHOTS) {
            mkdirSync(SHOTS, { recursive: true });
            await page.waitForTimeout(300);
            await page.screenshot({ path: join(SHOTS, `${name}-${size.name}-${theme}.png`) });
          }
        };
        await page.setViewportSize({ width: size.width, height: size.height });
        await page.addInitScript(
          (value) => window.localStorage.setItem('astrum-theme-preference', value),
          theme,
        );
        await signInAsAdmin(page, server.adminPassword);
        await page.goto('/api');
        await expect(page.getByRole('heading', { level: 1, name: 'API', exact: true })).toBeVisible();
        await expect(page.locator('.swagger-ui .opblock-tag').first()).toBeVisible();
        await shoot('api');
        const response = await openOperation(page, 'Blob store', '/v1/blobstores');
        await expect(response.locator('tbody .response-col_status').first()).toHaveText('200');
        await shoot('api-endpoint');
      });
    }
  }
});
