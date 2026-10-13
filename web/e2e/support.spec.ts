import { mkdirSync } from 'node:fs';
import { join } from 'node:path';
import type { Page } from '@playwright/test';
import { call, ensure, hosted, put } from './content';
import { expect, noHorizontalScroll, type Server, signIn, test } from './fixtures';

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

async function currentLog(server: Server): Promise<string> {
  const response = await call(server, 'GET', '/service/rest/internal/logging/logs');
  await ensure(response, 'list logs');
  const files = (await response.json()) as Array<{ fileName: string; current: boolean }>;
  const current = files.find((file) => file.current);
  expect(current, 'the current log file').toBeTruthy();
  return current?.fileName ?? '';
}

async function breakTheQuota(server: Server) {
  const store = await call(server, 'GET', '/service/rest/v1/blobstores/file/default');
  await ensure(store, 'read the default store');
  const settings = (await store.json()) as Record<string, unknown>;
  await ensure(
    await call(server, 'PUT', '/service/rest/v1/blobstores/file/default', {
      ...settings,
      softQuota: { type: 'spaceUsedQuota', limit: 1 },
    }),
    'set a quota',
  );
  await hosted(server, 'raw', 'quota-raw');
  await put(server, '/repository/quota-raw/big.txt', 'more than one byte');
}

test('logs list, view, mark and download', async ({ page, server }) => {
  const telemetry: string[] = [];
  page.on('request', (request) => {
    if (request.url().includes('user-telemetry')) {
      telemetry.push(request.url());
    }
  });
  const file = await currentLog(server);
  await signInAsAdmin(page, server.adminPassword);
  await page.goto('/settings');
  await page.getByRole('link', { name: /^Logs/ }).click();
  await expect(page.getByRole('heading', { level: 1, name: 'Logs' })).toBeVisible();
  await page.getByRole('button', { name: `Open ${file}` }).click();
  await expect(page.getByRole('heading', { level: 1, name: 'Log Viewer' })).toBeVisible();
  const log = page.getByRole('log', { name: `Contents of ${file}` });
  await expect(log).toBeVisible();
  await page.getByRole('textbox', { name: 'Marker to insert:' }).fill('e2e support mark');
  await page.getByRole('button', { name: 'Insert' }).click();
  await expect(page.getByText('The mark was inserted', { exact: true })).toBeVisible();
  await expect(log).toContainText('MARK - e2e support mark (by admin)');
  const href = await page.getByRole('link', { name: 'Download' }).getAttribute('href');
  const download = await page.request.get(href ?? '');
  expect(download.status()).toBe(200);
  expect(await download.text()).toContain('e2e support mark');
  expect(telemetry).toEqual([]);
});

test('logging levels are created, changed and removed', async ({ page, server }) => {
  await signInAsAdmin(page, server.adminPassword);
  await page.goto('/admin/support/logging');
  await expect(page.getByRole('heading', { level: 1, name: 'Logging' })).toBeVisible();
  await expect(page.getByText(/Showing \d+ of \d+ loggers/)).toBeVisible();
  await page.getByRole('link', { name: 'Create Logger' }).click();
  await page.getByRole('textbox', { name: /Logger Name/ }).fill('astrum_content::proxy');
  await page.getByRole('combobox', { name: /Logger Level/ }).click();
  await page.getByRole('option', { name: 'DEBUG' }).click();
  await page.getByRole('button', { name: 'Create Logger' }).click();
  await expect(page.getByText('Logger astrum_content::proxy created', { exact: true })).toBeVisible();
  await page.getByRole('textbox', { name: 'Filter by logger name' }).fill('proxy');
  await expect(page.getByText('Showing 1 of', { exact: false })).toBeVisible();
  const levels = await call(
    server,
    'GET',
    '/service/rest/internal/ui/loggingConfiguration/astrum_content::proxy',
  );
  expect(await levels.json()).toEqual({ name: 'astrum_content::proxy', level: 'DEBUG', override: true });
  await page.getByRole('button', { name: 'Edit astrum_content::proxy' }).first().click();
  await expect(page.getByRole('heading', { level: 1, name: 'astrum_content::proxy' })).toBeVisible();
  await page.getByRole('button', { name: 'Delete' }).click();
  await page.getByRole('alertdialog').getByRole('button', { name: 'Delete' }).click();
  await expect(
    page.getByText('The level override of astrum_content::proxy was removed', { exact: true }),
  ).toBeVisible();
  const after = await call(
    server,
    'GET',
    '/service/rest/internal/ui/loggingConfiguration/astrum_content::proxy',
  );
  expect(((await after.json()) as { override: boolean }).override).toBe(false);
});

