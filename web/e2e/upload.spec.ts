import { call, hosted, npmTarball, wheel, wheelName } from './content';
import { expect, noHorizontalScroll, signIn, test } from './fixtures';

const FILE = /^File\*?$/;

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

test('a raw file goes up under a directory and shows in browse', async ({ page, server }) => {
  await page.getByRole('navigation', { name: 'Main menu' }).getByRole('link', { name: 'Upload' }).click();
  await expect(page.getByRole('heading', { level: 1, name: 'Upload' })).toBeVisible();
  await page.getByLabel('Name').fill('upload-r');
  await page.getByRole('link', { name: 'upload-raw' }).click();
  await expect(page).toHaveURL(/\/upload\/upload-raw$/);
  await page.getByRole('button', { name: 'Upload', exact: true }).click();
  await expect(page.getByText('Fix the marked fields and try again.')).toBeVisible();
  await expect(page.getByText('Choose a file.')).toBeVisible();
  await page.getByLabel(/Directory/).fill('/e2e/notes');
  await page.getByLabel(FILE).setInputFiles({
    name: 'notes.txt',
    mimeType: 'text/plain',
    buffer: Buffer.from('uploaded from the browser\n'),
  });
  await expect(page.getByLabel(/File name/)).toHaveValue('notes.txt');
  await page.getByRole('button', { name: 'Add another file' }).click();
  const second = page.getByRole('group', { name: 'File 2' });
  await second.getByLabel(FILE).setInputFiles({
    name: 'more.txt',
    mimeType: 'text/plain',
    buffer: Buffer.from('more\n'),
  });
  await second.getByLabel(/File name/).fill('sub/more.txt');
  await page.getByRole('button', { name: 'Upload', exact: true }).click();
  await expect(page.getByRole('status').filter({ hasText: 'Uploaded to upload-raw.' }).first()).toBeVisible();
  await expect(page.getByText('No files chosen')).toBeVisible();
  const stored = await call(server, 'GET', '/repository/upload-raw/e2e/notes/notes.txt');
  expect(await stored.text()).toBe('uploaded from the browser\n');
  expect((await call(server, 'GET', '/repository/upload-raw/e2e/notes/sub/more.txt')).status).toBe(200);
  await page.getByRole('link', { name: 'Show in Browse' }).click();
  await expect(page).toHaveURL(/\/browse\/upload-raw\/e2e\/notes$/);
});

test('a maven artifact goes up with a generated POM', async ({ page, server }) => {
  await page.goto('/upload/upload-maven');
  await page.getByLabel(/Group ID/).fill('org.example');
  await page.getByLabel(/Artifact ID/).fill('demo');
  await page.getByLabel(/^Version/).fill('1.0.0');
  await page.getByLabel(/Packaging/).fill('jar');
  await page.getByLabel('Generate a POM').check();
  await page.getByLabel(FILE).setInputFiles({
    name: 'demo-1.0.0.jar',
    mimeType: 'application/java-archive',
    buffer: Buffer.from('PK\u0003\u0004 demo jar'),
  });
  await expect(page.getByLabel(/Extension/)).toHaveValue('jar');
  await page.getByRole('button', { name: 'Add another file' }).click();
  const second = page.getByRole('group', { name: 'File 2' });
  await second.getByLabel(FILE).setInputFiles({
    name: 'demo-1.0.0-sources.jar',
    mimeType: 'application/java-archive',
    buffer: Buffer.from('PK\u0003\u0004 demo sources'),
  });
  await expect(second.getByLabel(/Classifier/)).toHaveValue('sources');
  await page.getByRole('button', { name: 'Upload', exact: true }).click();
  await expect(page.getByRole('link', { name: 'Open demo 1.0.0' })).toHaveAttribute(
    'href',
    '/search/component/maven2%3Aorg.example%3Ademo?version=1.0.0',
  );
  await expect(page.getByRole('link', { name: 'Show in Browse' })).toHaveAttribute(
    'href',
    '/browse/upload-maven/org/example/demo/1.0.0',
  );
  const pom = await call(server, 'GET', '/repository/upload-maven/org/example/demo/1.0.0/demo-1.0.0.pom');
  expect(pom.status).toBe(200);
  expect(await pom.text()).toContain('<artifactId>demo</artifactId>');
  const sources = await call(
    server,
    'GET',
    '/repository/upload-maven/org/example/demo/1.0.0/demo-1.0.0-sources.jar',
  );
  expect(sources.status).toBe(200);

  await page.getByLabel(/Group ID/).fill('org.example');
  await page.getByLabel(/Artifact ID/).fill('demo');
  await page.getByLabel(/^Version/).fill('2.0.0-SNAPSHOT');
  await page.getByLabel(FILE).setInputFiles({
    name: 'demo.jar',
    mimeType: 'application/java-archive',
    buffer: Buffer.from('PK\u0003\u0004 snapshot'),
  });
  await page.getByRole('button', { name: 'Upload', exact: true }).click();
  await expect(page.getByText(/Version policy mismatch/)).toBeVisible();
});

