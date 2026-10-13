import type { APIRequestContext, Page } from '@playwright/test';
import { AdminApi, canonical, differences, type Json } from './admin-api';
import { expect, noHorizontalScroll, signIn, test } from './fixtures';

test.describe.configure({ timeout: 120_000 });

const FORMATS = [
  { id: 'raw', api: 'raw', name: 'Raw' },
  { id: 'maven2', api: 'maven', name: 'Maven' },
  { id: 'npm', api: 'npm', name: 'npm' },
  { id: 'pypi', api: 'pypi', name: 'PyPI' },
  { id: 'docker', api: 'docker', name: 'Docker' },
] as const;

type Format = (typeof FORMATS)[number];

function api(request: APIRequestContext, server: { url: string; adminPassword: string }) {
  return new AdminApi(request, server.url, server.adminPassword);
}

async function signInAsAdmin(page: Page, password: string) {
  await page.goto('/login');
  await signIn(page, password);
  await expect(page.getByRole('button', { name: /Account menu of/ })).toBeVisible();
}

function chooseLabel(format: Format, type: string): string {
  if (type === 'hosted') {
    return `Create a hosted ${format.name} repository`;
  }
  return `Create a ${format.name} ${type} repository`;
}

async function pick(page: Page, combobox: string, option: string) {
  await page.getByRole('combobox', { name: combobox }).click();
  await page.getByRole('option', { name: option, exact: true }).click();
}

function fullDocument(
  format: Format,
  type: 'hosted' | 'proxy' | 'group',
  name: string,
  member: string,
): Json {
  const body: Json = {
    name,
    online: true,
    storage: {
      blobStoreName: 'default',
      strictContentTypeValidation: false,
      ...(type === 'hosted' ? { writePolicy: 'DENY' } : {}),
      ...(type === 'hosted' && format.id === 'docker' ? { latestPolicy: true } : {}),
    },
  };
  if (type !== 'group') {
    body.cleanup = { policyNames: [] };
  }
  if (type === 'hosted') {
    body.component = { proprietaryComponents: true };
  }
  if (type === 'proxy') {
    body.proxy = {
      remoteUrl: `https://upstream.example.invalid/${name}/`,
      contentMaxAge: 11,
      metadataMaxAge: 12,
      preserveEncodedCharacters: true,
    };
    body.negativeCache = { enabled: false, timeToLive: 13 };
    body.httpClient = {
      blocked: true,
      autoBlock: false,
      connection: {
        retries: 3,
        userAgentSuffix: 'e2e',
        timeout: 30,
        enableCircularRedirects: true,
        enableCookies: true,
        useTrustStore: true,
      },
      authentication: {
        type: 'username',
        username: 'e2e-user',
        password: 'not-a-real-password',
        ...(format.id === 'maven2' ? { preemptive: true } : {}),
      },
    };
    body.replication = { preemptivePullEnabled: true, assetPathRegex: '.*e2e' };
  }
  if (type === 'group') {
    body.group = {
      memberNames: [member],
      ...(format.id === 'docker' ? { writableMember: member } : {}),
    };
  }
  if (format.id === 'maven2' && type !== 'group') {
    body.maven = { versionPolicy: 'MIXED', layoutPolicy: 'PERMISSIVE', contentDisposition: 'INLINE' };
  }
  if (format.id === 'npm' && type === 'proxy') {
    body.npm = { removeNonCataloged: true, removeQuarantined: true };
  }
  if (format.id === 'docker') {
    body.docker = {
      v1Enabled: true,
      forceBasicAuth: false,
      pathEnabled: true,
      subdomain: `${name.replace(/[^a-z0-9]/g, '')}`,
      ...(type === 'hosted' ? { httpPort: 18498, httpsPort: 18499 } : {}),
    };
  }
  if (format.id === 'docker' && type === 'proxy') {
    body.dockerProxy = {
      indexType: 'CUSTOM',
      indexUrl: 'https://index.example.invalid/',
      cacheForeignLayers: true,
      foreignLayerUrlWhitelist: ['.*e2e', '.*layers'],
    };
  }
  if (format.id === 'raw') {
    body.raw =
      type === 'proxy'
        ? {
            contentDisposition: 'INLINE',
            forwardQueryParameters: true,
            excludedQueryParameters: ['apiKey', 'token'],
          }
        : { contentDisposition: 'INLINE' };
  }
  return body;
}

