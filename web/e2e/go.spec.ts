import { mkdirSync } from 'node:fs';
import { join } from 'node:path';
import type { Page } from '@playwright/test';
import { call, zip } from './content';
import { expect, noHorizontalScroll, type Server, signIn, test } from './fixtures';

test.describe.configure({ timeout: 120_000 });

const DIR = process.env.ASTRUM_SHOTS_DIR;
const FILE = /^File\*?$/;

function moduleZip(module: string, version: string): Buffer {
  return zip([
    [`${module}@${version}/go.mod`, `module ${module}\n\ngo 1.22\n`],
    [`${module}@${version}/hello.go`, 'package hello\n'],
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
    await call(server, 'POST', '/service/rest/v1/repositories/go/hosted', {
      name,
      online: true,
      storage: { blobStoreName: 'default', strictContentTypeValidation: true, writePolicy: 'ALLOW_ONCE' },
    }),
    'create',
  );
}

test('a Go module goes up through the upload page and shows in browse with its usage', async ({
  page,
  server,
}) => {
  await hosted(server, 'modules');
  await signInAsAdmin(page, server);
  await page.goto('/upload/modules');
  await expect(page.getByRole('heading', { level: 1, name: 'Upload to modules' })).toBeVisible();
  await page
    .getByLabel(FILE)
    .and(page.locator('input[type=file]'))
    .setInputFiles({
      name: 'hello.zip',
      mimeType: 'application/zip',
      buffer: moduleZip('example.com/Astrum/hello', 'v1.0.0'),
    });
  await page.getByRole('button', { name: 'Upload Component' }).click();
  await expect(page).toHaveURL(/\/upload$/);
  await expect(page.getByText('Component uploaded to modules').first()).toBeVisible();
  const list = await (
    await call(server, 'GET', '/repository/modules/example.com/!astrum/hello/@v/list')
  ).text();
  expect(list).toBe('v1.0.0\n');

  await page.goto('/browse/modules');
  const tree = page.getByRole('tree');
  await tree.getByText('example.com', { exact: true }).click();
  await tree.getByText('!astrum', { exact: true }).click();
  await tree.getByText('hello', { exact: true }).click();
  await tree.getByText('v1.0.0', { exact: true }).click();
  await tree.getByText('v1.0.0.zip', { exact: true }).click();
  const url = `${server.url}/repository/modules`;
  await expect(page.getByRole('tab', { name: 'go get' })).toBeVisible();
  await page.getByRole('tab', { name: 'go get' }).click();
  await expect(page.getByText(`GOPROXY=${url} go get example.com/Astrum/hello@v1.0.0`)).toBeVisible();
  await page.getByRole('tab', { name: 'go env' }).click();
  await expect(page.getByText('go env -w GONOSUMDB=example.com/Astrum/hello')).toBeVisible();
  await page.getByRole('tab', { name: 'go.mod' }).click();
  await expect(page.getByText('require example.com/Astrum/hello v1.0.0')).toBeVisible();
  await page.getByRole('tab', { name: 'Upload' }).click();
  await expect(
    page.getByText(
      /^curl -u <user> --upload-file v1\.0\.0\.zip .*\/example\.com\/!astrum\/hello\/@v\/v1\.0\.0\.zip$/,
    ),
  ).toBeVisible();

  await page.setViewportSize({ width: 390, height: 844 });
  await page.reload();
  await expect(page.getByRole('tab', { name: 'go get' })).toBeVisible();
  await noHorizontalScroll(page);
});

test('a Go proxy is made through the form', async ({ page, server }) => {
  await signInAsAdmin(page, server);
  await page.goto('/admin/repository/repositories/create/go/proxy');
  await expect(
    page.getByText(
      /e\.g\., https:\/\/example\.com\/repository\/|for example, https:\/\/example\.com\/repository\//,
    ),
  ).toBeVisible();
  await page.getByRole('textbox', { name: 'Name', exact: true }).fill('golang');
  await page.getByRole('textbox', { name: /Remote Storage|Remote URL/ }).fill('https://proxy.golang.org');
  await page.getByRole('button', { name: 'Continue' }).click();
  await expect(page).toHaveURL(/\/admin\/repository\/repositories$/);
  const stored = await (await call(server, 'GET', '/service/rest/v1/repositories/go/proxy/golang')).json();
  expect(stored.proxy.remoteUrl).toBe('https://proxy.golang.org');
});

async function shoot(page: Page, name: string) {
  if (DIR) {
    mkdirSync(DIR, { recursive: true });
    await page.waitForTimeout(300);
    await page.screenshot({ path: join(DIR, `${name}.png`) });
  }
}

test('go screens', async ({ page, server }) => {
  test.skip(!DIR, 'set ASTRUM_SHOTS_DIR to take screenshots');
  test.setTimeout(240_000);
  await hosted(server, 'modules-shots');
  await expectOk(
    await call(
      server,
      'PUT',
      '/repository/modules-shots/example.com/hello/@v/v1.0.0.zip',
      moduleZip('example.com/hello', 'v1.0.0'),
      { 'Content-Type': 'application/zip' },
    ),
    'put',
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
      await shotPage.goto('/upload/modules-shots');
      await expect(shotPage.getByLabel(FILE).and(shotPage.locator('input[type=file]'))).toBeAttached();
      await shoot(shotPage, `go-upload-${suffix}`);
      await shotPage.goto('/browse/modules-shots/example.com/hello/v1.0.0/v1.0.0.zip');
      await expect(shotPage.getByRole('tab', { name: 'go get' })).toBeVisible();
      await shoot(shotPage, `go-browse-zip-${suffix}`);
      await context.close();
    }
  }
});
