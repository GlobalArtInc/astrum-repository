import type { APIRequestContext, Page } from '@playwright/test';
import { AdminApi, type Json } from './admin-api';
import { expect, signIn, test } from './fixtures';

test.describe.configure({ timeout: 90_000 });

const MB = 1024 * 1024;

function api(request: APIRequestContext, server: { url: string; adminPassword: string }) {
  return new AdminApi(request, server.url, server.adminPassword);
}

async function signInAsAdmin(page: Page, password: string) {
  await page.goto('/login');
  await signIn(page, password);
  await expect(page.getByRole('button', { name: /Account menu of/ })).toBeVisible();
}

async function pick(page: Page, combobox: string, option: string) {
  await page.getByRole('combobox', { name: combobox }).click();
  await page.getByRole('option', { name: option, exact: true }).click();
}

interface UiStore {
  name: string;
  blobCount: number;
}

test('a file blob store is created, changed and deleted', async ({ page, request, server }) => {
  const admin = api(request, server);
  await signInAsAdmin(page, server.adminPassword);
  await page.goto('/admin/repository/blobstores');
  await page.getByRole('link', { name: 'Create a blob store' }).click();
  await page.getByRole('link', { name: /^File/ }).click();
  await expect(page.getByRole('heading', { level: 1, name: 'New file blob store' })).toBeVisible();
  await page.getByRole('textbox', { name: 'Name', exact: true }).fill('e2e-files');
  await expect(page.getByRole('textbox', { name: 'Path', exact: true })).toHaveValue('e2e-files');
  await page.getByRole('switch', { name: 'Enable a soft quota' }).click();
  await page.getByRole('spinbutton', { name: 'Limit (MB)' }).fill('5');
  await page.getByRole('button', { name: 'Create blob store' }).click();
  await expect(page).toHaveURL(/\/admin\/repository\/blobstores\/file\/e2e-files$/);
  await expect(page.getByText('Blob store e2e-files created', { exact: true })).toBeVisible();
  expect(await admin.get('/v1/blobstores/file/e2e-files')).toEqual({
    path: 'e2e-files',
    softQuota: { type: 'spaceUsedQuota', limit: 5 * MB },
  });

  await expect(page.getByRole('spinbutton', { name: 'Limit (MB)' })).toHaveValue('5');
  await page.getByRole('spinbutton', { name: 'Limit (MB)' }).fill('7');
  await pick(page, 'Quota type', 'Space remaining');
  await page.getByRole('button', { name: 'Save', exact: true }).click();
  await expect(page.getByText('Changes to e2e-files saved', { exact: true })).toBeVisible();
  expect(await admin.get('/v1/blobstores/file/e2e-files')).toEqual({
    path: 'e2e-files',
    softQuota: { type: 'spaceRemainingQuota', limit: 7 * MB },
  });

  await page.getByRole('button', { name: 'Delete' }).click();
  await page
    .getByRole('alertdialog', { name: 'Delete the blob store e2e-files?' })
    .getByRole('button', { name: 'Delete' })
    .click();
  await expect(page).toHaveURL(/\/admin\/repository\/blobstores$/);
  await expect(page.getByText('Blob store e2e-files deleted', { exact: true })).toBeVisible();
  expect(await admin.status('/v1/blobstores/file/e2e-files')).toBe(404);
});

test('a store in use explains why it cannot be deleted', async ({ page, request, server }) => {
  await api(request, server).createRepository('raw', 'hosted', {
    name: 'on-default',
    online: true,
    storage: { blobStoreName: 'default', strictContentTypeValidation: true, writePolicy: 'ALLOW' },
  });
  await signInAsAdmin(page, server.adminPassword);
  await page.goto('/admin/repository/blobstores/file/default');
  await expect(page.getByRole('heading', { level: 1, name: 'default' })).toBeVisible();
  await expect(page.getByRole('button', { name: 'Delete' })).toBeDisabled();
  await expect(
    page.getByText(
      /^It cannot be deleted while it is in use\. The repositories (.+, )?on-default(, .+)? use it\.$/,
    ),
  ).toBeVisible();
});

test('a group store keeps its members in order and refuses to lose blobs', async ({
  page,
  request,
  server,
}) => {
  const admin = api(request, server);
  await admin.send('POST', '/v1/blobstores/file', { name: 'pool-a', path: 'pool-a' });
  await admin.send('POST', '/v1/blobstores/file', { name: 'pool-b', path: 'pool-b' });
  await signInAsAdmin(page, server.adminPassword);
  await page.goto('/admin/repository/blobstores/create/group');
  await page.getByRole('textbox', { name: 'Name', exact: true }).fill('e2e-pool');
  await pick(page, 'Add a blob store…', 'pool-a');
  await pick(page, 'Add a blob store…', 'pool-b');
  await pick(page, 'Fill policy', 'Each member in turn');
  await page.getByRole('button', { name: 'Create blob store' }).click();
  await expect(page).toHaveURL(/\/admin\/repository\/blobstores\/group\/e2e-pool$/);
  expect(await admin.get('/v1/blobstores/group/e2e-pool')).toEqual({
    name: 'e2e-pool',
    softQuota: null,
    members: ['pool-a', 'pool-b'],
    fillPolicy: 'roundRobin',
  });

  const members = page.getByRole('list', { name: 'Member stores' });
  await members
    .getByRole('listitem')
    .filter({ hasText: 'pool-b' })
    .dragTo(members.getByRole('listitem').filter({ hasText: 'pool-a' }));
  await expect(members.getByRole('listitem').first()).toContainText('pool-b');
  await pick(page, 'Fill policy', 'The first member that can take it');
  await page.getByRole('button', { name: 'Save', exact: true }).click();
  await expect(page.getByText('Changes to e2e-pool saved', { exact: true })).toBeVisible();
  expect(((await admin.get('/v1/blobstores/group/e2e-pool')) as Json).members).toEqual(['pool-b', 'pool-a']);

  await admin.createRepository('raw', 'hosted', {
    name: 'pooled-files',
    online: true,
    storage: { blobStoreName: 'e2e-pool', strictContentTypeValidation: true, writePolicy: 'ALLOW' },
  });
  await admin.upload('pooled-files', 'one.txt', 'kept in a member');
  const stores = await admin.get<UiStore[]>('/internal/ui/blobstores');
  const holder = stores.find((store) => store.name.startsWith('pool-') && store.blobCount > 0)?.name;
  expect(holder).toBeDefined();
  await page.reload();
  await page.getByRole('button', { name: `Remove ${holder}` }).click();
  await page.getByRole('button', { name: 'Save', exact: true }).click();
  await expect(
    page.getByText(`${holder} holds 1 blobs of the group; removing it would lose them`),
  ).toBeVisible();
  expect(((await admin.get('/v1/blobstores/group/e2e-pool')) as Json).members).toEqual(['pool-b', 'pool-a']);
  await page.getByRole('button', { name: 'Discard changes' }).click();
  await expect(page.getByRole('list', { name: 'Member stores' }).getByRole('listitem')).toHaveCount(2);

  await page.goto('/admin/repository/blobstores');
  const row = page.getByRole('row').filter({ has: page.getByRole('link', { name: 'pool-a', exact: true }) });
  await expect(row).toContainText('member of e2e-pool');
});
