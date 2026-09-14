import { mkdirSync } from 'node:fs';
import { join } from 'node:path';
import { gzipSync } from 'node:zlib';
import type { Page } from '@playwright/test';
import { call, tar } from './content';
import { expect, noHorizontalScroll, type Server, signIn, test } from './fixtures';

test.describe.configure({ timeout: 120_000 });

const DIR = process.env.ASTRUM_SHOTS_DIR;
const FILE = /^File\*?$/;

function dartPackage(name: string, version: string): Buffer {
  const pubspec = [
    `name: ${name}`,
    `version: ${version}`,
    'description: A greeting library of the end-to-end tests of Astrum Repository.',
    'environment:',
    "  sdk: '>=3.0.0 <4.0.0'",
    '',
  ].join('\n');
  return gzipSync(
    tar([
      ['pubspec.yaml', pubspec],
      [`lib/${name}.dart`, `String greet() => 'hello from ${name} ${version}';\n`],
      ['LICENSE', 'MIT\n'],
    ]),
  );
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
    await call(server, 'POST', '/service/rest/v1/repositories/pub/hosted', {
      name,
      online: true,
      storage: { blobStoreName: 'default', strictContentTypeValidation: true, writePolicy: 'ALLOW_ONCE' },
    }),
    'create',
  );
}

async function upload(server: Server, repository: string, archive: Buffer) {
  const form = new FormData();
  form.append('pub.asset', new Blob([new Uint8Array(archive)]), 'package.tar.gz');
  await expectOk(
    await call(server, 'POST', `/service/rest/v1/components?repository=${repository}`, form),
    'upload',
  );
}

test('a pub package goes up through the upload page and shows in browse with its usage', async ({
  page,
  server,
}) => {
  await hosted(server, 'pub-hosted');
  await signInAsAdmin(page, server);
  await page.goto('/upload/pub-hosted');
  await page
    .getByLabel(FILE)
    .and(page.locator('input[type=file]'))
    .setInputFiles({
      name: 'astrum_hello-1.0.0.tar.gz',
      mimeType: 'application/gzip',
      buffer: dartPackage('astrum_hello', '1.0.0'),
    });
  await page.getByRole('button', { name: 'Upload Component' }).click();
  await expect(page).toHaveURL(/\/upload$/);
  const url = `${server.url}/repository/pub-hosted`;
  const listing = await (
    await call(server, 'GET', '/repository/pub-hosted/api/packages/astrum_hello', undefined, {
      Accept: 'application/vnd.pub.v2+json',
    })
  ).json();
  expect(listing.latest.version).toBe('1.0.0');
  expect(listing.latest.archive_url).toBe(`${url}/packages/astrum_hello/versions/1.0.0.tar.gz`);

  await page.goto('/browse/pub-hosted');
  const tree = page.getByRole('tree');
  await tree.getByText('astrum_hello', { exact: true }).click();
  await tree.getByText('1.0.0', { exact: true }).click();
  await tree.getByText('astrum_hello-1.0.0.tar.gz').click();
  await expect(page.getByRole('tab', { name: 'dart pub token add' })).toHaveAttribute(
    'aria-selected',
    'true',
  );
  await expect(page.getByText(`dart pub token add ${url}`, { exact: true })).toBeVisible();
  await page.getByRole('tab', { name: 'pubspec.yaml' }).click();
  await expect(page.getByText(`hosted: ${url}`)).toBeVisible();
  await page.getByRole('tab', { name: 'dart pub add' }).click();
  await expect(page.getByText(`dart pub add 'astrum_hello:{hosted: ${url}, version: ^1.0.0}'`)).toBeVisible();
  await page.getByRole('tab', { name: 'dart pub publish' }).click();
  await expect(page.getByText(`publish_to: ${url}`, { exact: true })).toBeVisible();
  await expect(
    page.getByText('A greeting library of the end-to-end tests of Astrum Repository.'),
  ).toBeVisible();

  await page.goto(`/search/component/${encodeURIComponent('pub::astrum_hello')}?version=1.0.0`);
  await expect(page.getByText(`dart pub add 'astrum_hello:{hosted: ${url}, version: ^1.0.0}'`)).toBeVisible();

  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/browse/pub-hosted/astrum_hello/1.0.0/astrum_hello-1.0.0.tar.gz');
  await expect(page.getByRole('tab', { name: 'dart pub token add' })).toBeVisible();
  await noHorizontalScroll(page);
});

