import { mkdirSync } from 'node:fs';
import { join } from 'node:path';
import { gzipSync } from 'node:zlib';
import type { Page } from '@playwright/test';
import { call, tar } from './content';
import { expect, noHorizontalScroll, type Server, signIn, test } from './fixtures';

test.describe.configure({ timeout: 120_000 });

const DIR = process.env.ASTRUM_SHOTS_DIR;
const FILE = /^File\*?$/;

function gem(name: string, version: string, platform = 'ruby'): Buffer {
  const spec = [
    '--- !ruby/object:Gem::Specification',
    `name: ${name}`,
    'version: !ruby/object:Gem::Version',
    `  version: ${version}`,
    `platform: ${platform}`,
    'authors:',
    '- Astrum Tests',
    'date: 2026-10-17 00:00:00.000000000 Z',
    'dependencies: []',
    `description: The ${name} gem of the end-to-end tests`,
    'licenses:',
    '- MIT',
    'rubygems_version: 3.5.22',
    'specification_version: 4',
    `summary: ${name} summary`,
    '',
  ].join('\n');
  const data = tar([[`lib/${name}.rb`, `puts '${name} ${version}'\n`]]);
  return tar([
    ['metadata.gz', gzipSync(Buffer.from(spec, 'utf8'))],
    ['data.tar.gz', gzipSync(data)],
  ]);
}

function fileInput(page: Page) {
  return page.getByLabel(FILE).and(page.locator('input[type=file]'));
}

async function expectOk(response: Response, what: string) {
  expect(response.ok, `${what}: ${response.status} ${await response.text()}`).toBe(true);
}

async function signInAsAdmin(page: Page, server: Server) {
  await page.goto('/login');
  await signIn(page, server.adminPassword);
  await expect(page.getByRole('button', { name: /Account menu of/ })).toBeVisible();
}

async function hosted(server: Server, name: string) {
  await expectOk(
    await call(server, 'POST', '/service/rest/v1/repositories/rubygems/hosted', {
      name,
      online: true,
      storage: { blobStoreName: 'default', strictContentTypeValidation: true, writePolicy: 'ALLOW_ONCE' },
    }),
    'create',
  );
}

async function push(server: Server, repository: string, data: Buffer) {
  await expectOk(await call(server, 'POST', `/repository/${repository}/api/v1/gems`, data), 'push');
}

test('a hosted RubyGems repository is made through the form', async ({ page, server }) => {
  await signInAsAdmin(page, server);
  await page.goto('/admin/repository/repositories');
  await page.getByRole('link', { name: 'Create Repository' }).first().click();
  await expect(page.getByText('The standard package manager for Ruby.')).toBeHidden();
  await page.getByRole('button', { name: 'RubyGems', exact: true }).click();
  await expect(page.getByText('The standard package manager for Ruby.')).toBeVisible();
  await page.getByRole('button', { name: 'Create a hosted RubyGems repository' }).click();
  await page.getByRole('textbox', { name: 'Name', exact: true }).fill('gems');
  await page.getByRole('button', { name: 'Continue' }).click();
  await expect(page).toHaveURL(/\/admin\/repository\/repositories$/);
  const stored = await (
    await call(server, 'GET', '/service/rest/v1/repositories/rubygems/hosted/gems')
  ).json();
  expect(stored.format).toBe('rubygems');
  expect(stored.storage.writePolicy).toBe('ALLOW_ONCE');
});

