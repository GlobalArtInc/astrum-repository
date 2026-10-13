import type { Page } from '@playwright/test';
import { call, hosted, npmTarball, wheel, wheelName } from './content';
import { expect, noHorizontalScroll, signIn, test } from './fixtures';

const FILE = /^File\*?$/;

function fileInput(scope: Page | ReturnType<Page['getByRole']>) {
  return scope.getByLabel(FILE).and(scope.locator('input[type=file]'));
}

test.beforeEach(async ({ page, server }) => {
  await hosted(server, 'raw', 'upload-raw', { raw: { contentDisposition: 'ATTACHMENT' } });
  await hosted(server, 'maven', 'upload-maven', {
    maven: { versionPolicy: 'RELEASE', layoutPolicy: 'STRICT', contentDisposition: 'ATTACHMENT' },
  });
  await hosted(server, 'npm', 'upload-npm');
  await hosted(server, 'pypi', 'upload-pypi');
  await page.goto('/login');
  await signIn(page, server.adminPassword);
  await expect(page.getByRole('heading', { level: 1, name: 'Overview' })).toBeVisible();
});

test('a raw file goes up under a directory', async ({ page, server }) => {
  await page.getByRole('navigation', { name: 'Main menu' }).getByRole('link', { name: 'Upload' }).click();
  await expect(page.getByRole('heading', { level: 1, name: 'Upload' })).toBeVisible();
  await expect(page.getByText('4 repositories')).toBeVisible();
  await page
    .getByRole('complementary', { name: 'Filter panel' })
    .getByRole('checkbox', { name: 'Raw' })
    .click();
  await expect(page.getByText('1 repository')).toBeVisible();
  await page.getByRole('textbox', { name: 'Filter repositories by name' }).fill('upload-r');
  await page.getByRole('button', { name: 'Upload to upload-raw' }).click();
  await expect(page).toHaveURL(/\/upload\/upload-raw$/);
  await expect(page.getByRole('heading', { level: 1, name: 'Upload to upload-raw' })).toBeVisible();
  await expect(page.getByText('No files selected')).toBeVisible();
  await expect(page.getByRole('button', { name: 'Upload Component' })).toBeDisabled();
  await fileInput(page).setInputFiles({
    name: 'notes.txt',
    mimeType: 'text/plain',
    buffer: Buffer.from('uploaded from the browser\n'),
  });
  await expect(page.getByText('1 file ready')).toBeVisible();
  await expect(page.getByLabel(/File Name/)).toHaveValue('notes.txt');
  await page.getByRole('button', { name: 'Upload Component' }).click();
  await expect(page.getByText('Fix the errors in the form')).toBeVisible();
  await expect(page.getByText('This field is required')).toBeVisible();
  await page.getByLabel(/Directory/).fill('/e2e/notes');
  await page.getByRole('button', { name: 'Add Another File' }).click();
  const second = page.getByRole('group', { name: 'File 2' });
  await fileInput(second).setInputFiles({
    name: 'more.txt',
    mimeType: 'text/plain',
    buffer: Buffer.from('more\n'),
  });
  await second.getByLabel(/File Name/).fill('sub/more.txt');
  await page.getByRole('button', { name: 'Upload Component' }).click();
  await expect(page).toHaveURL(/\/upload$/);
  await expect(page.getByText('Component uploaded to upload-raw').first()).toBeVisible();
  await expect(page.getByText('Component: e2e notes').first()).toBeVisible();
  const stored = await call(server, 'GET', '/repository/upload-raw/e2e/notes/notes.txt');
  expect(await stored.text()).toBe('uploaded from the browser\n');
  expect((await call(server, 'GET', '/repository/upload-raw/e2e/notes/sub/more.txt')).status).toBe(200);
});

