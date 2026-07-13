import { createHash } from 'node:crypto';
import { mkdirSync } from 'node:fs';
import { join } from 'node:path';
import type { Page } from '@playwright/test';
import { call } from './content';
import { expect, noHorizontalScroll, type Server, signIn, test } from './fixtures';

test.describe.configure({ timeout: 120_000 });

const DIR = process.env.ASTRUM_SHOTS_DIR;
const FILE = /^File\*?$/;
const DATA = Buffer.from('a large file kept in Git LFS\n'.repeat(64), 'utf8');
const OID = createHash('sha256').update(DATA).digest('hex');
const OBJECT_PATH = `objects/${OID.slice(0, 2)}/${OID.slice(2, 4)}/${OID}`;

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
    await call(server, 'POST', '/service/rest/v1/repositories/gitlfs/hosted', {
      name,
      online: true,
      storage: { blobStoreName: 'default', strictContentTypeValidation: true, writePolicy: 'ALLOW_ONCE' },
    }),
    'create',
  );
}

test('a hosted Git LFS repository is made through the form', async ({ page, server }) => {
  await signInAsAdmin(page, server);
  await page.goto('/admin/repository/repositories');
  await page.getByRole('link', { name: 'Create Repository' }).first().click();
  await page.getByRole('button', { name: 'Git LFS', exact: true }).click();
  await expect(page.getByText('Git Large File Storage for versioning large files.')).toBeVisible();
  await page.getByRole('button', { name: 'Create a hosted Git LFS repository' }).click();
  await page.getByRole('textbox', { name: 'Name', exact: true }).fill('large-files');
  await page.getByRole('button', { name: 'Continue' }).click();
  await expect(page).toHaveURL(/\/admin\/repository\/repositories$/);
  const stored = await (
    await call(server, 'GET', '/service/rest/v1/repositories/gitlfs/hosted/large-files')
  ).json();
  expect(stored.format).toBe('gitlfs');
});

test('an object goes up through the upload page and shows in browse with its usage', async ({
  page,
  server,
}) => {
  await hosted(server, 'lfs-upload');
  await signInAsAdmin(page, server);
  await page.goto('/upload/lfs-upload');
  await expect(page.getByRole('heading', { level: 1, name: 'Upload to lfs-upload' })).toBeVisible();
  await fileInput(page).setInputFiles({
    name: 'design.psd',
    mimeType: 'application/octet-stream',
    buffer: DATA,
  });
  await page.getByRole('button', { name: 'Upload Component' }).click();
  await expect(page).toHaveURL(/\/upload$/);
  await expect(page.getByText('Component uploaded to lfs-upload').first()).toBeVisible();
  const download = await call(server, 'GET', `/repository/lfs-upload/info/lfs/objects/${OID}`);
  expect(Buffer.from(await download.arrayBuffer()).equals(DATA)).toBe(true);

  await page.goto('/browse/lfs-upload');
  const tree = page.getByRole('tree');
  await tree.getByText('objects', { exact: true }).click();
  await tree.getByText(OID.slice(0, 2), { exact: true }).click();
  await tree.getByText(OID.slice(2, 4), { exact: true }).click();
  await tree.getByText(OID, { exact: true }).click();
  await expect(page).toHaveURL(new RegExp(`/browse/lfs-upload/${OBJECT_PATH}$`));
  const api = `${server.url}/repository/lfs-upload/info/lfs`;
  await expect(page.getByRole('tab', { name: '.lfsconfig' })).toHaveAttribute('aria-selected', 'true');
  await expect(page.getByText(`git config -f .lfsconfig lfs.url ${api}`)).toBeVisible();
  await expect(
    page.getByText(/commit \.lfsconfig, so that every clone keeps its large files here/),
  ).toBeVisible();
  await page.getByRole('tab', { name: 'curl' }).click();
  await expect(page.getByText(`curl -u <user> -fL -o ${OID} ${api}/objects/${OID}`)).toBeVisible();

  await page.setViewportSize({ width: 390, height: 844 });
  await page.reload();
  await expect(page.getByRole('tab', { name: '.lfsconfig' })).toBeVisible();
  await noHorizontalScroll(page);
});

async function shoot(page: Page, name: string) {
  if (DIR) {
    mkdirSync(DIR, { recursive: true });
    await page.waitForTimeout(300);
    await page.screenshot({ path: join(DIR, `${name}.png`) });
  }
}

test('git lfs screens', async ({ page, server }) => {
  test.skip(!DIR, 'set ASTRUM_SHOTS_DIR to take screenshots');
  test.setTimeout(240_000);
  await hosted(server, 'lfs-shots');
  await expectOk(await call(server, 'PUT', `/repository/lfs-shots/info/lfs/objects/${OID}`, DATA), 'put');
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
      await shotPage.goto('/admin/repository/repositories/create/gitlfs/hosted');
      await expect(shotPage.getByRole('textbox', { name: 'Name', exact: true })).toBeVisible();
      await shoot(shotPage, `gitlfs-create-hosted-${suffix}`);
      await shotPage.goto('/upload/lfs-shots');
      await expect(fileInput(shotPage)).toBeAttached();
      await shoot(shotPage, `gitlfs-upload-${suffix}`);
      await shotPage.goto(`/browse/lfs-shots/${OBJECT_PATH}`);
      await expect(shotPage.getByRole('tab', { name: '.lfsconfig' })).toBeVisible();
      await shoot(shotPage, `gitlfs-browse-object-${suffix}`);
      await context.close();
    }
  }
});
