import { mkdirSync } from 'node:fs';
import { join } from 'node:path';
import { zstdCompressSync } from 'node:zlib';
import type { Page } from '@playwright/test';
import { call, tar, zip } from './content';
import { expect, noHorizontalScroll, type Server, signIn, test } from './fixtures';

test.describe.configure({ timeout: 120_000 });

const DIR = process.env.ASTRUM_SHOTS_DIR;
const FILE = /^File\*?$/;

function condaPackage(name: string, version: string, build: string, subdir: string): Buffer {
  const stem = `${name}-${version}-${build}`;
  const index = {
    name,
    version,
    build,
    build_number: 0,
    depends: [],
    license: 'MIT',
    subdir,
    ...(subdir === 'noarch' ? { noarch: 'generic' } : {}),
  };
  const file = `share/${name}/README.txt`;
  return zip([
    ['metadata.json', '{"conda_pkg_format_version": 2}'],
    [`pkg-${stem}.tar.zst`, zstdCompressSync(tar([[file, `${name} of the end-to-end tests\n`]]))],
    [
      `info-${stem}.tar.zst`,
      zstdCompressSync(
        tar([
          ['info/index.json', JSON.stringify(index)],
          ['info/about.json', JSON.stringify({ summary: `${name} of the end-to-end tests` })],
        ]),
      ),
    ],
  ]);
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
    await call(server, 'POST', '/service/rest/v1/repositories/conda/hosted', {
      name,
      online: true,
      storage: { blobStoreName: 'default', strictContentTypeValidation: true, writePolicy: 'ALLOW_ONCE' },
    }),
    'create',
  );
}

test('a conda hosted repository is made through the form, takes an upload and shows its usage', async ({
  page,
  server,
}) => {
  await signInAsAdmin(page, server);
  await page.goto('/admin/repository/repositories');
  await page.getByRole('link', { name: 'Create Repository' }).first().click();
  await page.getByRole('button', { name: 'Conda', exact: true }).click();
  await page.getByRole('button', { name: 'Create a hosted Conda repository' }).click();
  await page.getByRole('textbox', { name: 'Name', exact: true }).fill('conda-hosted');
  await page.getByRole('button', { name: 'Continue' }).click();
  await expect(page).toHaveURL(/\/admin\/repository\/repositories$/);

  await page.goto('/upload/conda-hosted');
  await page.getByRole('textbox', { name: /^Channel/ }).fill('main');
  await page
    .getByLabel(FILE)
    .and(page.locator('input[type=file]'))
    .setInputFiles({
      name: 'hello-1.0-py_0.conda',
      mimeType: 'application/octet-stream',
      buffer: condaPackage('hello', '1.0', 'py_0', 'noarch'),
    });
  await page.getByRole('button', { name: 'Upload Component' }).click();
  await expect(page).toHaveURL(/\/upload$/);
  const repodata = await (
    await call(server, 'GET', '/repository/conda-hosted/main/noarch/repodata.json')
  ).json();
  expect(Object.keys(repodata['packages.conda'])).toEqual(['hello-1.0-py_0.conda']);

  await page.goto('/browse/conda-hosted');
  const tree = page.getByRole('tree');
  await tree.getByText('main', { exact: true }).click();
  await tree.getByText('noarch', { exact: true }).click();
  await expect(tree.getByText('repodata.json', { exact: true })).toBeVisible();
  await tree.getByText('hello-1.0-py_0.conda', { exact: true }).click();
  const channel = `${server.url}/repository/conda-hosted/main`;
  await expect(page.getByRole('tab', { name: 'conda install' })).toHaveAttribute('aria-selected', 'true');
  await expect(page.getByText(`conda install -c ${channel} --override-channels hello=1.0`)).toBeVisible();
  await page.getByRole('tab', { name: '.condarc' }).click();
  await expect(
    page.getByText('Save it as ~/.condarc to install packages from this channel by default'),
  ).toBeVisible();
  await page.getByRole('tab', { name: 'Upload' }).click();
  await expect(
    page.getByText(
      `curl -u <user> --upload-file hello-1.0-py_0.conda ${channel}/noarch/hello-1.0-py_0.conda`,
    ),
  ).toBeVisible();

  await page.goto(`/search/component/${encodeURIComponent('conda:main:hello')}?version=1.0`);
  await expect(page.getByText(`conda install -c ${channel} hello=1.0`)).toBeVisible();

  await page.goto('/search/custom?conda.subdir=noarch&repository=conda-hosted');
  const found = page.getByRole('table').getByRole('button', { name: /^Open .* in the repository$/ });
  await expect(found).toHaveCount(1);
  await expect(found.first()).toContainText('hello');

  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/browse/conda-hosted/main/noarch/hello-1.0-py_0.conda');
  await expect(page.getByRole('tab', { name: 'conda install' })).toBeVisible();
  await noHorizontalScroll(page);
});

test('a conda proxy is made through the form', async ({ page, server }) => {
  await signInAsAdmin(page, server);
  await page.goto('/admin/repository/repositories/create/conda/proxy');
  await expect(page.getByText(/https:\/\/conda\.anaconda\.org\/conda-forge\//)).toBeVisible();
  await page.getByRole('textbox', { name: 'Name', exact: true }).fill('conda-forge');
  await page.getByRole('textbox', { name: /Remote Storage/ }).fill('https://conda.anaconda.org/conda-forge/');
  await page.getByRole('button', { name: 'Continue' }).click();
  await expect(page).toHaveURL(/\/admin\/repository\/repositories$/);
  const stored = await (
    await call(server, 'GET', '/service/rest/v1/repositories/conda/proxy/conda-forge')
  ).json();
  expect(stored.proxy.remoteUrl).toBe('https://conda.anaconda.org/conda-forge/');
});

async function shoot(page: Page, name: string) {
  if (DIR) {
    mkdirSync(DIR, { recursive: true });
    await page.waitForTimeout(300);
    await page.screenshot({ path: join(DIR, `${name}.png`) });
  }
}

test('conda screens', async ({ page, server }) => {
  test.skip(!DIR, 'set ASTRUM_SHOTS_DIR to take screenshots');
  test.setTimeout(240_000);
  await hosted(server, 'conda-shots');
  await expectOk(
    await call(
      server,
      'PUT',
      '/repository/conda-shots/main/linux-64/tool-2.0-h1_0.conda',
      condaPackage('tool', '2.0', 'h1_0', 'linux-64'),
      { 'Content-Type': 'application/octet-stream' },
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
      await shotPage.goto('/upload/conda-shots');
      await expect(shotPage.getByLabel(FILE).and(shotPage.locator('input[type=file]'))).toBeAttached();
      await shoot(shotPage, `conda-upload-${suffix}`);
      await shotPage.goto('/browse/conda-shots/main/linux-64/tool-2.0-h1_0.conda');
      await expect(shotPage.getByRole('tab', { name: 'conda install' })).toBeVisible();
      await shoot(shotPage, `conda-browse-package-${suffix}`);
      await shotPage.goto('/admin/repository/repositories/create/conda/proxy');
      await expect(shotPage.getByRole('textbox', { name: 'Name', exact: true })).toBeVisible();
      await shoot(shotPage, `conda-create-proxy-${suffix}`);
      await context.close();
    }
  }
});
