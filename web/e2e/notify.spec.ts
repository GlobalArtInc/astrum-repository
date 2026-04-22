import { mkdirSync } from 'node:fs';
import { createServer, type IncomingHttpHeaders } from 'node:http';
import type { AddressInfo } from 'node:net';
import { join } from 'node:path';
import type { Page } from '@playwright/test';
import { call, ensure, hosted, put } from './content';
import { expect, noHorizontalScroll, signIn, test } from './fixtures';

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

interface Received {
  headers: IncomingHttpHeaders;
  body: Record<string, unknown>;
}

async function receiver(): Promise<{ url: string; received: Received[]; close(): void }> {
  const received: Received[] = [];
  const server = createServer((request, response) => {
    const chunks: Buffer[] = [];
    request.on('data', (chunk: Buffer) => chunks.push(chunk));
    request.on('end', () => {
      received.push({ headers: request.headers, body: JSON.parse(Buffer.concat(chunks).toString('utf8')) });
      response.writeHead(200, { 'Content-Type': 'application/json' });
      response.end('{}');
    });
  });
  await new Promise<void>((resolve) => server.listen(0, '127.0.0.1', resolve));
  const { port } = server.address() as AddressInfo;
  return { url: `http://127.0.0.1:${port}/hook`, received, close: () => server.close() };
}

test('notification channels are added, tested, delivered to and removed', async ({ page, server }) => {
  const hook = await receiver();
  try {
    await ensure(
      await call(server, 'PUT', '/service/rest/v1/security/ssrf-protection', {
        enabled: true,
        allowedIPs: ['127.0.0.1'],
        allowedDomains: [],
      }),
      'allow the local webhook',
    );
    await hosted(server, 'raw', 'notify-raw', { raw: { contentDisposition: 'ATTACHMENT' } });
    await signInAsAdmin(page, server.adminPassword);
    await page.goto('/settings');
    await page.getByRole('link', { name: /Notifications/ }).click();
    await expect(page.getByRole('heading', { level: 1, name: 'Notifications' })).toBeVisible();
    await expect(page.getByText('No notification channels')).toBeVisible();
    await page.getByRole('button', { name: 'Add Channel' }).click();
    const dialog = page.getByRole('dialog', { name: 'New Notification Channel' });
    await dialog.getByLabel('Name').fill('Build hook');
    await dialog.getByRole('radio', { name: 'Webhook', exact: true }).click();
    await dialog.getByLabel('Webhook URL').fill(hook.url);
    await dialog.getByLabel('Signing Secret').fill('e2e-secret');
    await dialog.getByRole('button', { name: 'Send Test Message' }).click();
    await expect(page.getByText('The test message was delivered. Check the chat.').first()).toBeVisible();
    expect(hook.received.at(-1)?.headers['x-astrum-event']).toBe('test');
    await dialog.getByRole('button', { name: 'Save' }).click();
    await expect(dialog).toBeHidden();
    const row = page.getByRole('row', { name: /Build hook/ });
    await expect(row.getByText('127.0.0.1')).toBeVisible();
    await expect(row.getByText('All hosted repositories')).toBeVisible();

    await put(server, '/repository/notify-raw/builds/app-1.zip', 'app');
    await expect
      .poll(() => hook.received.filter((entry) => entry.body.event === 'component.published').length, {
        timeout: 40_000,
        intervals: [1000],
      })
      .toBe(1);
    const published = hook.received.find((entry) => entry.body.event === 'component.published');
    expect((published?.body.repository as Record<string, unknown> | undefined)?.name).toBe('notify-raw');
    expect(String(published?.headers['x-astrum-signature'])).toMatch(/^sha256=[0-9a-f]{64}$/);

    await row.getByRole('switch').click();
    await expect(row.getByRole('switch')).not.toBeChecked();
    await row.getByRole('button', { name: 'Edit' }).click();
    const edit = page.getByRole('dialog', { name: 'Edit Notification Channel' });
    await expect(edit.getByText('Saved. Leave the field empty to keep it.').first()).toBeVisible();
    await edit.getByRole('button', { name: 'Cancel' }).click();
    await row.getByRole('button', { name: 'Delete' }).click();
    await page.getByRole('alertdialog').getByRole('button', { name: 'Delete' }).click();
    await expect(page.getByText('No notification channels')).toBeVisible();
  } finally {
    hook.close();
  }
});