test('each format gets hosted, proxy and group repositories through the forms', async ({
  page,
  request,
  server,
}) => {
  const admin = api(request, server);
  await signInAsAdmin(page, server.adminPassword);
  for (const format of FORMATS) {
    for (const type of ['hosted', 'proxy', 'group'] as const) {
      const name = `ui-${format.id}-${type}`;
      await page.goto('/admin/repository/repositories');
      await page.getByRole('link', { name: 'Create Repository' }).first().click();
      await page.getByRole('button', { name: format.name, exact: true }).click();
      await page.getByRole('button', { name: chooseLabel(format, type) }).click();
      await page.getByRole('textbox', { name: 'Name', exact: true }).fill(name);
      if (type === 'proxy') {
        await page
          .getByRole('textbox', { name: 'Remote Storage', exact: true })
          .fill(`https://upstream.example.invalid/${format.id}/`);
      }
      if (type === 'group') {
        await pick(page, 'Member Repositories', `ui-${format.id}-hosted (hosted)`);
        await pick(page, 'Member Repositories', `ui-${format.id}-proxy (proxy)`);
        if (format.id === 'docker') {
          await pick(page, 'Writable Repository', `ui-${format.id}-hosted`);
        }
      }
      await page.getByRole('button', { name: 'Continue' }).click();
      await expect(page).toHaveURL(/\/admin\/repository\/repositories$/);
      await expect(page.getByText(`Repository "${name}" created`, { exact: true })).toBeVisible();
      const document = await admin.repository(format.api, type, name);
      expect(document.online).toBe(true);
      expect((document.storage as Json).blobStoreName).toBe('default');
      if (type === 'hosted') {
        expect((document.storage as Json).writePolicy).toBe('ALLOW_ONCE');
      }
      if (type === 'proxy') {
        expect((document.proxy as Json).remoteUrl).toBe(`https://upstream.example.invalid/${format.id}/`);
      }
      if (type === 'group') {
        expect((document.group as Json).memberNames).toEqual([
          `ui-${format.id}-hosted`,
          `ui-${format.id}-proxy`,
        ]);
        if (format.id === 'docker') {
          expect((document.group as Json).writableMember).toBe(`ui-${format.id}-hosted`);
        }
      }
      if (format.id === 'docker') {
        expect((document.docker as Json).pathEnabled).toBe(true);
      }
    }
  }
  await page.goto('/admin/repository/repositories?type=group');
  for (const format of FORMATS) {
    await expect(page.getByRole('row', { name: `Open ui-${format.id}-group` })).toBeVisible();
  }
  await expect(page.getByRole('row', { name: 'Open ui-raw-hosted' })).toHaveCount(0);
});

test('saving a form unchanged keeps every field, and an edit changes only its field', async ({
  page,
  request,
  server,
}) => {
  const admin = api(request, server);
  for (const format of FORMATS) {
    for (const type of ['hosted', 'proxy', 'group'] as const) {
      const name = `rt-${format.id}-${type}`;
      await admin.createRepository(
        format.api,
        type,
        fullDocument(format, type, name, `rt-${format.id}-hosted`),
      );
    }
  }
  await signInAsAdmin(page, server.adminPassword);
  for (const format of FORMATS) {
    for (const type of ['hosted', 'proxy', 'group'] as const) {
      const name = `rt-${format.id}-${type}`;
      const before = await admin.repository(format.api, type, name);
      await page.goto(`/admin/repository/repositories/${name}`);
      const form = page.getByRole('form', { name: 'Save Changes' });
      await expect(form).toBeVisible();
      const saved = page.waitForResponse(
        (response) => response.request().method() === 'PUT' && response.url().endsWith(`/${type}/${name}`),
      );
      await form.evaluate((element) => (element as HTMLFormElement).requestSubmit());
      expect((await saved).status()).toBe(204);
      await expect(page.getByText(`Changes to ${name} saved`, { exact: true })).toBeVisible();
      const after = await admin.repository(format.api, type, name);
      expect(canonical(after), `${name} after saving unchanged`).toEqual(canonical(before));
    }
  }

  const proxy = 'rt-raw-proxy';
  const before = await admin.repository('raw', 'proxy', proxy);
  await page.goto(`/admin/repository/repositories/${proxy}`);
  await page.getByRole('button', { name: 'HTTP Request Settings' }).click();
  await page.getByRole('spinbutton', { name: 'Connection Timeout' }).fill('45');
  await page.getByRole('button', { name: 'Save Changes', exact: true }).click();
  await expect(page.getByText(`Changes to ${proxy} saved`, { exact: true })).toBeVisible();
  const after = await admin.repository('raw', 'proxy', proxy);
  expect(differences(before, after)).toEqual(['httpClient.connection.timeout']);
  expect(((after.httpClient as Json).connection as Json).timeout).toBe(45);

  const group = 'rt-docker-group';
  const groupBefore = await admin.repository('docker', 'group', group);
  await page.goto(`/admin/repository/repositories/${group}`);
  await pick(page, 'Member Repositories', 'rt-docker-proxy (proxy)');
  await page
    .getByRole('list', { name: 'Member Repositories' })
    .getByRole('listitem')
    .filter({ hasText: 'rt-docker-proxy' })
    .getByRole('button', { name: 'Move Up' })
    .click();
  await page.getByRole('button', { name: 'Save Changes', exact: true }).click();
  await expect(page.getByText(`Changes to ${group} saved`, { exact: true })).toBeVisible();
  const groupAfter = await admin.repository('docker', 'group', group);
  expect(differences(groupBefore, groupAfter)).toEqual(['group.memberNames']);
  expect((groupAfter.group as Json).memberNames).toEqual(['rt-docker-proxy', 'rt-docker-hosted']);
});

