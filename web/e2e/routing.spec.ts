import { mkdirSync } from 'node:fs';
import { join } from 'node:path';
import type { Page } from '@playwright/test';
import { call, ensure, hosted, put } from './content';
import { expect, type Server, signIn, test } from './fixtures';

test.describe.configure({ timeout: 120_000 });

const SHOTS = process.env.ASTRUM_SHOTS_DIR;
const SIZES = [
  { name: '1440', width: 1440, height: 900 },
  { name: '390', width: 390, height: 844 },
] as const;
const THEMES = ['dark', 'light'] as const;
const BLOCKED_TEXT = 'Routing rules block the requested item from this repository';

async function signInAsAdmin(page: Page, password: string) {
  await page.goto('/login');
  await signIn(page, password);
  await expect(page.getByRole('button', { name: /Account menu of/ })).toBeVisible();
}

async function proxyOf(server: Server, name: string, upstream: string) {
  const existing = await call(server, 'GET', `/service/rest/v1/repositories/${name}`);
  if (existing.ok) {
    return;
  }
  await ensure(
    await call(server, 'POST', '/service/rest/v1/repositories/raw/proxy', {
      name,
      online: true,
      storage: { blobStoreName: 'default', strictContentTypeValidation: false },
      proxy: {
        remoteUrl: `${server.url}/repository/${upstream}/`,
        contentMaxAge: 1440,
        metadataMaxAge: 1440,
      },
      negativeCache: { enabled: false, timeToLive: 1440 },
      httpClient: { blocked: false, autoBlock: false },
      raw: { contentDisposition: 'ATTACHMENT' },
    }),
    `create ${name}`,
  );
}

async function group(server: Server, name: string, members: string[]) {
  const existing = await call(server, 'GET', `/service/rest/v1/repositories/${name}`);
  if (existing.ok) {
    return;
  }
  await ensure(
    await call(server, 'POST', '/service/rest/v1/repositories/raw/group', {
      name,
      online: true,
      storage: { blobStoreName: 'default', strictContentTypeValidation: false },
      group: { memberNames: members },
      raw: { contentDisposition: 'ATTACHMENT' },
    }),
    `create ${name}`,
  );
}

async function seed(server: Server) {
  await ensure(
    await call(server, 'PUT', '/service/rest/v1/security/ssrf-protection', {
      enabled: true,
      allowedIPs: ['127.0.0.1'],
      allowedDomains: [],
    }),
    'allow the loopback upstream',
  );
  await hosted(server, 'raw', 'rr-upstream', { raw: { contentDisposition: 'ATTACHMENT' } });
  await put(server, '/repository/rr-upstream/org/app/app-1.0.jar', 'binary');
  await put(server, '/repository/rr-upstream/org/app/app-1.0-sources.jar', 'sources');
  await proxyOf(server, 'rr-remote', 'rr-upstream');
  await group(server, 'rr-all', ['rr-remote']);
}

