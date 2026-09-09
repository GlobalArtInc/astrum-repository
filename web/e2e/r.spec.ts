import { mkdirSync } from 'node:fs';
import { join } from 'node:path';
import { gunzipSync, gzipSync } from 'node:zlib';
import type { Page } from '@playwright/test';
import { call, tar } from './content';
import { expect, noHorizontalScroll, type Server, signIn, test } from './fixtures';

test.describe.configure({ timeout: 120_000 });

const DIR = process.env.ASTRUM_SHOTS_DIR;
const FILE = /^File\*?$/;

function sourcePackage(name: string, version: string): Buffer {
  const description = [
    `Package: ${name}`,
    'Type: Package',
    'Title: A Package of the End-to-End Tests',
    `Version: ${version}`,
    'Author: Astrum Tests',
    'Maintainer: Astrum Tests <e2e@example.org>',
    'Description: Says hello from an R repository.',
    'License: MIT + file LICENSE',
    'Depends: R (>= 3.5.0)',
    'Imports: utils',
    'NeedsCompilation: no',
    '',
  ].join('\n');
  return gzipSync(
    tar([
      [`${name}/DESCRIPTION`, description],
      [`${name}/NAMESPACE`, 'export(hello)\n'],
      [`${name}/R/hello.R`, 'hello <- function() "hello"\n'],
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
    await call(server, 'POST', '/service/rest/v1/repositories/r/hosted', {
      name,
      online: true,
      storage: { blobStoreName: 'default', strictContentTypeValidation: true, writePolicy: 'ALLOW_ONCE' },
    }),
    'create',
  );
}

test('an R package goes up through the upload page and shows in browse with its usage', async ({
  page,
  server,
}) => {
  await hosted(server, 'r-hosted');
  await signInAsAdmin(page, server);
  await page.goto('/upload/r-hosted');
  await expect(page.getByRole('textbox', { name: /^Package Path/ })).toHaveAttribute(
    'placeholder',
    'src/contrib',
  );
  await page
    .getByLabel(FILE)
    .and(page.locator('input[type=file]'))
    .setInputFiles({
      name: 'astrumhello_1.1.0.tar.gz',
      mimeType: 'application/gzip',
      buffer: sourcePackage('astrumhello', '1.1.0'),
    });
  await page.getByRole('button', { name: 'Upload Component' }).click();
  await expect(page).toHaveURL(/\/upload$/);
  const index = await (await call(server, 'GET', '/repository/r-hosted/src/contrib/PACKAGES')).text();
  expect(index).toContain('Package: astrumhello\nVersion: 1.1.0\nDepends: R (>= 3.5.0)\nImports: utils\n');
  const packed = await call(server, 'GET', '/repository/r-hosted/src/contrib/PACKAGES.gz');
  expect(gunzipSync(Buffer.from(await packed.arrayBuffer())).toString('utf8')).toBe(index);

  await page.goto('/browse/r-hosted');
  const tree = page.getByRole('tree');
  await tree.getByText('src', { exact: true }).click();
  await tree.getByText('contrib', { exact: true }).click();
  await expect(tree.getByText('PACKAGES.rds', { exact: true })).toBeVisible();
  await tree.getByText('astrumhello_1.1.0.tar.gz').click();
  const url = `${server.url}/repository/r-hosted`;
  await expect(page.getByRole('tab', { name: 'install.packages' })).toHaveAttribute('aria-selected', 'true');
  await expect(
    page.getByText(`install.packages("astrumhello", repos = c("${url}", getOption("repos")))`),
  ).toBeVisible();
  await page.getByRole('tab', { name: '.Rprofile' }).click();
  await expect(page.getByText(`options(repos = c("r-hosted" = "${url}", getOption("repos")))`)).toBeVisible();
  await page.getByRole('tab', { name: 'DESCRIPTION' }).click();
  await expect(page.getByText('Imports: astrumhello (>= 1.1.0)')).toBeVisible();
  await page.getByRole('tab', { name: 'Upload' }).click();
  await expect(
    page.getByText(
      `curl -u <user> --upload-file astrumhello_1.1.0.tar.gz ${url}/src/contrib/astrumhello_1.1.0.tar.gz`,
    ),
  ).toBeVisible();
  await expect(page.getByText('A Package of the End-to-End Tests')).toBeVisible();

  await page.goto(`/search/component/${encodeURIComponent('r::astrumhello')}?version=1.1.0`);
  await expect(page.getByText(`install.packages("astrumhello", repos = "${url}")`)).toBeVisible();

  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/browse/r-hosted/src/contrib/astrumhello_1.1.0.tar.gz');
  await expect(page.getByRole('tab', { name: 'install.packages' })).toBeVisible();
  await noHorizontalScroll(page);
});

test('an R proxy and an R group are made through the form', async ({ page, server }) => {
  await hosted(server, 'r-own');
  await signInAsAdmin(page, server);
  await page.goto('/admin/repository/repositories/create');
  await page.getByRole('button', { name: 'R', exact: true }).click();
  await expect(page.getByText('Packages for R, served in the layout of CRAN.')).toBeVisible();
  await page.goto('/admin/repository/repositories/create/r/proxy');
  await expect(page.getByText(/https:\/\/cloud\.r-project\.org\//)).toBeVisible();
  await page.getByRole('textbox', { name: 'Name', exact: true }).fill('cran');
  await page.getByRole('textbox', { name: /Remote Storage/ }).fill('https://cloud.r-project.org/');
  await page.getByRole('button', { name: 'Continue' }).click();
  await expect(page).toHaveURL(/\/admin\/repository\/repositories$/);
  const stored = await (await call(server, 'GET', '/service/rest/v1/repositories/r/proxy/cran')).json();
  expect(stored.proxy.remoteUrl).toBe('https://cloud.r-project.org/');
  await expectOk(
    await call(server, 'POST', '/service/rest/v1/repositories/r/group', {
      name: 'r-all',
      online: true,
      storage: { blobStoreName: 'default', strictContentTypeValidation: true },
      group: { memberNames: ['r-own'] },
    }),
    'group',
  );
  await page.goto('/admin/repository/repositories/r-all');
  await expect(page.getByRole('textbox', { name: 'Name', exact: true })).toHaveValue('r-all');
  const empty = await call(server, 'GET', '/repository/r-all/src/contrib/PACKAGES');
  expect(empty.status).toBe(200);
  expect(await empty.text()).toBe('');
});

async function shoot(page: Page, name: string) {
  if (DIR) {
    mkdirSync(DIR, { recursive: true });
    await page.waitForTimeout(300);
    await page.screenshot({ path: join(DIR, `${name}.png`) });
  }
}

test('r screens', async ({ page, server }) => {
  test.skip(!DIR, 'set ASTRUM_SHOTS_DIR to take screenshots');
  test.setTimeout(240_000);
  await hosted(server, 'r-shots');
  await expectOk(
    await call(
      server,
      'PUT',
      '/repository/r-shots/src/contrib/astrumhello_1.1.0.tar.gz',
      sourcePackage('astrumhello', '1.1.0'),
      { 'Content-Type': 'application/gzip' },
    ),
    'upload',
  );
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
      await shotPage.goto('/upload/r-shots');
      await expect(shotPage.getByLabel(FILE).and(shotPage.locator('input[type=file]'))).toBeAttached();
      await shoot(shotPage, `r-upload-${suffix}`);
      await shotPage.goto('/browse/r-shots/src/contrib/astrumhello_1.1.0.tar.gz');
      await expect(shotPage.getByRole('tab', { name: 'install.packages' })).toBeVisible();
      await shoot(shotPage, `r-browse-package-${suffix}`);
      await shotPage.goto('/admin/repository/repositories/create/r/proxy');
      await expect(shotPage.getByText(/https:\/\/cloud\.r-project\.org\//)).toBeVisible();
      await shoot(shotPage, `r-create-proxy-${suffix}`);
      await context.close();
    }
  }
});
