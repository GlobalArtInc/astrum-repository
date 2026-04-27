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

test('the API page lists, filters and runs the REST API', async ({ page, server }) => {
  const telemetry: string[] = [];
  page.on('request', (request) => {
    if (request.url().includes('user-telemetry')) {
      telemetry.push(request.url());
    }
  });
  await signInAsAdmin(page, server.adminPassword);
  await page.getByRole('link', { name: 'API', exact: true }).first().click();
  await expect(page).toHaveURL(/\/api$/);
  await expect(page.getByRole('heading', { level: 1, name: 'API' })).toBeVisible();
  const list = page.getByRole('navigation', { name: 'Endpoints' });
  await expect(list.getByText('/service/rest/v1/repositories', { exact: true }).first()).toBeVisible();
  await page.getByRole('textbox', { name: 'Search by path or summary…' }).fill('v1/status');
  await page.getByRole('combobox', { name: 'Method' }).click();
  await page.getByRole('option', { name: 'GET', exact: true }).click();
  await expect(list.getByRole('button')).toHaveCount(3);
  await list.getByRole('button', { name: /\/service\/rest\/v1\/status$/ }).click();
  await expect(page).toHaveURL(/method=GET&path=%2Fv1%2Fstatus$/);
  await expect(page.getByRole('heading', { level: 2, name: '/service/rest/v1/status' })).toBeVisible();
  await page.getByRole('button', { name: 'Execute' }).click();
  await expect(page.getByText(/^200/).first()).toBeVisible();
  await expect(page.getByText('The response has no body')).toBeVisible();

  await page.goto('/api?method=GET&path=%2Fv1%2Fsecurity%2Fusers');
  await page.getByRole('button', { name: 'Execute' }).click();
  await expect(page.getByText(/"userId": "admin"/)).toBeVisible();
  await page.goto('/#browse/api');
  await expect(page).toHaveURL(/\/api$/);
  expect(telemetry).toEqual([]);
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
        await expect(page.getByRole('heading', { level: 1, name: 'API' })).toBeVisible();
        await expect(page.getByText('/service/rest/v1/assets', { exact: true }).first()).toBeVisible();
        await shoot('api');
        await page.goto('/api?method=GET&path=%2Fv1%2Fblobstores');
        await page.getByRole('button', { name: 'Execute' }).click();
        await expect(page.getByText(/^200/).first()).toBeVisible();
        await shoot('api-endpoint');
      });
    }
  }
});