test('creates, tests, assigns and previews a routing rule', async ({ page, server }) => {
  await seed(server);
  await server.setAnonymous(true);
  await signInAsAdmin(page, server.adminPassword);

  await page.goto('/settings');
  await page.getByRole('link', { name: /Routing Rules/ }).click();
  await expect(page.getByRole('heading', { level: 1, name: 'Routing Rules' })).toBeVisible();
  await expect(page.getByText('No routing rules')).toBeVisible();

  await page.getByRole('link', { name: 'Create Rule' }).click();
  await expect(page.getByRole('heading', { level: 1, name: 'Create Routing Rule' })).toBeVisible();
  await page.getByRole('textbox', { name: /^Name/ }).fill('no-sources');
  await page.getByRole('textbox', { name: /^Description/ }).fill('No source jars from upstream');
  await page.getByRole('textbox', { name: 'Matcher 1' }).fill('(unclosed');
  await expect(page.getByText('Invalid', { exact: true })).toBeVisible();
  await expect(page.getByText('Invalid regular expression: (unclosed')).toBeVisible();
  await page.getByRole('button', { name: 'Create' }).click();
  await expect(page.getByText(/regular expression/).first()).toBeVisible();
  await expect(page).toHaveURL(/routing-rules\/create$/);
  await page.getByRole('button', { name: 'Remove matcher 1' }).click();
  await page.getByRole('button', { name: 'Add Matcher' }).click();
  await page.getByRole('menuitem', { name: /Ends with/ }).click();
  await page.getByRole('textbox', { name: 'Matcher 1' }).fill('-sources.jar');
  await page.getByRole('textbox', { name: 'Test Path' }).fill('/org/app/app-1.0-sources.jar');
  await expect(page.getByText('This request would be BLOCKED')).toBeVisible();
  await expect(page.getByText('Matches', { exact: true })).toBeVisible();
  await page.getByRole('textbox', { name: 'Test Path' }).fill('org/app/app-1.0.jar');
  await expect(page.getByText('This request would be ALLOWED')).toBeVisible();
  await expect(page.getByText('No match', { exact: true })).toBeVisible();
  await expect(page.getByText('ends with “-sources.jar”')).toBeVisible();
  await page.getByRole('button', { name: 'Create' }).click();
  await expect(page.getByText('Routing rule no-sources created').first()).toBeVisible();
  await expect(page.getByRole('heading', { level: 1, name: 'Routing Rules' })).toBeVisible();
  const row = page.getByRole('row', { name: 'Open no-sources' });
  await expect(row).toContainText('No source jars from upstream');
  await expect(row).toContainText('Block');

  await page.goto('/admin/repository/repositories/rr-remote');
  const select = page.getByRole('combobox', { name: /Routing Rule/ });
  await select.scrollIntoViewIfNeeded();
  await select.click();
  await page.getByRole('option', { name: 'no-sources (BLOCK)' }).click();
  await page.getByRole('button', { name: 'Save Changes', exact: true }).click();
  await expect
    .poll(async () => {
      const document = (await (
        await call(server, 'GET', '/service/rest/v1/repositories/raw/proxy/rr-remote')
      ).json()) as {
        routingRuleName: string | null;
      };
      return document.routingRuleName;
    })
    .toBe('no-sources');

  const blocked = await page.request.get(`${server.url}/repository/rr-remote/org/app/app-1.0-sources.jar`);
  expect(blocked.status()).toBe(403);
  expect(await blocked.text()).toBe(BLOCKED_TEXT);
  const allowed = await page.request.get(`${server.url}/repository/rr-remote/org/app/app-1.0.jar`);
  expect(allowed.status()).toBe(200);
  expect(await allowed.text()).toBe('binary');
  const viaGroup = await page.request.get(`${server.url}/repository/rr-all/org/app/app-1.0-sources.jar`);
  expect(viaGroup.status()).toBe(404);

  await page.goto('/admin/repository/routing-rules');
  await page.getByRole('row', { name: 'Open no-sources' }).click();
  await expect(page.getByText('Mode: Block • 1 matcher • Assigned to 1 repository')).toBeVisible();
  await expect(page.getByRole('button', { name: 'Delete' })).toBeDisabled();
  await expect(page.getByText(/assigned to 1 repository: rr-remote/)).toBeVisible();
  const refused = await call(server, 'DELETE', '/service/rest/v1/routing-rules/no-sources');
  expect(refused.status).toBe(400);
  expect(((await refused.json()) as { message: string }).message).toContain('still in use');
  await page.getByRole('textbox', { name: /^Description/ }).fill('No source or javadoc jars');
  await page.getByRole('button', { name: 'Add Matcher' }).click();
  await page.getByRole('menuitem', { name: /Ends with/ }).click();
  await page.getByRole('textbox', { name: 'Matcher 2' }).fill('-javadoc.jar');
  await page.getByRole('button', { name: 'Save' }).click();
  await expect(page.getByText('Routing rule no-sources updated').first()).toBeVisible();
  await expect(page.getByRole('row', { name: 'Open no-sources' })).toContainText('No source or javadoc jars');

  await page.getByRole('link', { name: 'Preview' }).click();
  await expect(page.getByRole('heading', { level: 1, name: 'Global Routing Preview' })).toBeVisible();
  await page.getByRole('textbox', { name: 'Test Path' }).fill('/org/app/app-1.0-javadoc.jar');
  await page.getByRole('button', { name: 'Test' }).click();
  const tree = page.getByRole('list', { name: 'Global Routing Preview' });
  await expect(tree.getByRole('listitem', { name: 'rr-all' })).toBeVisible();
  await expect(tree.getByRole('listitem', { name: 'rr-remote' })).toHaveCount(2);
  await expect(tree.getByText('Blocked')).toHaveCount(2);
  await page.getByRole('button', { name: 'Open the routing rule no-sources' }).first().click();
  const dialog = page.getByRole('dialog', { name: 'no-sources' });
  await expect(dialog.getByText('/org/app/app-1.0-javadoc.jar')).toBeVisible();
  await expect(dialog.getByText('.*-javadoc\\.jar')).toBeVisible();
  await dialog.getByRole('button', { name: 'Close' }).last().click();

  await page.goto('/admin/repository/repositories/rr-remote');
  const again = page.getByRole('combobox', { name: /Routing Rule/ });
  await again.scrollIntoViewIfNeeded();
  await again.click();
  await page.getByRole('option', { name: 'None (allow every request)' }).click();
  await page.getByRole('button', { name: 'Save Changes', exact: true }).click();
  await expect
    .poll(async () =>
      (await call(server, 'GET', '/service/rest/internal/ui/routing-rules/no-sources')).json(),
    )
    .toMatchObject({ assignedRepositoryCount: 0 });
  await page.goto('/admin/repository/routing-rules/no-sources');
  await page.getByRole('button', { name: 'Delete' }).click();
  await page.getByRole('alertdialog').getByRole('button', { name: 'Delete' }).click();
  await expect(page.getByText('Routing rule no-sources deleted').first()).toBeVisible();
  await expect(page.getByText('No routing rules')).toBeVisible();
});