test('an npm tarball goes up and becomes a package', async ({ page, server }) => {
  await page.goto('/upload/upload-npm');
  await expect(
    page.getByText('Upload a package tarball (.tgz); the name and version come from its package.json.'),
  ).toBeVisible();
  await page.getByLabel(FILE).setInputFiles({
    name: 'widgets-2.0.0.tgz',
    mimeType: 'application/gzip',
    buffer: npmTarball('@e2e/widgets', '2.0.0'),
  });
  await page.getByRole('button', { name: 'Upload', exact: true }).click();
  await expect(page.getByRole('link', { name: 'Open widgets 2.0.0' })).toHaveAttribute(
    'href',
    '/search/component/npm%3Ae2e%3Awidgets?version=2.0.0',
  );
  const packument = await call(server, 'GET', '/repository/upload-npm/@e2e%2fwidgets');
  expect((await packument.json())['dist-tags'].latest).toBe('2.0.0');
  await page.getByLabel(FILE).setInputFiles({
    name: 'broken.tgz',
    mimeType: 'application/gzip',
    buffer: Buffer.from('not gzip'),
  });
  await page.getByRole('button', { name: 'Upload', exact: true }).click();
  await expect(page.getByText(/npm\.asset: the file is not a gzip-compressed tarball/)).toBeVisible();
  await expect(page.getByLabel(FILE)).toHaveAttribute('aria-invalid', 'true');
});

test('a wheel goes up to a pypi repository', async ({ page, server }) => {
  await page.goto('/upload/upload-pypi');
  await page.getByLabel(FILE).setInputFiles({
    name: wheelName('e2e-tool', '0.1.0'),
    mimeType: 'application/zip',
    buffer: wheel('e2e-tool', '0.1.0', 'A tool from the upload page'),
  });
  await page.getByRole('button', { name: 'Upload', exact: true }).click();
  await expect(page.getByRole('link', { name: 'Open e2e-tool 0.1.0' })).toBeVisible();
  const simple = await call(server, 'GET', '/repository/upload-pypi/simple/e2e-tool/');
  expect(await simple.text()).toContain(wheelName('e2e-tool', '0.1.0'));
  await page.getByRole('link', { name: 'Open e2e-tool 0.1.0' }).click();
  await expect(page).toHaveURL(/\/search\/component\/pypi%3A%3Ae2e-tool\?version=0\.1\.0$/);
});

test('the upload pages fit a phone', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/upload');
  await expect(page.getByRole('link', { name: 'upload-maven' })).toBeVisible();
  await noHorizontalScroll(page);
  await page.goto('/upload/upload-maven');
  await expect(page.getByLabel(/Group ID/)).toBeVisible();
  await page.getByRole('button', { name: 'Add another file' }).click();
  await noHorizontalScroll(page);
});