test('a gem goes up through the upload page and shows in browse with its usage', async ({ page, server }) => {
  await hosted(server, 'gems-upload');
  await signInAsAdmin(page, server);
  await page.goto('/upload/gems-upload');
  await expect(page.getByRole('heading', { level: 1, name: 'Upload to gems-upload' })).toBeVisible();
  await fileInput(page).setInputFiles({
    name: 'astrum-hola-1.0.0.gem',
    mimeType: 'application/octet-stream',
    buffer: gem('astrum-hola', '1.0.0'),
  });
  await page.getByRole('button', { name: 'Upload Component' }).click();
  await expect(page).toHaveURL(/\/upload$/);
  await expect(page.getByText('Component uploaded to gems-upload').first()).toBeVisible();
  const versions = await (await call(server, 'GET', '/repository/gems-upload/versions')).text();
  expect(versions).toContain('astrum-hola 1.0.0 ');

  await page.goto('/browse/gems-upload');
  const tree = page.getByRole('tree');
  await expect(tree.getByText('versions', { exact: true })).toBeVisible();
  await tree.getByText('astrum-hola', { exact: true }).click();
  await tree.getByText('1.0.0', { exact: true }).click();
  await tree.getByText('astrum-hola-1.0.0.gem').click();
  await expect(page).toHaveURL(/\/browse\/gems-upload\/astrum-hola\/1\.0\.0\/astrum-hola-1\.0\.0\.gem$/);
  const repositoryUrl = `${server.url}/repository/gems-upload/`;
  await expect(page.getByRole('tab', { name: 'gem install' })).toHaveAttribute('aria-selected', 'true');
  await expect(
    page.getByText(`gem install astrum-hola -v 1.0.0 --clear-sources --source ${repositoryUrl}`),
  ).toBeVisible();
  await page.getByRole('tab', { name: 'Gemfile' }).click();
  await expect(page.getByText("Put this at the top of the project's Gemfile")).toBeVisible();
  await page.getByRole('tab', { name: 'gem push' }).click();
  await expect(page.getByText(/gem push astrum-hola-1\.0\.0\.gem/)).toBeVisible();
  await expect(page.getByText(/sign in once as a user of this server to get an API key/)).toBeVisible();

  await page.setViewportSize({ width: 390, height: 844 });
  await page.reload();
  await expect(page.getByRole('tab', { name: 'gem install' })).toBeVisible();
  await noHorizontalScroll(page);
});

test('the component page gives the install command and the search knows gem platforms', async ({
  page,
  server,
}) => {
  await hosted(server, 'gems-search');
  await push(server, 'gems-search', gem('astrum-native', '2.1.0', 'x86_64-linux'));
  await push(server, 'gems-search', gem('astrum-plain', '1.0.0'));
  await signInAsAdmin(page, server);
  await page.goto(`/search/component/${encodeURIComponent('rubygems::astrum-native')}?version=2.1.0`);
  await expect(page.getByText('gem install astrum-native -v 2.1.0')).toBeVisible();
  await page.goto('/search/custom?rubygems.platform=x86_64-linux&repository=gems-search');
  const found = page.getByRole('table').getByRole('button', { name: /^Open .* in the repository$/ });
  await expect(found).toHaveCount(1);
  await expect(found.first()).toContainText('astrum-native');
});

async function shoot(page: Page, name: string) {
  if (DIR) {
    mkdirSync(DIR, { recursive: true });
    await page.waitForTimeout(300);
    await page.screenshot({ path: join(DIR, `${name}.png`) });
  }
}

test('rubygems screens', async ({ page, server }) => {
  test.skip(!DIR, 'set ASTRUM_SHOTS_DIR to take screenshots');
  test.setTimeout(240_000);
  await hosted(server, 'gems-shots');
  await push(server, 'gems-shots', gem('astrum-hola', '1.0.0'));
  for (const size of [
    { name: '1440', width: 1440, height: 900 },
    { name: '390', width: 390, height: 844 },
  ]) {
    for (const theme of ['dark', 'light']) {
      const context = await page
        .context()
        .browser()
        ?.newContext({ baseURL: server.url, viewport: { width: size.width, height: size.height } });
      if (!context) {
        throw new Error('no browser');
      }
      await context.addInitScript(
        (value) => window.localStorage.setItem('astrum-theme-preference', value),
        theme,
      );
      const shotPage = await context.newPage();
      await signInAsAdmin(shotPage, server);
      const suffix = `${size.name}-${theme}`;
      await shotPage.goto('/admin/repository/repositories/create/rubygems/hosted');
      await expect(shotPage.getByRole('textbox', { name: 'Name', exact: true })).toBeVisible();
      await shoot(shotPage, `rubygems-create-hosted-${suffix}`);
      await shotPage.goto('/upload/gems-shots');
      await expect(fileInput(shotPage)).toBeAttached();
      await shoot(shotPage, `rubygems-upload-${suffix}`);
      await shotPage.goto('/browse/gems-shots/astrum-hola/1.0.0/astrum-hola-1.0.0.gem');
      await expect(shotPage.getByRole('tab', { name: 'gem install' })).toBeVisible();
      await shoot(shotPage, `rubygems-browse-gem-${suffix}`);
      await shotPage.goto(`/search/component/${encodeURIComponent('rubygems::astrum-hola')}?version=1.0.0`);
      await expect(shotPage.getByText('gem install astrum-hola -v 1.0.0')).toBeVisible();
      await shoot(shotPage, `rubygems-component-${suffix}`);
      await context.close();
    }
  }
});
