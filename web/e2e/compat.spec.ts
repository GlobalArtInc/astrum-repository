import { mkdirSync } from 'node:fs';
import { join } from 'node:path';
import { AdminApi } from './admin-api';
import { componentUpload, hosted, wheel, wheelName } from './content';
import { expect, signIn, test } from './fixtures';

const DIR = process.env.ASTRUM_SHOTS_DIR;
const BLOCKED =
  'Proxy URL blocked: Host resolves to private/local IP address(es): 10.1.2.3 (private network address). To allow connections, update the SSRF protection settings via the API at /v1/security/ssrf-protection.';

async function shot(page: import('@playwright/test').Page, name: string) {
  if (DIR) {
    mkdirSync(DIR, { recursive: true });
    await page.waitForTimeout(300);
    await page.screenshot({ path: join(DIR, `${name}-1440-dark.png`) });
  }
}

test('a proxy whose remote is on a private network is refused like the old forms refuse it', async ({
  page,
  request,
  server,
}) => {
  const admin = new AdminApi(request, server.url, server.adminPassword);
  await page.goto('/login');
  await signIn(page, server.adminPassword);
  await expect(page.getByRole('button', { name: /Account menu of/ })).toBeVisible();

  await page.goto('/admin/repository/repositories/create/raw/proxy');
  await page.getByRole('textbox', { name: 'Name', exact: true }).fill('inside');
  await page.getByRole('textbox', { name: 'Remote Storage', exact: true }).fill('http://10.1.2.3/');
  await page.getByRole('button', { name: 'Continue' }).click();
  const banner = page
    .getByRole('alert')
    .filter({ hasText: BLOCKED })
    .filter({ has: page.getByRole('button', { name: 'Dismiss' }) });
  await expect(banner).toBeVisible();
  await expect(banner).toBeInViewport();
  await expect(page.getByText(BLOCKED)).not.toHaveCount(0);
  await shot(page, 'ssrf-create');
  await banner.getByRole('button', { name: 'Dismiss' }).click();
  await expect(banner).toHaveCount(0);
  expect(await admin.status('/v1/repositories/raw/proxy/inside')).toBe(404);

  await admin.send('PUT', '/v1/security/ssrf-protection', {
    enabled: true,
    allowedIPs: ['10.0.0.0/8'],
    allowedDomains: [],
  });
  await page.getByRole('button', { name: 'Continue' }).click();
  await expect(page.getByText('Repository "inside" created', { exact: true })).toBeVisible();
  await admin.send('PUT', '/v1/security/ssrf-protection', {
    enabled: true,
    allowedIPs: [],
    allowedDomains: [],
  });

  await page.goto('/admin/repository/repositories/inside');
  const online = page.getByRole('checkbox', { name: /Online/ });
  await online.click();
  await page.getByRole('button', { name: 'Save' }).click();
  await expect(page.getByText('Changes to inside saved', { exact: true })).toBeVisible();

  await page.getByRole('textbox', { name: 'Remote Storage', exact: true }).fill('http://10.1.2.3/other/');
  await page.getByRole('button', { name: 'Save' }).click();
  const footer = page.getByRole('alert').filter({ hasText: 'Proxy URL blocked' }).first();
  await expect(footer).toBeVisible();
  await expect(footer).toContainText('10.1.2.3 (private network address)');
  await shot(page, 'ssrf-edit');
  const stored = await admin.repository('raw', 'proxy', 'inside');
  expect((stored.proxy as Record<string, unknown>).remoteUrl).toBe('http://10.1.2.3/');
});

test('a PyPI project is found however its name is spelled', async ({ page, server }) => {
  await hosted(server, 'pypi', 'compat-pypi');
  const form = new FormData();
  form.append(
    'pypi.asset',
    new Blob([new Uint8Array(wheel('report-builder', '0.4.2', 'Builds reports'))]),
    wheelName('report-builder', '0.4.2'),
  );
  await componentUpload(server, 'compat-pypi', form);
  await page.goto('/login');
  await signIn(page, server.adminPassword);
  await expect(page.getByRole('button', { name: /Account menu of/ })).toBeVisible();
  for (const spelling of ['report_builder', 'Report.Builder', 'report builder', 'report-b']) {
    await page.goto(`/search?q=${encodeURIComponent(spelling)}`);
    await expect(
      page.getByRole('region', { name: 'Search results' }).getByText('report-builder').first(),
      spelling,
    ).toBeVisible();
  }
  await page.goto(`/search?q=${encodeURIComponent('rep-builder')}`);
  await expect(page.getByText('Nothing found', { exact: true })).toBeVisible();
});
