import { mkdirSync } from 'node:fs';
import { join } from 'node:path';
import type { Page } from '@playwright/test';
import { call, zip } from './content';
import { expect, noHorizontalScroll, type Server, signIn, test } from './fixtures';

test.describe.configure({ timeout: 120_000 });

const DIR = process.env.ASTRUM_SHOTS_DIR;
const FILE = /^File\*?$/;

function packageZip(name: string): Buffer {
  return zip([
    ['hello/composer.json', JSON.stringify({ name, description: 'A package of the end-to-end tests' })],
    ['hello/src/Hello.php', '<?php\n'],
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
    await call(server, 'POST', '/service/rest/v1/repositories/composer/hosted', {
      name,
      online: true,
      storage: { blobStoreName: 'default', strictContentTypeValidation: true, writePolicy: 'ALLOW_ONCE' },
    }),
    'create',
  );
}

test('a Composer package goes up through the upload page and shows in browse with its usage', async ({
  page,
  server,
}) => {
  await hosted(server, 'php');
  await signInAsAdmin(page, server);
  await page.goto('/upload/php');
  await page
    .getByLabel(FILE)
    .and(page.locator('input[type=file]'))
    .setInputFiles({ name: 'hello.zip', mimeType: 'application/zip', buffer: packageZip('acme/hello') });
  await page.getByRole('textbox', { name: /^Vendor/ }).fill('acme');
  await page.getByRole('textbox', { name: /^Project/ }).fill('hello');
  await page.getByRole('textbox', { name: /^Version/ }).fill('1.0.0');
  await page.getByRole('button', { name: 'Upload Component' }).click();
  await expect(page).toHaveURL(/\/upload$/);
  const metadata = await (await call(server, 'GET', '/repository/php/p2/acme/hello.json')).json();
  expect(metadata.packages['acme/hello'][0].version).toBe('1.0.0');

  await page.goto('/browse/php');
  const tree = page.getByRole('tree');
  await expect(tree.getByText('p2', { exact: true })).toBeVisible();
  await tree.getByText('acme', { exact: true }).click();
  await tree.getByText('hello', { exact: true }).click();
  await tree.getByText('1.0.0', { exact: true }).click();
  await tree.getByText('acme-hello-1.0.0.zip').click();
  const url = `${server.url}/repository/php/`;
  await expect(page.getByRole('tab', { name: 'composer config' })).toHaveAttribute('aria-selected', 'true');
  await expect(page.getByText(`composer config repositories.php composer ${url}`)).toBeVisible();
  await page.getByRole('tab', { name: 'composer require' }).click();
  await expect(page.getByText('composer require acme/hello:1.0.0')).toBeVisible();
  await page.getByRole('tab', { name: 'Upload' }).click();
  await expect(page.getByText(`${url}packages/upload/acme/hello/1.0.0`)).toBeVisible();

  await page.goto(`/search/component/${encodeURIComponent('composer:acme:hello')}?version=1.0.0`);
  await expect(page.getByText('composer require acme/hello:1.0.0')).toBeVisible();

  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/browse/php/acme/hello/1.0.0/acme-hello-1.0.0.zip');
  await expect(page.getByRole('tab', { name: 'composer config' })).toBeVisible();
  await noHorizontalScroll(page);
});

test('a Composer proxy is made through the form', async ({ page, server }) => {
  await signInAsAdmin(page, server);
  await page.goto('/admin/repository/repositories/create/composer/proxy');
  await expect(page.getByText(/https:\/\/repo\.packagist\.org/)).toBeVisible();
  await page.getByRole('textbox', { name: 'Name', exact: true }).fill('packagist');
  await page.getByRole('textbox', { name: /Remote Storage/ }).fill('https://repo.packagist.org');
  await page.getByRole('button', { name: 'Continue' }).click();
  await expect(page).toHaveURL(/\/admin\/repository\/repositories$/);
});

async function shoot(page: Page, name: string) {
  if (DIR) {
    mkdirSync(DIR, { recursive: true });
    await page.waitForTimeout(300);
    await page.screenshot({ path: join(DIR, `${name}.png`) });
  }
}

test('composer screens', async ({ page, server }) => {
  test.skip(!DIR, 'set ASTRUM_SHOTS_DIR to take screenshots');
  test.setTimeout(240_000);
  await hosted(server, 'php-shots');
  await expectOk(
    await call(
      server,
      'PUT',
      '/repository/php-shots/packages/upload/acme/hello/1.0.0',
      packageZip('acme/hello'),
      {
        'Content-Type': 'application/zip',
      },
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
      await shotPage.goto('/upload/php-shots');
      await expect(shotPage.getByLabel(FILE).and(shotPage.locator('input[type=file]'))).toBeAttached();
      await shoot(shotPage, `composer-upload-${suffix}`);
      await shotPage.goto('/browse/php-shots/acme/hello/1.0.0/acme-hello-1.0.0.zip');
      await expect(shotPage.getByRole('tab', { name: 'composer config' })).toBeVisible();
      await shoot(shotPage, `composer-browse-zip-${suffix}`);
      await context.close();
    }
  }
});
