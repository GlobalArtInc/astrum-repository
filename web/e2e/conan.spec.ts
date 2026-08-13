import { mkdirSync } from 'node:fs';
import { join } from 'node:path';
import type { Page } from '@playwright/test';
import { call } from './content';
import { expect, noHorizontalScroll, type Server, signIn, test } from './fixtures';

test.describe.configure({ timeout: 120_000 });

const DIR = process.env.ASTRUM_SHOTS_DIR;
const RREV = '30918f2669fd83ad91770cc857ca5f69';
const PKG = '63fead0844576fc02943e16909f08fcdddd6f44b';
const PREV = '62fca0bfb7d150a008f36f90514fb34f';

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
    await call(server, 'POST', '/service/rest/v1/repositories/conan/hosted', {
      name,
      online: true,
      storage: { blobStoreName: 'default', strictContentTypeValidation: true, writePolicy: 'ALLOW_ONCE' },
    }),
    'create',
  );
  const recipe = `/repository/${name}/v2/conans/hello/1.0.0/_/_/revisions/${RREV}`;
  const files: Array<[string, string]> = [
    [`${recipe}/files/conanfile.py`, 'from conan import ConanFile\n'],
    [`${recipe}/files/conanmanifest.txt`, '123\n'],
    [`${recipe}/packages/${PKG}/revisions/${PREV}/files/conaninfo.txt`, '[settings]\nos=Linux\n'],
    [`${recipe}/packages/${PKG}/revisions/${PREV}/files/conanmanifest.txt`, '123\n'],
  ];
  for (const [path, body] of files) {
    await expectOk(
      await call(server, 'PUT', path, Buffer.from(body), { 'Content-Type': 'application/octet-stream' }),
      path,
    );
  }
}

test('a Conan proxy is made through the form with its protocol version', async ({ page, server }) => {
  await signInAsAdmin(page, server);
  await page.goto('/admin/repository/repositories/create/conan/proxy');
  await expect(page.getByText('The Conan protocol of the remote')).toBeVisible();
  await expect(page.getByText(/https:\/\/center2\.conan\.io/)).toBeVisible();
  await page.getByRole('textbox', { name: 'Name', exact: true }).fill('center');
  await page.getByRole('textbox', { name: /Remote Storage/ }).fill('https://center2.conan.io');
  await page.getByRole('button', { name: 'Continue' }).click();
  await expect(page).toHaveURL(/\/admin\/repository\/repositories$/);
  const stored = await (await call(server, 'GET', '/service/rest/v1/repositories/conan/proxy/center')).json();
  expect(stored.conanProxy.conanVersion).toBe('V2');
  await page.goto('/admin/repository/repositories/center');
  await expect(page.getByRole('combobox', { name: /Protocol version/ })).toBeDisabled();
});

test('uploaded recipes show in browse without the empty user and channel', async ({ page, server }) => {
  await hosted(server, 'conan');
  await signInAsAdmin(page, server);
  await page.goto('/browse/conan');
  const tree = page.getByRole('tree');
  await tree.getByText('hello', { exact: true }).click();
  await tree.getByText('1.0.0', { exact: true }).click();
  await tree.getByText(RREV, { exact: true }).click();
  await tree.getByText('export', { exact: true }).click();
  await tree.getByText('conanfile.py', { exact: true }).click();
  await expect(page).toHaveURL(new RegExp(`/browse/conan/hello/1\\.0\\.0/${RREV}/export/conanfile\\.py$`));
  const url = `${server.url}/repository/conan/`;
  await expect(page.getByRole('tab', { name: 'conan remote' })).toHaveAttribute('aria-selected', 'true');
  await expect(page.getByText(`conan remote add conan ${url}`)).toBeVisible();
  await page.getByRole('tab', { name: 'conan install' }).click();
  await expect(page.getByText('conan install --requires=hello/1.0.0 -r conan')).toBeVisible();
  await page.getByRole('tab', { name: 'conan upload' }).click();
  await expect(page.getByText('conan upload hello/1.0.0 -r conan')).toBeVisible();

  await page.setViewportSize({ width: 390, height: 844 });
  await page.reload();
  await expect(page.getByRole('tab', { name: 'conan remote' })).toBeVisible();
  await noHorizontalScroll(page);
});

async function shoot(page: Page, name: string) {
  if (DIR) {
    mkdirSync(DIR, { recursive: true });
    await page.waitForTimeout(300);
    await page.screenshot({ path: join(DIR, `${name}.png`) });
  }
}

test('conan screens', async ({ page, server }) => {
  test.skip(!DIR, 'set ASTRUM_SHOTS_DIR to take screenshots');
  test.setTimeout(240_000);
  await hosted(server, 'conan-shots');
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
      await shotPage.goto('/admin/repository/repositories/create/conan/proxy');
      await expect(shotPage.getByRole('textbox', { name: 'Name', exact: true })).toBeVisible();
      await shoot(shotPage, `conan-create-proxy-${suffix}`);
      await shotPage.goto(`/browse/conan-shots/hello/1.0.0/${RREV}/export/conanfile.py`);
      await expect(shotPage.getByRole('tab', { name: 'conan remote' })).toBeVisible();
      await shoot(shotPage, `conan-browse-file-${suffix}`);
      await context.close();
    }
  }
});