test('system information shows the server', async ({ page, server }) => {
  await signInAsAdmin(page, server.adminPassword);
  await page.goto('/admin/support/system-information');
  await expect(page.getByRole('heading', { level: 1, name: 'System Information' })).toBeVisible();
  await expect(page.getByRole('button', { name: /Astrum Status/ })).toHaveAttribute('aria-expanded', 'true');
  await page.getByRole('button', { name: /System Database/ }).click();
  await expect(page.getByText('PostgreSQL', { exact: true })).toBeVisible();
});

test('a failing check lights the bell and opens Metric Health', async ({ page, server }) => {
  await breakTheQuota(server);
  await signInAsAdmin(page, server.adminPassword);
  const bell = page.getByRole('button', { name: 'System status: 1 health check fails' });
  await expect(bell).toBeVisible();
  await bell.click();
  await expect(page.getByRole('heading', { level: 1, name: 'Metric Health' })).toBeVisible();
  await expect(page.getByRole('heading', { level: 2, name: 'Blob Stores Quota' })).toBeVisible();
  await expect(page.getByText('Unhealthy', { exact: true })).toBeVisible();
  await page.getByRole('button', { name: 'Database', exact: true }).click();
  await expect(page.getByRole('heading', { level: 2, name: 'Database' })).toBeVisible();
  await expect(page.getByText('PostgreSQL answers.')).toBeVisible();
});

test('a support ZIP is created and downloaded', async ({ page, server }) => {
  await signInAsAdmin(page, server.adminPassword);
  await page.goto('/admin/support/support-zip');
  await expect(page.getByRole('heading', { level: 1, name: 'Support ZIP' })).toBeVisible();
  await page.getByRole('button', { name: 'Create Support ZIP' }).click();
  await expect(page.getByRole('heading', { name: 'Support ZIP created' })).toBeVisible();
  const href = await page.getByRole('link', { name: 'Download' }).getAttribute('href');
  const download = await page.request.get(href ?? '');
  expect(download.status()).toBe(200);
  expect((await download.body()).subarray(0, 2).toString()).toBe('PK');
});

test.describe('screenshots', () => {
  test.skip(!SHOTS, 'set ASTRUM_SHOTS_DIR to take screenshots');

  for (const size of SIZES) {
    for (const theme of THEMES) {
      test(`support screens at ${size.name} in ${theme}`, async ({ page, server }) => {
        const shoot = async (name: string) => {
          if (SHOTS) {
            mkdirSync(SHOTS, { recursive: true });
            await page.waitForTimeout(400);
            await page.screenshot({ path: join(SHOTS, `${name}-${size.name}-${theme}.png`) });
          }
        };
        const file = await currentLog(server);
        await page.setViewportSize({ width: size.width, height: size.height });
        await page.addInitScript(
          (value) => window.localStorage.setItem('astrum-theme-preference', value),
          theme,
        );
        await call(server, 'POST', '/service/rest/internal/logging/log/mark', 'screenshot');
        await signInAsAdmin(page, server.adminPassword);
        const pages: Array<[string, string, string]> = [
          ['logs', '/admin/support/logs', 'Logs'],
          ['log-viewer', `/admin/support/logs/${file}`, 'Log Viewer'],
          ['logging', '/admin/support/logging', 'Logging'],
          ['logger-create', '/admin/support/logging/create', 'Create Logger'],
          ['system-information', '/admin/support/system-information', 'System Information'],
          ['metric-health', '/admin/support/metric-health', 'Metric Health'],
          ['support-zip', '/admin/support/support-zip', 'Support ZIP'],
        ];
        for (const [name, path, title] of pages) {
          await page.goto(path);
          await expect(page.getByRole('heading', { level: 1, name: title })).toBeVisible();
          await noHorizontalScroll(page);
          await shoot(name);
        }
      });
    }
  }
});
