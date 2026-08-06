import { execFileSync } from 'node:child_process';
import { mkdirSync, mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { gzipSync } from 'node:zlib';
import type { Page } from '@playwright/test';
import { call, tar } from './content';
import { expect, noHorizontalScroll, type Server, signIn, test } from './fixtures';

test.describe.configure({ timeout: 120_000 });

const DIR = process.env.ASTRUM_SHOTS_DIR;
const FILE = /^File\*?$/;

function signingKey(): string {
  const home = mkdtempSync(join(tmpdir(), 'astrum-gpg-'));
  try {
    execFileSync(
      'gpg',
      [
        '--homedir',
        home,
        '--batch',
        '--passphrase',
        '',
        '--quick-gen-key',
        'Astrum E2E <e2e@example.org>',
        'ed25519',
        'sign',
        'never',
      ],
      { stdio: 'ignore' },
    );
    return execFileSync('gpg', [
      '--homedir',
      home,
      '--batch',
      '--armor',
      '--export-secret-keys',
      'e2e@example.org',
    ]).toString('utf8');
  } finally {
    rmSync(home, { recursive: true, force: true });
  }
}

function arMember(name: string, data: Buffer): Buffer {
  const header = Buffer.from(
    `${name.padEnd(16)}${'0'.padEnd(12)}${'0'.padEnd(6)}${'0'.padEnd(6)}${'100644'.padEnd(8)}${String(data.length).padEnd(10)}\`\n`,
    'ascii',
  );
  return Buffer.concat([header, data, data.length % 2 === 1 ? Buffer.from('\n') : Buffer.alloc(0)]);
}

function deb(name: string, version: string, architecture: string): Buffer {
  const control = `Package: ${name}\nVersion: ${version}\nArchitecture: ${architecture}\nMaintainer: Astrum Tests <e2e@example.org>\nDescription: ${name} of the end-to-end tests\n`;
  return Buffer.concat([
    Buffer.from('!<arch>\n', 'ascii'),
    arMember('debian-binary', Buffer.from('2.0\n')),
    arMember('control.tar.gz', gzipSync(tar([['./control', control]]))),
    arMember('data.tar.gz', gzipSync(tar([[`./usr/share/doc/${name}/README`, 'hello\n']]))),
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
    await call(server, 'POST', '/service/rest/v1/repositories/apt/hosted', {
      name,
      online: true,
      storage: { blobStoreName: 'default', strictContentTypeValidation: true, writePolicy: 'ALLOW_ONCE' },
      apt: { distribution: 'bookworm' },
      aptSigning: { keypair: signingKey() },
    }),
    'create',
  );
}

test('a hosted APT repository is made through the form with its signing key', async ({ page, server }) => {
  await signInAsAdmin(page, server);
  await page.goto('/admin/repository/repositories/create/apt/hosted');
  await expect(page.getByText('APT Settings', { exact: true })).toBeVisible();
  await expect(page.getByText('APT Signing', { exact: true })).toBeVisible();
  await page.getByRole('textbox', { name: 'Name', exact: true }).fill('debs');
  await page.getByRole('button', { name: 'Continue' }).click();
  await expect(page.getByText('Enter the distribution')).toBeVisible();
  await expect(page.getByText('Paste the GPG signing key')).toBeVisible();
  await page.getByRole('textbox', { name: /^Distribution/ }).fill('bookworm');
  await page.getByRole('textbox', { name: /^GPG Signing Key\*?$/ }).fill(signingKey());
  await page.getByRole('button', { name: 'Continue' }).click();
  await expect(page).toHaveURL(/\/admin\/repository\/repositories$/);
  const stored = await (await call(server, 'GET', '/service/rest/v1/repositories/apt/hosted/debs')).json();
  expect(stored.apt.distribution).toBe('bookworm');
  const release = await call(server, 'GET', '/repository/debs/dists/bookworm/InRelease');
  expect(await release.text()).toContain('-----BEGIN PGP SIGNED MESSAGE-----');
});

test('a package goes up through the upload page and shows in browse with its usage', async ({
  page,
  server,
}) => {
  await hosted(server, 'debs-upload');
  await signInAsAdmin(page, server);
  await page.goto('/upload/debs-upload');
  await page
    .getByLabel(FILE)
    .and(page.locator('input[type=file]'))
    .setInputFiles({
      name: 'hello-astrum_1.0.0_amd64.deb',
      mimeType: 'application/vnd.debian.binary-package',
      buffer: deb('hello-astrum', '1.0.0', 'amd64'),
    });
  await page.getByRole('button', { name: 'Upload Component' }).click();
  await expect(page).toHaveURL(/\/upload$/);
  const listing = await (
    await call(server, 'GET', '/repository/debs-upload/dists/bookworm/main/binary-amd64/Packages')
  ).text();
  expect(listing).toContain('Package: hello-astrum');

  await page.goto('/browse/debs-upload');
  const tree = page.getByRole('tree');
  await expect(tree.getByText('metadata', { exact: true })).toBeVisible();
  await tree.getByText('packages', { exact: true }).click();
  await tree.getByText('h', { exact: true }).click();
  await tree.getByText('hello-astrum', { exact: true }).click();
  await tree.getByText('1.0.0', { exact: true }).click();
  await tree.getByText('hello-astrum_1.0.0_amd64.deb').click();
  const url = `${server.url}/repository/debs-upload/`;
  await expect(page.getByRole('tab', { name: 'sources.list' })).toHaveAttribute('aria-selected', 'true');
  await expect(page.getByText(`deb ${url} bookworm main`)).toBeVisible();
  await expect(page.getByText('/etc/apt/sources.list.d/debs-upload.list')).toBeVisible();
  await page.getByRole('tab', { name: 'apt-get' }).click();
  await expect(page.getByText(/sudo apt-get install hello-astrum=1\.0\.0/)).toBeVisible();
  await page.getByRole('tab', { name: 'Upload' }).click();
  await expect(page.getByText(/--data-binary "@hello-astrum_1\.0\.0_amd64\.deb"/)).toBeVisible();

  await page.setViewportSize({ width: 390, height: 844 });
  await page.reload();
  await expect(page.getByRole('tab', { name: 'sources.list' })).toBeVisible();
  await noHorizontalScroll(page);
});

async function shoot(page: Page, name: string) {
  if (DIR) {
    mkdirSync(DIR, { recursive: true });
    await page.waitForTimeout(300);
    await page.screenshot({ path: join(DIR, `${name}.png`) });
  }
}

test('apt screens', async ({ page, server }) => {
  test.skip(!DIR, 'set ASTRUM_SHOTS_DIR to take screenshots');
  test.setTimeout(240_000);
  await hosted(server, 'debs-shots');
  await expectOk(
    await call(server, 'POST', '/repository/debs-shots/', deb('hello-astrum', '1.0.0', 'all'), {
      'Content-Type': 'application/vnd.debian.binary-package',
    }),
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
      await shotPage.goto('/admin/repository/repositories/create/apt/proxy');
      await expect(shotPage.getByRole('textbox', { name: 'Name', exact: true })).toBeVisible();
      await shoot(shotPage, `apt-create-proxy-${suffix}`);
      await shotPage.goto('/browse/debs-shots/packages/h/hello-astrum/1.0.0/hello-astrum_1.0.0_all.deb');
      await expect(shotPage.getByRole('tab', { name: 'sources.list' })).toBeVisible();
      await shoot(shotPage, `apt-browse-deb-${suffix}`);
      await context.close();
    }
  }
});