test('a maven artifact goes up with a generated POM', async ({ page, server }) => {
  await page.goto('/upload/upload-maven');
  await page.getByLabel(/Group ID/).fill('org.example');
  await page.getByLabel(/Artifact ID/).fill('demo');
  await page.getByLabel(/^Version/).fill('1.0.0');
  await expect(page.getByRole('combobox', { name: 'Packaging' })).toBeDisabled();
  await page.getByText('Generate POM', { exact: true }).click();
  await expect(page.getByRole('combobox', { name: 'Packaging' })).toHaveText('jar');
  await fileInput(page).setInputFiles({
    name: 'demo-1.0.0.jar',
    mimeType: 'application/java-archive',
    buffer: Buffer.from('PK\u0003\u0004 demo jar'),
  });
  await expect(page.getByLabel(/Extension/)).toHaveValue('jar');
  await page.getByRole('button', { name: 'Add Another File' }).click();
  const second = page.getByRole('group', { name: 'File 2' });
  await fileInput(second).setInputFiles({
    name: 'demo-1.0.0-sources.jar',
    mimeType: 'application/java-archive',
    buffer: Buffer.from('PK\u0003\u0004 demo sources'),
  });
  await expect(second.getByLabel(/Classifier/)).toHaveValue('sources');
  await page.getByRole('button', { name: 'Upload Component' }).click();
  await expect(page).toHaveURL(/\/upload$/);
  await expect(page.getByText('Component: org.example demo 1.0.0').first()).toBeVisible();
  const pom = await call(server, 'GET', '/repository/upload-maven/org/example/demo/1.0.0/demo-1.0.0.pom');
  expect(pom.status).toBe(200);
  expect(await pom.text()).toContain('<artifactId>demo</artifactId>');
  const sources = await call(
    server,
    'GET',
    '/repository/upload-maven/org/example/demo/1.0.0/demo-1.0.0-sources.jar',
  );
  expect(sources.status).toBe(200);

  await page.goto('/upload/upload-maven');
  await page.getByLabel(/Group ID/).fill('org.example');
  await page.getByLabel(/Artifact ID/).fill('demo');
  await page.getByLabel(/^Version/).fill('2.0.0-SNAPSHOT');
  await fileInput(page).setInputFiles({
    name: 'demo.jar',
    mimeType: 'application/java-archive',
    buffer: Buffer.from('PK\u0003\u0004 snapshot'),
  });
  await page.getByRole('button', { name: 'Upload Component' }).click();
  await expect(page.getByText(/Version policy mismatch/)).toBeVisible();
});

test('an npm tarball goes up and becomes a package', async ({ page, server }) => {
  await page.goto('/upload/upload-npm');
  await fileInput(page).setInputFiles({
    name: 'widgets-2.0.0.tgz',
    mimeType: 'application/gzip',
    buffer: npmTarball('@e2e/widgets', '2.0.0'),
  });
  await page.getByRole('button', { name: 'Upload Component' }).click();
  await expect(page).toHaveURL(/\/upload$/);
  await expect(page.getByText('Component uploaded to upload-npm').first()).toBeVisible();
  const packument = await call(server, 'GET', '/repository/upload-npm/@e2e%2fwidgets');
  expect((await packument.json())['dist-tags'].latest).toBe('2.0.0');
  await page.goto('/upload/upload-npm');
  await fileInput(page).setInputFiles({
    name: 'broken.tgz',
    mimeType: 'application/gzip',
    buffer: Buffer.from('not gzip'),
  });
  await page.getByRole('button', { name: 'Upload Component' }).click();
  await expect(page.getByText(/npm\.asset: the file is not a gzip-compressed tarball/)).toBeVisible();
  await expect(fileInput(page)).toHaveAttribute('aria-invalid', 'true');
});

test('a wheel goes up to a pypi repository chosen in the form', async ({ page, server }) => {
  await page.goto('/upload/upload-raw');
  const target = page.getByRole('combobox', { name: /Target Repository/ });
  await target.fill('upload-py');
  await page.getByRole('option', { name: /upload-pypi/ }).click();
  await expect(page).toHaveURL(/\/upload\/upload-pypi$/);
  await fileInput(page).setInputFiles({
    name: wheelName('e2e-tool', '0.1.0'),
    mimeType: 'application/zip',
    buffer: wheel('e2e-tool', '0.1.0', 'A tool from the upload page'),
  });
  await page.getByRole('button', { name: 'Upload Component' }).click();
  await expect(page).toHaveURL(/\/upload$/);
  const simple = await call(server, 'GET', '/repository/upload-pypi/simple/e2e-tool/');
  expect(await simple.text()).toContain(wheelName('e2e-tool', '0.1.0'));
});

test('the upload pages fit a phone', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/upload');
  await expect(page.getByRole('button', { name: 'Upload to upload-maven' })).toBeVisible();
  await noHorizontalScroll(page);
  await page.getByRole('button', { name: 'Open filters' }).click();
  const sheet = page.getByRole('dialog', { name: 'Filter' });
  await sheet.getByRole('checkbox', { name: 'Maven' }).click();
  await sheet.getByRole('button', { name: 'Close' }).click();
  await expect(page.getByText('1 repository')).toBeVisible();
  await page.goto('/upload/upload-maven');
  await expect(page.getByLabel(/Group ID/)).toBeVisible();
  await page.getByRole('button', { name: 'Add Another File' }).click();
  await noHorizontalScroll(page);
});