test('a pub proxy and a pub group are made through the form', async ({ page, server }) => {
  await hosted(server, 'pub-own');
  await signInAsAdmin(page, server);
  await page.goto('/admin/repository/repositories/create');
  await page.getByRole('button', { name: 'pub (Dart)', exact: true }).click();
  await expect(
    page.getByText('Packages for Dart and Flutter, served to dart pub and flutter pub.'),
  ).toBeVisible();
  await page.goto('/admin/repository/repositories/create/pub/proxy');
  await expect(page.getByText(/https:\/\/pub\.dev/)).toBeVisible();
  await page.getByRole('textbox', { name: 'Name', exact: true }).fill('pub-dev');
  await page.getByRole('textbox', { name: /Remote Storage/ }).fill('https://pub.dev');
  await page.getByRole('button', { name: 'Continue' }).click();
  await expect(page).toHaveURL(/\/admin\/repository\/repositories$/);
  const stored = await (await call(server, 'GET', '/service/rest/v1/repositories/pub/proxy/pub-dev')).json();
  expect(stored.proxy.remoteUrl).toBe('https://pub.dev');
  await expectOk(
    await call(server, 'POST', '/service/rest/v1/repositories/pub/group', {
      name: 'pub-all',
      online: true,
      storage: { blobStoreName: 'default', strictContentTypeValidation: true },
      group: { memberNames: ['pub-own'] },
    }),
    'group',
  );
  await upload(server, 'pub-own', dartPackage('astrum_group', '0.2.0-dev.1'));
  await page.goto('/admin/repository/repositories/pub-all');
  await expect(page.getByRole('textbox', { name: 'Name', exact: true })).toHaveValue('pub-all');
  const merged = await (
    await call(server, 'GET', '/repository/pub-all/api/packages/astrum_group', undefined, {
      Accept: 'application/vnd.pub.v2+json',
    })
  ).json();
  expect(merged.latest.archive_url).toBe(
    `${server.url}/repository/pub-all/packages/astrum_group/versions/0.2.0-dev.1.tar.gz`,
  );
  await page.goto('/browse/pub-all');
  await page.getByRole('tree').getByText('astrum_group', { exact: true }).click();
  await page.getByRole('tree').getByText('0.2.0-dev.1', { exact: true }).click();
  await page.getByRole('tree').getByText('astrum_group-0.2.0-dev.1.tar.gz').click();
  await page.getByRole('tab', { name: 'PUB_HOSTED_URL' }).click();
  await expect(page.getByText(`export PUB_HOSTED_URL=${server.url}/repository/pub-all`)).toBeVisible();
});

async function shoot(page: Page, name: string) {
  if (DIR) {
    mkdirSync(DIR, { recursive: true });
    await page.waitForTimeout(300);
    await page.screenshot({ path: join(DIR, `${name}.png`) });
  }
}

test('pub screens', async ({ page, server }) => {
  test.skip(!DIR, 'set ASTRUM_SHOTS_DIR to take screenshots');
  test.setTimeout(240_000);
  await hosted(server, 'pub-shots');
  await upload(server, 'pub-shots', dartPackage('astrum_hello', '1.0.0'));
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
      await shotPage.goto('/browse/pub-shots/astrum_hello/1.0.0/astrum_hello-1.0.0.tar.gz');
      await expect(shotPage.getByRole('tab', { name: 'dart pub token add' })).toBeVisible();
      await shoot(shotPage, `pub-browse-package-${suffix}`);
      await shotPage.goto(`/search/component/${encodeURIComponent('pub::astrum_hello')}?version=1.0.0`);
      await expect(shotPage.getByText(/dart pub add 'astrum_hello:/)).toBeVisible();
      await shoot(shotPage, `pub-component-${suffix}`);
      await shotPage.goto('/admin/repository/repositories/create');
      await expect(shotPage.getByRole('button', { name: 'pub (Dart)', exact: true })).toBeVisible();
      await shotPage.getByRole('button', { name: 'pub (Dart)', exact: true }).click();
      await shoot(shotPage, `pub-create-picker-${suffix}`);
      await shotPage.goto('/admin/repository/repositories/create/pub/proxy');
      await expect(shotPage.getByText(/https:\/\/pub\.dev/)).toBeVisible();
      await shoot(shotPage, `pub-create-proxy-${suffix}`);
      await context.close();
    }
  }
});