test('server problems land on their fields and leaving asks first', async ({ page, request, server }) => {
  const admin = api(request, server);
  await admin.createRepository('raw', 'hosted', fullDocument(FORMATS[0], 'hosted', 'taken-name', ''));
  await signInAsAdmin(page, server.adminPassword);
  await page.goto('/admin/repository/repositories/create/raw/hosted');
  await page.getByRole('textbox', { name: 'Name', exact: true }).fill('TAKEN-name');
  await page.getByRole('button', { name: 'Continue' }).click();
  await expect(
    page.getByText('is already used by another repository (names ignore case)', { exact: true }),
  ).toBeVisible();
  await page.getByRole('link', { name: 'Repositories', exact: true }).click();
  const dialog = page.getByRole('alertdialog', { name: 'Leave without saving?' });
  await expect(dialog).toBeVisible();
  await dialog.getByRole('button', { name: 'Stay' }).click();
  await expect(page).toHaveURL(/\/create\/raw\/hosted$/);
});

test('a repository goes offline and online, and its cache and index can be refreshed', async ({
  page,
  request,
  server,
}) => {
  const admin = api(request, server);
  await admin.createRepository('raw', 'hosted', fullDocument(FORMATS[0], 'hosted', 'switch-hosted', ''));
  await admin.createRepository('raw', 'proxy', fullDocument(FORMATS[0], 'proxy', 'switch-proxy', ''));
  await signInAsAdmin(page, server.adminPassword);
  await page.goto('/admin/repository/repositories/switch-hosted');
  await page.getByRole('button', { name: 'Take Offline' }).click();
  await page.getByRole('alertdialog').getByRole('button', { name: 'Take Offline' }).click();
  await expect(page.getByText('Repository "switch-hosted" is offline', { exact: true })).toBeVisible();
  await expect(page.getByRole('button', { name: 'Bring Online' })).toBeVisible();
  expect((await admin.repository('raw', 'hosted', 'switch-hosted')).online).toBe(false);
  await page.getByRole('button', { name: 'Bring Online' }).click();
  await page.getByRole('alertdialog').getByRole('button', { name: 'Bring Online' }).click();
  await expect(page.getByText('Repository "switch-hosted" is online', { exact: true })).toBeVisible();
  expect((await admin.repository('raw', 'hosted', 'switch-hosted')).online).toBe(true);

  await expect(page.getByRole('button', { name: 'Invalidate Cache' })).toHaveCount(0);
  await page.getByRole('button', { name: 'Rebuild Index' }).click();
  await page.getByRole('alertdialog').getByRole('button', { name: 'Rebuild Index' }).click();
  await expect(page.getByText('Index rebuild of "switch-hosted" started', { exact: true })).toBeVisible();

  await page.goto('/admin/repository/repositories/switch-proxy');
  await page.getByRole('button', { name: 'Invalidate Cache' }).click();
  await page.getByRole('alertdialog').getByRole('button', { name: 'Invalidate Cache' }).click();
  await expect(page.getByText('The cache of "switch-proxy" was invalidated', { exact: true })).toBeVisible();
});