test('capabilities set the base URL, the title and the branding', async ({ page, server }) => {
  await signInAsAdmin(page, server.adminPassword);
  await page.goto('/admin/system/capabilities');
  await expect(page.getByRole('heading', { level: 1, name: 'Capabilities' })).toBeVisible();
  await expect(page.getByText('Shown 1 of 1')).toBeVisible();
  await page.getByRole('link', { name: 'UI: Settings' }).click();
  await page.getByRole('tab', { name: 'Settings' }).click();
  await page.getByLabel('Title').fill('Team Repository');
  await page.getByRole('button', { name: 'Save' }).click();
  await expect(page.getByText('Capability “UI: Settings” updated').first()).toBeVisible();
  await expect(page).toHaveTitle('UI: Settings - Team Repository');

  await page.goto('/admin/system/capabilities/create');
  await expect(page.getByText('Types available: 2 of 3')).toBeVisible();
  await page.getByRole('radio', { name: /Base URL/ }).click();
  await page.getByRole('button', { name: 'Continue' }).click();
  await page.getByLabel('URL').fill('https://repo.example.com');
  await page.getByRole('button', { name: 'Create' }).click();
  await expect(page.getByRole('heading', { level: 1, name: 'Base URL' })).toBeVisible();
  const state = await (await call(server, 'GET', '/service/rest/internal/ui/state')).json();
  expect(state.baseUrl).toBe('https://repo.example.com');
  const repositories = await call(server, 'GET', '/service/rest/v1/capabilities/baseurl');
  expect((await repositories.json()).state).toBe('active');
  await page.getByRole('button', { name: 'Delete' }).click();
  await page.getByRole('alertdialog').getByRole('button', { name: 'Delete' }).click();
  await expect(page.getByRole('heading', { level: 1, name: 'Capabilities' })).toBeVisible();

  await ensure(
    await call(server, 'POST', '/service/rest/v1/capabilities', {
      type: 'rapture.branding',
      enabled: true,
      notes: '',
      properties: {
        headerEnabled: 'true',
        headerHtml: '<b class="e2e-brand">Team header</b><script>window.hacked = true</script>',
        footerEnabled: 'true',
        footerHtml: '<a href="javascript:alert(1)" onclick="window.hacked = true">Team footer</a>',
      },
    }),
    'branding',
  );
  await page.goto('/');
  await expect(page.getByTestId('branding-header')).toHaveText('Team header');
  await expect(page.getByTestId('branding-footer').getByText('Team footer')).toBeVisible();
  await page.getByTestId('branding-footer').getByText('Team footer').click();
  expect(await page.evaluate(() => (window as unknown as { hacked?: boolean }).hacked)).toBeUndefined();
  expect(await page.getByTestId('branding-footer').innerHTML()).toBe('<a>Team footer</a>');
});

test('the nodes page shows this node', async ({ page, server }) => {
  await signInAsAdmin(page, server.adminPassword);
  await page.goto('/admin/system/nodes');
  await expect(page.getByRole('heading', { level: 1, name: 'Nodes' })).toBeVisible();
  await expect(page.getByText(/runs as a single node/)).toBeVisible();
  await expect(page.getByText('Current node')).toBeVisible();
  await page.setViewportSize({ width: 390, height: 844 });
  await noHorizontalScroll(page);
});

test.describe('screenshots', () => {
  test.skip(!SHOTS, 'set ASTRUM_SHOTS_DIR to take screenshots');

  for (const size of SIZES) {
    for (const theme of THEMES) {
      test(`notification, capability and node screens at ${size.name} in ${theme}`, async ({
        page,
        server,
      }) => {
        const shoot = async (name: string) => {
          if (SHOTS) {
            mkdirSync(SHOTS, { recursive: true });
            await page.waitForTimeout(300);
            await page.screenshot({ path: join(SHOTS, `${name}-${size.name}-${theme}.png`) });
          }
        };
        const channels = await (await call(server, 'GET', '/service/rest/internal/ui/notifications')).json();
        if ((channels as unknown[]).length === 0) {
          await hosted(server, 'raw', 'releases', { raw: { contentDisposition: 'ATTACHMENT' } });
          await ensure(
            await call(server, 'POST', '/service/rest/internal/ui/notifications', {
              name: 'Release announcements',
              type: 'webhook',
              secret: 'https://hooks.example.com/astrum',
              events: ['published', 'deleted'],
              repositories: ['releases'],
              enabled: false,
              proxyMode: 'custom',
              proxyType: 'http',
              proxyHost: 'proxy.example.com',
              proxyPort: 3128,
            }),
            'channel',
          );
        }
        await page.setViewportSize({ width: size.width, height: size.height });
        await page.addInitScript(
          (value) => window.localStorage.setItem('astrum-theme-preference', value),
          theme,
        );
        await signInAsAdmin(page, server.adminPassword);
        await page.goto('/admin/system/notifications');
        await expect(page.getByText('Release announcements')).toBeVisible();
        await shoot('notifications');
        await page.getByRole('button', { name: 'Edit' }).click();
        await expect(page.getByRole('dialog')).toBeVisible();
        await shoot('notification-dialog');
        await page.goto('/admin/system/capabilities');
        await expect(page.getByRole('link', { name: 'UI: Settings' })).toBeVisible();
        await shoot('capabilities');
        await page.goto('/admin/system/capabilities/rapture.settings?tab=settings');
        await expect(page.getByLabel('Title')).toBeVisible();
        await shoot('capability-settings');
        await page.goto('/admin/system/capabilities/create');
        await expect(page.getByText(/Types available/)).toBeVisible();
        await shoot('capability-create');
        await page.goto('/admin/system/nodes');
        await expect(page.getByText('Current node')).toBeVisible();
        await shoot('nodes');
      });
    }
  }
});
