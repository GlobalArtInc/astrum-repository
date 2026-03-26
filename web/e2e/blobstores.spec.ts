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
  await page.getByRole('link', { name: 'Create Blob Store' }).click();
  await page.getByRole('button', { name: /^File/ }).click();
  await page.getByRole('button', { name: 'Continue' }).click();
  await expect(page.getByRole('heading', { level: 2, name: 'Basics' })).toBeVisible();
  await page.getByRole('textbox', { name: 'Name', exact: true }).fill('e2e-files');
  await page.getByRole('textbox', { name: 'Path', exact: true }).fill('e2e-files');
  await page.getByRole('button', { name: 'Continue' }).click();
  await expect(page.getByText('No Credentials Needed')).toBeVisible();
  await page.getByRole('button', { name: 'Continue' }).click();
  await page.getByRole('checkbox', { name: 'Enable Soft Quota' }).click();
  await page.getByRole('spinbutton', { name: 'Constraint Limit (MB)' }).fill('5');
  await page.getByRole('button', { name: 'Create Blob Store' }).click();
  await expect(page).toHaveURL(/\/admin\/repository\/blobstores$/);
  await expect(page.getByText('Blob store "e2e-files" created', { exact: true })).toBeVisible();
  expect(await admin.get('/v1/blobstores/file/e2e-files')).toEqual({
    path: 'e2e-files',
    softQuota: { type: 'spaceUsedQuota', limit: 5 * MB },
  });

  await page.getByRole('row', { name: 'Open e2e-files' }).click();
  await expect(page.getByRole('heading', { level: 1, name: 'Edit: e2e-files' })).toBeVisible();
  await expect(page.getByRole('spinbutton', { name: 'Constraint Limit (MB)' })).toHaveValue('5');
  await page.getByRole('spinbutton', { name: 'Constraint Limit (MB)' }).fill('7');
  await pick(page, 'Constraint Type', 'Space remaining');
  await page.getByRole('button', { name: 'Save', exact: true }).click();
  await expect(page.getByText('Blob store "e2e-files" updated', { exact: true })).toBeVisible();
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
  await expect(page.getByText('Blob store "e2e-files" deleted', { exact: true })).toBeVisible();
  expect(await admin.status('/v1/blobstores/file/e2e-files')).toBe(404);
});

test('a store in use cannot be deleted', async ({ page, request, server }) => {
  await api(request, server).createRepository('raw', 'hosted', {
    name: 'on-default',
    online: true,
    storage: { blobStoreName: 'default', strictContentTypeValidation: true, writePolicy: 'ALLOW' },
  });
  await signInAsAdmin(page, server.adminPassword);
  await page.goto('/admin/repository/blobstores/file/default');
  await expect(page.getByRole('heading', { level: 1, name: 'Edit: default' })).toBeVisible();
  await expect(page.getByText('File Blob Store', { exact: true })).toBeVisible();
  await expect(page.getByRole('button', { name: 'Delete' })).toBeDisabled();
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
  await pick(page, 'Member stores', 'pool-a (File)');
  await pick(page, 'Member stores', 'pool-b (File)');
  await pick(page, 'Fill policy', 'Each member in turn');
  await page.getByRole('button', { name: 'Continue' }).click();
  await page.getByRole('button', { name: 'Continue' }).click();
  await page.getByRole('button', { name: 'Create Blob Store' }).click();
  await expect(page).toHaveURL(/\/admin\/repository\/blobstores$/);
  expect(await admin.get('/v1/blobstores/group/e2e-pool')).toEqual({
    name: 'e2e-pool',
    softQuota: null,
    members: ['pool-a', 'pool-b'],
    fillPolicy: 'roundRobin',
  });

  await page.goto('/admin/repository/blobstores/group/e2e-pool');
  const members = page.getByRole('list', { name: 'Member stores' });
  await members
    .getByRole('listitem')
    .filter({ hasText: 'pool-b' })
    .getByRole('button', { name: 'Move Up' })
    .click();
  await expect(members.getByRole('listitem').first()).toContainText('pool-b');
  await pick(page, 'Fill policy', 'The first member that can take it');
  await page.getByRole('button', { name: 'Save', exact: true }).click();
  await expect(page.getByText('Blob store "e2e-pool" updated', { exact: true })).toBeVisible();
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
  await page
    .getByRole('list', { name: 'Member stores' })
    .getByRole('listitem')
    .filter({ hasText: holder ?? '' })
    .getByRole('button', { name: 'Remove' })
    .click();
  await page.getByRole('button', { name: 'Save', exact: true }).click();
  await expect(
    page.getByText(`${holder} holds 1 blobs of the group; removing it would lose them`),
  ).toBeVisible();
  expect(((await admin.get('/v1/blobstores/group/e2e-pool')) as Json).members).toEqual(['pool-b', 'pool-a']);
  await page.getByRole('button', { name: 'Discard Changes' }).click();
  await expect(page.getByRole('list', { name: 'Member stores' }).getByRole('listitem')).toHaveCount(2);

  await page.goto('/admin/repository/blobstores');
  await expect(page.getByRole('row', { name: 'Open pool-a' })).toBeVisible();
});
