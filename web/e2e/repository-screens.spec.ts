import { mkdirSync } from 'node:fs';
import { join } from 'node:path';
import type { Page } from '@playwright/test';
import { AdminApi, type Json } from './admin-api';
import { expect, signIn, test } from './fixtures';

const DIR = process.env.ASTRUM_SHOTS_DIR;
const SIZES = [
  { name: '1440', width: 1440, height: 900 },
  { name: '390', width: 390, height: 844 },
] as const;
const THEMES = ['dark', 'light'] as const;

test.skip(!DIR, 'set ASTRUM_SHOTS_DIR to take screenshots');
test.describe.configure({ timeout: 120_000 });

async function shoot(page: Page, name: string) {
  if (DIR) {
    mkdirSync(DIR, { recursive: true });
    await page.waitForTimeout(300);
    await page.screenshot({ path: join(DIR, `${name}.png`) });
  }
}

async function ensureRepository(admin: AdminApi, format: string, type: string, body: Json) {
  if ((await admin.status(`/v1/repositories/${String(body.name)}`)) === 404) {
    await admin.createRepository(format, type, body);
  }
}

async function ensureStore(admin: AdminApi, type: string, body: Json) {
  if ((await admin.status(`/v1/blobstores/${type}/${String(body.name)}`)) === 404) {
    await admin.send('POST', `/v1/blobstores/${type}`, body);
  }
}

async function seed(admin: AdminApi) {
  const storage = { blobStoreName: 'default', strictContentTypeValidation: true };
  await ensureRepository(admin, 'maven', 'hosted', {
    name: 'maven-releases',
    online: true,
    storage: { ...storage, writePolicy: 'ALLOW_ONCE' },
    maven: { versionPolicy: 'RELEASE', layoutPolicy: 'STRICT' },
  });
  await ensureRepository(admin, 'maven', 'proxy', {
    name: 'maven-central',
    online: true,
    storage,
    proxy: { remoteUrl: 'https://repo1.maven.org/maven2/', contentMaxAge: -1, metadataMaxAge: 1440 },
    negativeCache: { enabled: true, timeToLive: 1440 },
    httpClient: {
      blocked: true,
      autoBlock: true,
      authentication: { type: 'username', username: 'mirror', password: 'not-a-real-password' },
    },
    maven: { versionPolicy: 'RELEASE', layoutPolicy: 'PERMISSIVE' },
  });
  await ensureRepository(admin, 'maven', 'group', {
    name: 'maven-public',
    online: true,
    storage,
    group: { memberNames: ['maven-releases', 'maven-central'] },
  });
  await ensureRepository(admin, 'raw', 'hosted', {
    name: 'raw-files',
    online: true,
    storage: { ...storage, writePolicy: 'ALLOW' },
  });
  await ensureRepository(admin, 'npm', 'hosted', {
    name: 'npm-internal',
    online: false,
    storage: { ...storage, writePolicy: 'ALLOW_ONCE' },
  });
  await ensureRepository(admin, 'docker', 'hosted', {
    name: 'docker-internal',
    online: true,
    storage: { ...storage, writePolicy: 'ALLOW', latestPolicy: true },
    docker: { v1Enabled: false, forceBasicAuth: true, pathEnabled: true },
  });
  await admin.upload('raw-files', 'docs/readme.txt', 'Screens of the repository pages');
  await ensureStore(admin, 'file', { name: 'archive-a', path: 'archive-a' });
  await ensureStore(admin, 'file', { name: 'archive-b', path: 'archive-b' });
  await ensureStore(admin, 'file', { name: 'spare', path: 'spare' });
  await ensureStore(admin, 'group', {
    name: 'archive',
    members: ['archive-a', 'archive-b'],
    fillPolicy: 'roundRobin',
    softQuota: { type: 'spaceUsedQuota', limit: 500 * 1024 * 1024 },
  });
}