test.describe('screenshots', () => {
  test.skip(!SHOTS, 'set ASTRUM_SHOTS_DIR to take screenshots');

  for (const size of SIZES) {
    for (const theme of THEMES) {
      test(`routing rule screens at ${size.name} in ${theme}`, async ({ page, server }) => {
        const shoot = async (name: string) => {
          if (SHOTS) {
            mkdirSync(SHOTS, { recursive: true });
            await page.waitForTimeout(400);
            await page.screenshot({ path: join(SHOTS, `${name}-${size.name}-${theme}.png`) });
          }
        };
        await seed(server);
        for (const [name, description, mode, matchers] of [
          ['block-sources', 'No source jars', 'BLOCK', ['^/blocked/.*', '.*/secret/.*']],
          ['only-org', '', 'ALLOW', ['^/org/.*']],
        ] as const) {
          const existing = await call(server, 'GET', `/service/rest/v1/routing-rules/${name}`);
          if (!existing.ok) {
            await ensure(
              await call(server, 'POST', '/service/rest/v1/routing-rules', {
                name,
                description,
                mode,
                matchers,
              }),
              name,
            );
          }
        }
        const remote = (await (
          await call(server, 'GET', '/service/rest/v1/repositories/raw/proxy/rr-remote')
        ).json()) as Record<string, unknown>;
        if (remote.routingRuleName !== 'block-sources') {
          await ensure(
            await call(server, 'PUT', '/service/rest/v1/repositories/raw/proxy/rr-remote', {
              ...remote,
              routingRuleName: 'block-sources',
            }),
            'assign',
          );
        }
        await page.setViewportSize({ width: size.width, height: size.height });
        await page.addInitScript(
          (value) => window.localStorage.setItem('astrum-theme-preference', value),
          theme,
        );
        await signInAsAdmin(page, server.adminPassword);
        await page.goto('/admin/repository/routing-rules');
        await expect(page.getByRole('row', { name: 'Open block-sources' })).toBeVisible();
        await shoot('routing-list');
        await page.goto('/admin/repository/routing-rules/create');
        await expect(page.getByRole('heading', { level: 1, name: 'Create Routing Rule' })).toBeVisible();
        await shoot('routing-create');
        await page.goto('/admin/repository/routing-rules/block-sources');
        await expect(page.getByText(/Assigned to 1 repository/)).toBeVisible();
        await shoot('routing-edit');
        await page.getByRole('textbox', { name: 'Test Path' }).fill('/blocked/x.jar');
        await expect(page.getByText('This request would be BLOCKED')).toBeVisible();
        await page.getByRole('textbox', { name: 'Test Path' }).scrollIntoViewIfNeeded();
        await shoot('routing-edit-test');
        await page.goto('/admin/repository/routing-rules/preview');
        await expect(page.getByRole('heading', { level: 1, name: 'Global Routing Preview' })).toBeVisible();
        await shoot('routing-preview');
        await page.getByRole('textbox', { name: 'Test Path' }).fill('/blocked/x.jar');
        await page.getByRole('button', { name: 'Test' }).click();
        await expect(page.getByRole('list', { name: 'Global Routing Preview' })).toBeVisible();
        await shoot('routing-preview-result');
      });
    }
  }
});
