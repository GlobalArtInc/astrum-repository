import { mkdirSync } from 'node:fs';
import { join } from 'node:path';
import { gzipSync } from 'node:zlib';
import type { Page } from '@playwright/test';
import { call, tar } from './content';
import { expect, noHorizontalScroll, type Server, signIn, test } from './fixtures';

test.describe.configure({ timeout: 120_000 });

const DIR = process.env.ASTRUM_SHOTS_DIR;

function publishBody(name: string, version: string): Buffer {
  const metadata = Buffer.from(
    JSON.stringify({
      name,
      vers: version,
      deps: [],
      features: {},
      authors: ['Astrum Tests'],
      description: `The ${name} crate of the end-to-end tests`,
      keywords: [],
      categories: [],
      license: 'MIT',
    }),
    'utf8',
  );
  const archive = gzipSync(tar([[`${name}-${version}/Cargo.toml`, `[package]\nname = "${name}"\n`]]));
  const head = Buffer.alloc(4);
  head.writeUInt32LE(metadata.length, 0);
  const middle = Buffer.alloc(4);
  middle.writeUInt32LE(archive.length, 0);
  return Buffer.concat([head, metadata, middle, archive]);
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
    await call(server, 'POST', '/service/rest/v1/repositories/cargo/hosted', {
      name,
      online: true,
      storage: { blobStoreName: 'default', strictContentTypeValidation: true, writePolicy: 'ALLOW_ONCE' },
      cargo: { requireAuthentication: false },
    }),
    'create',
  );
}

async function publish(server: Server, repository: string, name: string, version: string) {
  await expectOk(
    await call(server, 'PUT', `/repository/${repository}/api/v1/crates/new`, publishBody(name, version), {
      'Content-Type': 'application/octet-stream',
    }),
    'publish',
  );
}

test('a hosted Cargo registry is made through the form with its own settings', async ({ page, server }) => {
  await signInAsAdmin(page, server);
  await page.goto('/admin/repository/repositories');
  await page.getByRole('link', { name: 'Create Repository' }).first().click();
  await page.getByRole('button', { name: 'Cargo (Rust)', exact: true }).click();
  await expect(page.getByText('Manage cargo components.')).toBeVisible();
  await page.getByRole('button', { name: 'Create a hosted Cargo (Rust) repository' }).click();
  await expect(page.getByText('Cargo Settings', { exact: true })).toBeVisible();
  await page.getByRole('textbox', { name: 'Name', exact: true }).fill('crates');
  await page.getByRole('checkbox', { name: /Show the repository content to signed-in users only/ }).check();
  await page.getByRole('button', { name: 'Continue' }).click();
  await expect(page).toHaveURL(/\/admin\/repository\/repositories$/);
  const stored = await (
    await call(server, 'GET', '/service/rest/v1/repositories/cargo/hosted/crates')
  ).json();
  expect(stored.cargo.requireAuthentication).toBe(true);
  expect(stored.storage.writePolicy).toBe('ALLOW_ONCE');
});

test('a published crate shows in browse with the cargo usage and on its component page', async ({
  page,
  server,
}) => {
  await hosted(server, 'crates-browse');
  await publish(server, 'crates-browse', 'astrum-hola', '1.0.0');
  await signInAsAdmin(page, server);
  await page.goto('/browse/crates-browse');
  const tree = page.getByRole('tree');
  await tree.getByText('crates', { exact: true }).click();
  await tree.getByText('astrum-hola', { exact: true }).click();
  await tree.getByText('1.0.0', { exact: true }).click();
  await tree.getByText('astrum-hola-1.0.0.crate').click();
  await expect(page).toHaveURL(
    /\/browse\/crates-browse\/crates\/astrum-hola\/1\.0\.0\/astrum-hola-1\.0\.0\.crate$/,
  );
  const url = `${server.url}/repository/crates-browse/`;
  await expect(page.getByRole('tab', { name: 'config.toml' })).toHaveAttribute('aria-selected', 'true');
  await expect(page.getByText(`index = "sparse+${url}"`)).toBeVisible();
  await page.getByRole('tab', { name: 'cargo add' }).click();
  await expect(page.getByText('cargo add astrum-hola@1.0.0 --registry crates-browse')).toBeVisible();
  await page.getByRole('tab', { name: 'Cargo.toml' }).click();
  await expect(
    page.getByText('astrum-hola = { version = "1.0.0", registry = "crates-browse" }'),
  ).toBeVisible();
  await page.getByRole('tab', { name: 'cargo publish' }).click();
  await expect(page.getByText(/Sign in once with a token of this server/)).toBeVisible();

  await page.goto(`/search/component/${encodeURIComponent('cargo::astrum-hola')}?version=1.0.0`);
  await expect(page.getByText('cargo add astrum-hola@1.0.0')).toBeVisible();

  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/browse/crates-browse/crates/astrum-hola/1.0.0/astrum-hola-1.0.0.crate');
  await expect(page.getByRole('tab', { name: 'config.toml' })).toBeVisible();
  await noHorizontalScroll(page);
});

async function shoot(page: Page, name: string) {
  if (DIR) {
    mkdirSync(DIR, { recursive: true });
    await page.waitForTimeout(300);
    await page.screenshot({ path: join(DIR, `${name}.png`) });
  }
}

test('cargo screens', async ({ page, server }) => {
  test.skip(!DIR, 'set ASTRUM_SHOTS_DIR to take screenshots');
  test.setTimeout(240_000);
  await hosted(server, 'crates-shots');
  await publish(server, 'crates-shots', 'astrum-hola', '1.0.0');
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
      await shotPage.goto('/admin/repository/repositories/create/cargo/proxy');
      await expect(shotPage.getByRole('textbox', { name: 'Name', exact: true })).toBeVisible();
      await shoot(shotPage, `cargo-create-proxy-${suffix}`);
      await shotPage.goto('/browse/crates-shots/crates/astrum-hola/1.0.0/astrum-hola-1.0.0.crate');
      await expect(shotPage.getByRole('tab', { name: 'config.toml' })).toBeVisible();
      await shoot(shotPage, `cargo-browse-crate-${suffix}`);
      await context.close();
    }
  }
});
