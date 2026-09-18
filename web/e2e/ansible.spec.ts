import { mkdirSync } from 'node:fs';
import { join } from 'node:path';
import { gzipSync } from 'node:zlib';
import type { Page } from '@playwright/test';
import { call, tar } from './content';
import { expect, noHorizontalScroll, type Server, signIn, test } from './fixtures';

test.describe.configure({ timeout: 120_000 });

const DIR = process.env.ASTRUM_SHOTS_DIR;
const FILE = /^File\*?$/;

function collection(namespace: string, name: string, version: string): Buffer {
  const manifest = {
    collection_info: {
      namespace,
      name,
      version,
      authors: ['Astrum Tests <e2e@example.org>'],
      readme: 'README.md',
      tags: ['demo'],
      description: 'A collection of the end-to-end tests',
      license: ['MIT'],
      license_file: null,
      dependencies: { 'community.general': '>=8.0.0' },
      repository: 'https://example.org/acme/tools',
      documentation: null,
      homepage: null,
      issues: null,
    },
    file_manifest_file: {
      name: 'FILES.json',
      ftype: 'file',
      chksum_type: 'sha256',
      chksum_sha256: '0',
      format: 1,
    },
    format: 1,
  };
  return gzipSync(
    tar([
      ['MANIFEST.json', JSON.stringify(manifest)],
      ['FILES.json', '{"files": [], "format": 1}'],
      ['README.md', '# acme.tools\n'],
      ['meta/runtime.yml', "requires_ansible: '>=2.15.0'\n"],
      ['plugins/modules/hello.py', 'print("hello")\n'],
    ]),
  );
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
    await call(server, 'POST', '/service/rest/v1/repositories/ansible/hosted', {
      name,
      online: true,
      storage: { blobStoreName: 'default', strictContentTypeValidation: true, writePolicy: 'ALLOW_ONCE' },
    }),
    'create',
  );
}

async function upload(server: Server, repository: string, data: Buffer, fileName: string) {
  const form = new FormData();
  form.append('ansible.asset', new Blob([new Uint8Array(data)]), fileName);
  await expectOk(
    await call(server, 'POST', `/service/rest/v1/components?repository=${repository}`, form),
    'upload',
  );
}

test('an Ansible collection goes up through the upload page and shows in browse with its usage', async ({
  page,
  server,
}) => {
  await hosted(server, 'ansible-hosted');
  await signInAsAdmin(page, server);
  await page.goto('/upload/ansible-hosted');
  await page
    .getByLabel(FILE)
    .and(page.locator('input[type=file]'))
    .setInputFiles({
      name: 'acme-tools-1.0.0.tar.gz',
      mimeType: 'application/gzip',
      buffer: collection('acme', 'tools', '1.0.0'),
    });
  await page.getByRole('button', { name: 'Upload Component' }).click();
  await expect(page).toHaveURL(/\/upload$/);
  const url = `${server.url}/repository/ansible-hosted/`;
  const version = await (
    await call(server, 'GET', '/repository/ansible-hosted/api/v3/collections/acme/tools/versions/1.0.0/')
  ).json();
  expect(version.download_url).toBe(`${url}acme/tools/1.0.0/acme-tools-1.0.0.tar.gz`);
  expect(version.metadata.dependencies).toEqual({ 'community.general': '>=8.0.0' });
  expect(version.requires_ansible).toBe('>=2.15.0');

  await page.goto('/browse/ansible-hosted');
  const tree = page.getByRole('tree');
  await tree.getByText('acme', { exact: true }).click();
  await tree.getByText('tools', { exact: true }).click();
  await tree.getByText('1.0.0', { exact: true }).click();
  await tree.getByText('acme-tools-1.0.0.tar.gz').click();
  await expect(page.getByRole('tab', { name: 'ansible.cfg' })).toHaveAttribute('aria-selected', 'true');
  await expect(page.getByText('server_list = ansible_hosted')).toBeVisible();
  await page.getByRole('tab', { name: 'ansible-galaxy' }).click();
  await expect(page.getByText(`ansible-galaxy collection install acme.tools:1.0.0 -s ${url}`)).toBeVisible();
  await page.getByRole('tab', { name: 'requirements.yml' }).click();
  await expect(page.getByText(`source: ${url}`)).toBeVisible();
  await page.getByRole('tab', { name: 'Upload' }).click();
  await expect(
    page.getByText(
      `ansible-galaxy collection publish acme-tools-1.0.0.tar.gz -s ${url} --token <access token>`,
    ),
  ).toBeVisible();

  await page.goto(`/search/component/${encodeURIComponent('ansible:acme:tools')}?version=1.0.0`);
  await expect(page.getByText(`ansible-galaxy collection install acme.tools:1.0.0 -s ${url}`)).toBeVisible();

  await page.goto('/search/custom');
  await expect(page.getByText('Ansible Tag').first()).toBeAttached();

  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/browse/ansible-hosted/acme/tools/1.0.0/acme-tools-1.0.0.tar.gz');
  await expect(page.getByRole('tab', { name: 'ansible.cfg' })).toBeVisible();
  await noHorizontalScroll(page);
});

