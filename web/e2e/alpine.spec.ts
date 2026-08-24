import { createHash, generateKeyPairSync } from 'node:crypto';
import { mkdirSync } from 'node:fs';
import { join } from 'node:path';
import { gunzipSync, gzipSync } from 'node:zlib';
import type { Page } from '@playwright/test';
import { call, tar } from './content';
import { expect, noHorizontalScroll, type Server, signIn, test } from './fixtures';

test.describe.configure({ timeout: 120_000 });

const DIR = process.env.ASTRUM_SHOTS_DIR;
const FILE = /^File\*?$/;

function signingKey(): string {
  return generateKeyPairSync('rsa', {
    modulusLength: 2048,
    publicKeyEncoding: { type: 'spki', format: 'pem' },
    privateKeyEncoding: { type: 'pkcs8', format: 'pem' },
  }).privateKey;
}

function cut(archive: Buffer): Buffer {
  return archive.subarray(0, archive.length - 1024);
}

function apk(name: string, version: string, arch: string): Buffer {
  const data = gzipSync(tar([[`usr/share/doc/${name}/README`, `${name} ${version}\n`]]));
  const info = [
    `pkgname = ${name}`,
    `pkgver = ${version}`,
    `pkgdesc = ${name} of the end-to-end tests`,
    'url = https://example.org/',
    'builddate = 1700000000',
    'size = 16',
    `arch = ${arch}`,
    `origin = ${name}`,
    'license = MIT',
    `datahash = ${createHash('sha256').update(data).digest('hex')}`,
    '',
  ].join('\n');
  return Buffer.concat([gzipSync(cut(tar([['.PKGINFO', info]]))), data]);
}

function indexNames(file: Buffer): string[] {
  const text = gunzipSync(file).toString('latin1');
  return [...text.matchAll(/^P:(.+)$/gm)].map((match) => match[1] ?? '');
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
    await call(server, 'POST', '/service/rest/v1/repositories/alpine/hosted', {
      name,
      online: true,
      storage: { blobStoreName: 'default', strictContentTypeValidation: true, writePolicy: 'ALLOW_ONCE' },
      alpineSigning: { keypair: signingKey() },
    }),
    'create',
  );
}