test('deleting asks for the name and takes the repository out of its groups', async ({
  page,
  request,
  server,
}) => {
  const admin = api(request, server);
  await admin.createRepository('raw', 'hosted', {
    ...fullDocument(FORMATS[0], 'hosted', 'doomed', ''),
    storage: { blobStoreName: 'default', strictContentTypeValidation: false, writePolicy: 'ALLOW' },
  });
  await admin.createRepository('raw', 'hosted', fullDocument(FORMATS[0], 'hosted', 'survivor', ''));
  await admin.createRepository('raw', 'group', {
    name: 'doomed-group',
    online: true,
    storage: { blobStoreName: 'default', strictContentTypeValidation: true },
    group: { memberNames: ['doomed', 'survivor'] },
  });
  await admin.upload('doomed', 'docs/readme.txt', 'goodbye');
  await signInAsAdmin(page, server.adminPassword);
  await page.goto('/admin/repository/repositories/doomed/profile?tab=membership');
  await expect(page.getByRole('link', { name: 'doomed-group' })).toBeVisible();
  await page.goto('/admin/repository/repositories/doomed');
  await page.getByRole('button', { name: 'Delete', exact: true }).click();
  const dialog = page.getByRole('alertdialog', { name: 'Delete doomed?' });
  await expect(dialog).toContainText('every component and asset published to it');
  await expect(dialog).toContainText('doomed-group');
  const confirm = dialog.getByRole('button', { name: 'Delete the repository' });
  await expect(confirm).toBeDisabled();
  await dialog.getByRole('textbox', { name: 'Type doomed to confirm' }).fill('doome');
  await expect(confirm).toBeDisabled();
  await dialog.getByRole('textbox', { name: 'Type doomed to confirm' }).fill('doomed');
  await confirm.click();
  await expect(page).toHaveURL(/\/admin\/repository\/repositories$/);
  await expect(page.getByText('Repository "doomed" deleted', { exact: true })).toBeVisible();
  await expect(page.getByRole('row', { name: 'Open doomed', exact: true })).toHaveCount(0);
  expect(await admin.status('/v1/repositories/doomed')).toBe(404);
  expect(((await admin.repository('raw', 'group', 'doomed-group')).group as Json).memberNames).toEqual([
    'survivor',
  ]);
});

test('the profile shows membership and recent audit events', async ({ page, request, server }) => {
  const admin = api(request, server);
  await admin.createRepository('raw', 'hosted', fullDocument(FORMATS[0], 'hosted', 'profile-hosted', ''));
  await admin.createRepository('raw', 'group', {
    name: 'profile-group',
    online: true,
    storage: { blobStoreName: 'default', strictContentTypeValidation: true },
    group: { memberNames: ['profile-hosted'] },
  });
  await signInAsAdmin(page, server.adminPassword);
  await page.goto('/admin/repository/repositories/profile-group/profile');
  await expect(page.getByRole('region', { name: 'Repository Configuration' })).toContainText(
    'profile-hosted',
  );
  await page.getByRole('tab', { name: /Audit/ }).click();
  await expect(page.getByRole('cell', { name: 'Created' }).first()).toBeVisible();
  await expect(page.getByRole('link', { name: 'Full Audit Log' })).toHaveAttribute(
    'href',
    '/audit?repositoryName=profile-group',
  );
  await page.goto('/#browse/repository-profile/profile-hosted');
  await expect(page).toHaveURL(/\/admin\/repository\/repositories\/profile-hosted\/profile$/);
  await page.getByRole('tab', { name: /Group Membership/ }).click();
  await expect(page.getByRole('link', { name: 'profile-group' })).toBeVisible();
  await page.setViewportSize({ width: 390, height: 844 });
  await noHorizontalScroll(page);
  await page.goto('/admin/repository/repositories');
  await expect(page.getByRole('row', { name: 'Open profile-group' })).toBeVisible();
  await noHorizontalScroll(page);
});

test('an administrator who may only read sees disabled forms', async ({ page, request, server }) => {
  const admin = api(request, server);
  await admin.createRepository('maven', 'proxy', fullDocument(FORMATS[1], 'proxy', 'read-only-proxy', ''));
  await admin.createUser('repo-reader', 'reader-Pass-1', 'astrum:repository-admin:*:*:read');
  await page.goto('/login');
  await signIn(page, 'reader-Pass-1', 'repo-reader');
  await expect(page.getByRole('button', { name: /Account menu of/ })).toBeVisible();
  await page.goto('/settings');
  await expect(page.getByRole('link', { name: /Repositories/ })).toBeVisible();
  await page.goto('/admin/repository/repositories');
  await expect(page.getByRole('row', { name: 'Open read-only-proxy' })).toBeVisible();
  await expect(page.getByRole('link', { name: 'Create Repository' })).toHaveCount(0);
  await page.getByRole('row', { name: 'Open read-only-proxy' }).click();
  await expect(
    page.getByText('You can read these settings but not change them.', { exact: true }),
  ).toBeVisible();
  await expect(page.getByRole('textbox', { name: 'Remote Storage' })).toBeDisabled();
  await expect(page.getByRole('checkbox', { name: 'Blocked' })).toBeDisabled();
  await expect(page.getByRole('combobox', { name: 'Version Policy' })).toBeDisabled();
  await expect(page.getByRole('button', { name: 'Save Changes' })).toHaveCount(0);
  await expect(page.getByRole('button', { name: 'Take Offline' })).toHaveCount(0);
  await expect(page.getByRole('button', { name: 'Delete' })).toHaveCount(0);
  await page.goto('/admin/repository/repositories/create');
  await expect(page.getByRole('heading', { name: 'You cannot open this page' })).toBeVisible();
});