for (const size of SIZES) {
  for (const theme of THEMES) {
    test(`repository and blob store screens at ${size.name} in ${theme}`, async ({
      page,
      request,
      server,
    }) => {
      await seed(new AdminApi(request, server.url, server.adminPassword));
      await page.setViewportSize({ width: size.width, height: size.height });
      await page.addInitScript(
        (value) => window.localStorage.setItem('astrum-theme-preference', value),
        theme,
      );
      await page.goto('/login');
      await signIn(page, server.adminPassword);
      await expect(page.getByRole('button', { name: /Account menu of/ })).toBeVisible();
      const suffix = `${size.name}-${theme}`;

      await page.goto('/admin/repository/repositories');
      await expect(page.getByRole('link', { name: 'maven-central' })).toBeVisible();
      await shoot(page, `repositories-${suffix}`);

      await page.goto('/admin/repository/repositories/create');
      await expect(page.getByRole('link', { name: 'Create a Maven proxy repository' })).toBeVisible();
      await shoot(page, `repositories-create-${suffix}`);

      await page.goto('/admin/repository/repositories/create/maven2/proxy');
      await page.getByRole('textbox', { name: 'Name', exact: true }).fill('maven-google');
      await page.getByRole('button', { name: 'Create repository' }).click();
      await expect(
        page.getByText('Enter the address of the remote repository', { exact: true }),
      ).toBeVisible();
      await shoot(page, `repository-create-form-${suffix}`);

      await page.goto('/admin/repository/repositories/maven-central');
      await expect(page.getByRole('textbox', { name: 'Remote storage URL', exact: true })).toBeVisible();
      await shoot(page, `repository-proxy-${suffix}`);
      await page.getByRole('textbox', { name: 'User-Agent suffix' }).scrollIntoViewIfNeeded();
      await page.getByRole('combobox', { name: 'Authentication type' }).scrollIntoViewIfNeeded();
      await shoot(page, `repository-proxy-client-${suffix}`);

      await page.goto('/admin/repository/repositories/maven-public');
      await page.getByRole('list', { name: 'Members' }).scrollIntoViewIfNeeded();
      await shoot(page, `repository-group-${suffix}`);

      await page.goto('/admin/repository/repositories/maven-releases?tab=membership');
      await expect(page.getByRole('list', { name: 'Groups that contain it' })).toBeVisible();
      await shoot(page, `repository-membership-${suffix}`);

      await page.goto('/admin/repository/repositories/maven-releases?tab=audit');
      await expect(page.getByText('Created', { exact: true }).first()).toBeVisible();
      await shoot(page, `repository-audit-${suffix}`);

      await page.goto('/admin/repository/repositories/raw-files');
      await page.getByRole('button', { name: 'More actions' }).click();
      await shoot(page, `repository-actions-${suffix}`);
      await page.getByRole('menuitem', { name: 'Delete' }).click();
      await page.getByRole('textbox', { name: 'Type raw-files to confirm' }).fill('raw-fi');
      await shoot(page, `repository-delete-${suffix}`);
      await page.keyboard.press('Escape');

      await page.goto('/admin/repository/blobstores');
      await expect(page.getByRole('link', { name: 'archive', exact: true })).toBeVisible();
      await shoot(page, `blobstores-${suffix}`);

      await page.goto('/admin/repository/blobstores/create');
      await shoot(page, `blobstores-create-${suffix}`);

      await page.goto('/admin/repository/blobstores/create/s3');
      await expect(page.getByRole('textbox', { name: 'Bucket name' })).toBeVisible();
      await shoot(page, `blobstore-create-s3-${suffix}`);

      await page.goto('/admin/repository/blobstores/create/group');
      await page.getByRole('textbox', { name: 'Name', exact: true }).fill('new-pool');
      await shoot(page, `blobstore-create-group-${suffix}`);

      await page.goto('/admin/repository/blobstores/file/default');
      await expect(page.getByRole('textbox', { name: 'Path', exact: true })).toBeVisible();
      await shoot(page, `blobstore-file-${suffix}`);

      await page.goto('/admin/repository/blobstores/group/archive');
      await expect(page.getByRole('list', { name: 'Member stores' })).toBeVisible();
      await shoot(page, `blobstore-group-${suffix}`);
      await page.getByRole('list', { name: 'Member stores' }).scrollIntoViewIfNeeded();
      await shoot(page, `blobstore-group-members-${suffix}`);

      await page.goto('/settings');
      await expect(page.getByRole('link', { name: /Blob stores/ })).toBeVisible();
      await shoot(page, `settings-repository-${suffix}`);
    });
  }
}
