import { mkdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import type { Page } from '@playwright/test';
import { call } from './content';
import { expect, noHorizontalScroll, type Server, signIn, test } from './fixtures';

test.describe.configure({ timeout: 120_000 });

const DIR = process.env.ASTRUM_SHOTS_DIR;
const RPM = readFileSync(join(import.meta.dirname, 'fixtures', 'astrum-hello-1.0-1.noarch.rpm'));
const FILE = /^File\*?$/;

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

async function hosted(server: Server, name: string, depth = 0) {
  await expectOk(
    await call(server, 'POST', '/service/rest/v1/repositories/yum/hosted', {
      name,
      online: true,
      storage: { blobStoreName: 'default', strictContentTypeValidation: true, writePolicy: 'ALLOW' },
      yum: { repodataDepth: depth, deployPolicy: 'STRICT' },
    }),
    'create',
  );
}

test('a hosted yum repository is made through the form with its repodata settings', async ({
  page,
  server,
}) => {
  await signInAsAdmin(page, server);
  await page.goto('/admin/repository/repositories');
  await page.getByRole('link', { name: 'Create Repository' }).first().click();
  await page.getByRole('button', { name: 'Yum', exact: true }).click();
  await page.getByRole('button', { name: 'Create a hosted Yum repository' }).click();
  await page.getByRole('textbox', { name: 'Name', exact: true }).fill('rpms');
  await expect(page.getByText('Yum Settings', { exact: true })).toBeVisible();
  await expect(page.getByText('Metadata Signing', { exact: true })).toBeVisible();
  await page.getByRole('spinbutton', { name: 'Repodata Depth' }).fill('2');
  await page.getByRole('combobox', { name: 'Layout Policy' }).click();
  await page.getByRole('option', { name: 'Permissive', exact: true }).click();
  await page.getByRole('button', { name: 'Continue' }).click();
  await expect(page).toHaveURL(/\/admin\/repository\/repositories$/);
  const stored = await (await call(server, 'GET', '/service/rest/v1/repositories/yum/hosted/rpms')).json();
  expect(stored.yum).toEqual({ repodataDepth: 2, deployPolicy: 'PERMISSIVE' });
  expect(stored.yumSigning ?? null).toBeNull();
});

test('a package goes up through the upload page and shows in browse with its usage', async ({
  page,
  server,
}) => {
  await hosted(server, 'rpms-upload');
  await signInAsAdmin(page, server);
  await page.goto('/upload/rpms-upload');
  await expect(page.getByRole('heading', { level: 1, name: 'Upload to rpms-upload' })).toBeVisible();
  await page.getByRole('textbox', { name: 'Directory' }).fill('Packages');
  await fileInput(page).setInputFiles({
    name: 'astrum-hello-1.0-1.noarch.rpm',
    mimeType: 'application/x-rpm',
    buffer: RPM,
  });
  await page.getByRole('button', { name: 'Upload Component' }).click();
  await expect(page).toHaveURL(/\/upload$/);
  await expect(page.getByText('Component uploaded to rpms-upload').first()).toBeVisible();
  const repomd = await (await call(server, 'GET', '/repository/rpms-upload/repodata/repomd.xml')).text();
  expect(repomd).toContain('<data type="primary">');

  await page.goto('/browse/rpms-upload');
  const tree = page.getByRole('tree');
  await expect(tree.getByText('repodata', { exact: true })).toBeVisible();
  await tree.getByText('Packages', { exact: true }).click();
  await tree.getByText('astrum-hello-1.0-1.noarch.rpm').click();
  await expect(page).toHaveURL(/\/browse\/rpms-upload\/Packages\/astrum-hello-1\.0-1\.noarch\.rpm$/);
  const repositoryUrl = `${server.url}/repository/rpms-upload/`;
  await expect(page.getByRole('tab', { name: 'rpms-upload.repo' })).toHaveAttribute('aria-selected', 'true');
  await expect(
    page.getByText('Save it as /etc/yum.repos.d/rpms-upload.repo and install packages with dnf or yum'),
  ).toBeVisible();
  await expect(page.getByText(`baseurl=${repositoryUrl}`)).toBeVisible();
  await page.getByRole('tab', { name: 'dnf' }).click();
  await expect(page.getByText('sudo dnf install astrum-hello-1.0-1')).toBeVisible();
  await page.getByRole('tab', { name: 'Upload', exact: true }).click();
  await expect(
    page.getByText(
      `curl -u <user> --upload-file ./astrum-hello-1.0-1.noarch.rpm ${repositoryUrl}Packages/astrum-hello-1.0-1.noarch.rpm`,
    ),
  ).toBeVisible();

  await page.setViewportSize({ width: 390, height: 844 });
  await page.reload();
  await expect(page.getByRole('tab', { name: 'rpms-upload.repo' })).toBeVisible();
  await noHorizontalScroll(page);
});

test('the criteria search finds packages by their architecture', async ({ page, server }) => {
  await hosted(server, 'rpms-search');
  await expectOk(
    await call(server, 'PUT', '/repository/rpms-search/astrum-hello-1.0-1.noarch.rpm', RPM),
    'put',
  );
  await signInAsAdmin(page, server);
  await page.goto('/search/custom?yum.architecture=noarch&repository=rpms-search');
  const found = page.getByRole('table').getByRole('button', { name: /^Open .* in the repository$/ });
  await expect(found).toHaveCount(1);
  await expect(found.first()).toContainText('astrum-hello');
});

async function shoot(page: Page, name: string) {
  if (DIR) {
    mkdirSync(DIR, { recursive: true });
    await page.waitForTimeout(300);
    await page.screenshot({ path: join(DIR, `${name}.png`) });
  }
}

test('yum screens', async ({ page, server }) => {
  test.skip(!DIR, 'set ASTRUM_SHOTS_DIR to take screenshots');
  test.setTimeout(240_000);
  await hosted(server, 'rpms-shots');
  await expectOk(
    await call(server, 'PUT', '/repository/rpms-shots/Packages/astrum-hello-1.0-1.noarch.rpm', RPM),
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
      await shotPage.goto('/admin/repository/repositories/create/yum/hosted');
      await expect(shotPage.getByRole('textbox', { name: 'Name', exact: true })).toBeVisible();
      await shoot(shotPage, `yum-create-hosted-${suffix}`);
      await shotPage.goto('/upload/rpms-shots');
      await expect(fileInput(shotPage)).toBeAttached();
      await shoot(shotPage, `yum-upload-${suffix}`);
      await shotPage.goto('/browse/rpms-shots/Packages/astrum-hello-1.0-1.noarch.rpm');
      await expect(shotPage.getByRole('tab', { name: 'rpms-shots.repo' })).toBeVisible();
      await shoot(shotPage, `yum-browse-package-${suffix}`);
      await context.close();
    }
  }
});