test('an Ansible proxy and an Ansible group are made through the form', async ({ page, server }) => {
  await hosted(server, 'ansible-own');
  await signInAsAdmin(page, server);
  await page.goto('/admin/repository/repositories/create');
  await page.getByRole('button', { name: 'Ansible Galaxy', exact: true }).click();
  await expect(
    page.getByText('Collections of Ansible modules, plugins and roles, served through the Galaxy API.'),
  ).toBeVisible();
  await page.goto('/admin/repository/repositories/create/ansible/proxy');
  await expect(page.getByText(/https:\/\/galaxy\.ansible\.com/)).toBeVisible();
  await page.getByRole('textbox', { name: 'Name', exact: true }).fill('galaxy');
  await page.getByRole('textbox', { name: /Remote Storage/ }).fill('https://galaxy.ansible.com');
  await page.getByRole('button', { name: 'Continue' }).click();
  await expect(page).toHaveURL(/\/admin\/repository\/repositories$/);
  const stored = await (
    await call(server, 'GET', '/service/rest/v1/repositories/ansible/proxy/galaxy')
  ).json();
  expect(stored.proxy.remoteUrl).toBe('https://galaxy.ansible.com');
  await expectOk(
    await call(server, 'POST', '/service/rest/v1/repositories/ansible/group', {
      name: 'ansible-all',
      online: true,
      storage: { blobStoreName: 'default', strictContentTypeValidation: true },
      group: { memberNames: ['ansible-own'] },
    }),
    'group',
  );
  await upload(server, 'ansible-own', collection('acme', 'base', '2.0.0'), 'acme-base-2.0.0.tar.gz');
  await page.goto('/admin/repository/repositories/ansible-all');
  await expect(page.getByRole('textbox', { name: 'Name', exact: true })).toHaveValue('ansible-all');
  const root = await (await call(server, 'GET', '/repository/ansible-all/api/')).json();
  expect(root.available_versions.v3).toBe('v3/');
  const versions = await (
    await call(server, 'GET', '/repository/ansible-all/api/v3/collections/acme/base/versions/')
  ).json();
  expect(versions.meta.count).toBe(1);
  expect(versions.data[0].href).toBe('/repository/ansible-all/api/v3/collections/acme/base/versions/2.0.0/');
});

async function shoot(page: Page, name: string) {
  if (DIR) {
    mkdirSync(DIR, { recursive: true });
    await page.waitForTimeout(300);
    await page.screenshot({ path: join(DIR, `${name}.png`) });
  }
}

test('ansible screens', async ({ page, server }) => {
  test.skip(!DIR, 'set ASTRUM_SHOTS_DIR to take screenshots');
  test.setTimeout(240_000);
  await hosted(server, 'ansible-shots');
  await upload(server, 'ansible-shots', collection('acme', 'tools', '1.0.0'), 'acme-tools-1.0.0.tar.gz');
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
      await shotPage.goto('/upload/ansible-shots');
      await expect(shotPage.getByLabel(FILE).and(shotPage.locator('input[type=file]'))).toBeAttached();
      await shoot(shotPage, `ansible-upload-${suffix}`);
      await shotPage.goto('/browse/ansible-shots/acme/tools/1.0.0/acme-tools-1.0.0.tar.gz');
      await expect(shotPage.getByRole('tab', { name: 'ansible.cfg' })).toBeVisible();
      await shoot(shotPage, `ansible-browse-collection-${suffix}`);
      await shotPage.goto('/admin/repository/repositories/create');
      await expect(shotPage.getByRole('button', { name: 'Ansible Galaxy', exact: true })).toBeVisible();
      await shoot(shotPage, `ansible-create-picker-${suffix}`);
      await shotPage.goto('/admin/repository/repositories/create/ansible/proxy');
      await expect(shotPage.getByText(/https:\/\/galaxy\.ansible\.com/)).toBeVisible();
      await shoot(shotPage, `ansible-create-proxy-${suffix}`);
      await context.close();
    }
  }
});