test('a hosted Alpine repository is made through the form with its signing key', async ({ page, server }) => {
  await signInAsAdmin(page, server);
  await page.goto('/admin/repository/repositories/create/alpine/hosted');
  await expect(page.getByText('Alpine Signing', { exact: true })).toBeVisible();
  await page.getByRole('textbox', { name: 'Name', exact: true }).fill('apks');
  await expect(page.getByText(/clients save the public key apks\.rsa\.pub/)).toBeVisible();
  await page.getByRole('textbox', { name: /^Signing Key/ }).fill(signingKey());
  await page.getByRole('button', { name: 'Continue' }).click();
  await expect(page).toHaveURL(/\/admin\/repository\/repositories$/);
  const stored = await (await call(server, 'GET', '/service/rest/v1/repositories/alpine/hosted/apks')).json();
  expect(stored.alpineSigning).toEqual({});
  const key = await call(server, 'GET', '/repository/apks/apks.rsa.pub');
  expect(await key.text()).toContain('-----BEGIN PUBLIC KEY-----');

  await page.goto('/admin/repository/repositories/create/alpine/group');
  await expect(page.getByText('Alpine Signing', { exact: true })).toBeVisible();
  await expect(page.getByText(/without a key the group passes the signed index/)).toBeVisible();
  await page.goto('/admin/repository/repositories/create/alpine/proxy');
  await expect(page.getByText(/for example, https:\/\/dl-cdn\.alpinelinux\.org\/alpine\//)).toBeVisible();
});

test('a package goes up through the upload page and shows in browse with its usage', async ({
  page,
  server,
}) => {
  await hosted(server, 'apks-upload');
  await signInAsAdmin(page, server);
  await page.goto('/upload/apks-upload');
  await expect(page.getByRole('heading', { level: 1, name: 'Upload to apks-upload' })).toBeVisible();
  await page.getByRole('textbox', { name: 'Directory' }).fill('v3.20/main/x86_64');
  await page
    .getByLabel(FILE)
    .and(page.locator('input[type=file]'))
    .setInputFiles({
      name: 'hello-astrum-1.0.0-r0.apk',
      mimeType: 'application/octet-stream',
      buffer: apk('hello-astrum', '1.0.0-r0', 'x86_64'),
    });
  await page.getByRole('button', { name: 'Upload Component' }).click();
  await expect(page).toHaveURL(/\/upload$/);
  const index = await call(server, 'GET', '/repository/apks-upload/v3.20/main/x86_64/APKINDEX.tar.gz');
  expect(index.status).toBe(200);
  const file = Buffer.from(await index.arrayBuffer());
  expect(gunzipSync(file).toString('latin1')).toContain('.SIGN.RSA.apks-upload.rsa.pub');
  expect(indexNames(file)).toEqual(['hello-astrum']);

  await page.goto('/browse/apks-upload');
  const tree = page.getByRole('tree');
  await tree.getByText('v3.20', { exact: true }).click();
  await tree.getByText('main', { exact: true }).click();
  await tree.getByText('x86_64', { exact: true }).click();
  await expect(tree.getByText('APKINDEX.tar.gz')).toBeVisible();
  await tree.getByText('hello-astrum-1.0.0-r0.apk').click();
  const url = `${server.url}/repository/apks-upload/`;
  await expect(page.getByRole('tab', { name: '/etc/apk/repositories' })).toHaveAttribute(
    'aria-selected',
    'true',
  );
  await expect(
    page.getByText(`wget -O /etc/apk/keys/apks-upload.rsa.pub ${url}apks-upload.rsa.pub`),
  ).toBeVisible();
  await expect(page.getByText(`echo "${url}v3.20/main" >> /etc/apk/repositories`)).toBeVisible();
  await expect(page.getByText('Save the signing key once, then add the branch')).toBeVisible();
  await page.getByRole('tab', { name: 'apk add' }).click();
  await expect(page.getByText('apk add hello-astrum=1.0.0-r0')).toBeVisible();
  await page.getByRole('tab', { name: 'Upload', exact: true }).click();
  await expect(
    page.getByText(
      `curl -u <user> --upload-file hello-astrum-1.0.0-r0.apk ${url}v3.20/main/x86_64/hello-astrum-1.0.0-r0.apk`,
    ),
  ).toBeVisible();

  await page.setViewportSize({ width: 390, height: 844 });
  await page.reload();
  await expect(page.getByRole('tab', { name: '/etc/apk/repositories' })).toBeVisible();
  await noHorizontalScroll(page);
});

async function shoot(page: Page, name: string) {
  if (DIR) {
    mkdirSync(DIR, { recursive: true });
    await page.waitForTimeout(300);
    await page.screenshot({ path: join(DIR, `${name}.png`) });
  }
}

test('alpine screens', async ({ page, server }) => {
  test.skip(!DIR, 'set ASTRUM_SHOTS_DIR to take screenshots');
  test.setTimeout(240_000);
  await hosted(server, 'apks-shots');
  await expectOk(
    await call(
      server,
      'PUT',
      '/repository/apks-shots/v3.20/main/x86_64/hello-astrum-1.0.0-r0.apk',
      apk('hello-astrum', '1.0.0-r0', 'x86_64'),
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
      await shotPage.goto('/admin/repository/repositories/create/alpine/hosted');
      await expect(shotPage.getByText('Alpine Signing', { exact: true })).toBeVisible();
      await shotPage.getByText('Alpine Signing', { exact: true }).scrollIntoViewIfNeeded();
      await shoot(shotPage, `alpine-create-hosted-${suffix}`);
      await shotPage.goto('/browse/apks-shots/v3.20/main/x86_64/hello-astrum-1.0.0-r0.apk');
      await expect(shotPage.getByRole('tab', { name: '/etc/apk/repositories' })).toBeVisible();
      await shoot(shotPage, `alpine-browse-apk-${suffix}`);
      await context.close();
    }
  }
});
