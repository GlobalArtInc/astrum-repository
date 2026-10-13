import { call, ensure, hosted, put } from './content';
import { expect, noHorizontalScroll, signIn, test } from './fixtures';

test.beforeEach(async ({ page, server }) => {
  await page.goto('/login');
  await signIn(page, server.adminPassword);
  await expect(page.getByRole('heading', { level: 1, name: 'Overview' })).toBeVisible();
});

test('the audit log shows changes and filters them', async ({ page, server }) => {
  await hosted(server, 'raw', 'audit-kept', { raw: { contentDisposition: 'ATTACHMENT' } });
  await put(server, '/repository/audit-kept/a/gone.txt', 'gone');
  const found = await call(server, 'GET', '/service/rest/v1/search?repository=audit-kept');
  const id = (await found.json()).items[0].id as string;
  await ensure(await call(server, 'DELETE', `/service/rest/v1/components/${id}`), 'delete component');
  await hosted(server, 'raw', 'audit-dropped', { raw: { contentDisposition: 'ATTACHMENT' } });
  await ensure(
    await call(server, 'DELETE', '/service/rest/v1/repositories/audit-dropped'),
    'delete repository',
  );

  await page.goto('/audit');
  await expect(page).toHaveURL(/\/audit$/);
  await expect(page.getByRole('heading', { level: 1, name: 'Audit Log' })).toBeVisible();
  await expect(page.getByText(/^\d+ events$/)).toBeVisible();
  const table = page.getByRole('table');
  const first = table.getByRole('row').nth(1);
  await expect(first).toContainText('Repository');
  await expect(first).toContainText('Deleted');
  await expect(first).toContainText('Repository “audit-dropped” was deleted (initiator: admin');
  await expect(table).toContainText('User “admin”: signed in');

  const filters = page.getByRole('complementary', { name: 'Audit log filters' });
  await filters.getByRole('checkbox', { name: 'Repository' }).click();
  await filters.getByRole('checkbox', { name: 'Deleted' }).click();
  await expect(page).toHaveURL(/\/audit\?categories=repository&types=deleted$/);
  await expect(table).toContainText('audit-dropped');
  await expect(table).not.toContainText('signed in');

  await page.goto('/audit?repositoryName=audit-kept');
  await expect(filters.getByRole('combobox', { name: 'Repository' })).toHaveText('audit-kept');
  await expect(page.getByText('2 events')).toBeVisible();
  await expect(table).toContainText('Component “/a:/a/gone.txt:” was deleted');
  await table.getByRole('button', { name: 'Show details' }).first().click();
  await expect(page.getByRole('heading', { name: 'Event Details' })).toBeVisible();
  await expect(table).toContainText('"repository": "audit-kept"');

  await page.getByRole('textbox', { name: 'Search by context' }).fill('dropped');
  await expect(page).toHaveURL(/q=dropped/);
  await expect(page.getByText('No audit events found')).toBeVisible();
  await filters.getByRole('button', { name: 'Reset Filters' }).click();
  await expect(page).toHaveURL(/\/audit$/);
  await filters.getByRole('checkbox', { name: 'Sign-in' }).click();
  await expect(table).toContainText('User “admin”: signed in');
  await expect(table).not.toContainText('audit-dropped');
  await filters.getByRole('textbox', { name: 'Initiator' }).fill('nobody-like-this');
  await expect(page).toHaveURL(/initiator=nobody-like-this/);
  await expect(page.getByText('No audit events found')).toBeVisible();
  await filters.getByRole('button', { name: 'Reset Filters' }).click();
  await filters.getByRole('combobox', { name: 'Period' }).click();
  await page.getByRole('option', { name: 'Last 24 hours' }).click();
  await expect(page).toHaveURL(/period=1/);
  await expect(table).toContainText('audit-dropped');
});

test('the audit log pages through many events', async ({ page, server }) => {
  for (let index = 0; index < 23; index += 1) {
    await hosted(server, 'raw', `audit-page-${index}`, { raw: { contentDisposition: 'ATTACHMENT' } });
  }
  await page.goto('/audit');
  await expect(page.getByRole('navigation', { name: /Pagination, page 1 of \d+/ })).toBeVisible();
  await expect(page.getByRole('table').getByRole('button', { name: 'Show details' })).toHaveCount(20);
  await page.getByRole('button', { name: 'Next page' }).click();
  await expect(page).toHaveURL(/page=2/);
  await expect(page.getByRole('navigation', { name: /Pagination, page 2 of/ })).toBeVisible();
  await page.getByRole('combobox', { name: 'Rows per page' }).click();
  await page.getByRole('option', { name: '50', exact: true }).click();
  await expect(page).toHaveURL(/size=50/);
  await expect(page.getByRole('navigation', { name: 'Pagination, page 1 of 1' })).toBeVisible();
});

test('the audit log fits a phone', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/audit');
  await expect(page.getByRole('heading', { level: 1, name: 'Audit Log' })).toBeVisible();
  await page.getByRole('button', { name: 'Open filters' }).click();
  const sheet = page.getByRole('dialog', { name: 'Filter' });
  await expect(sheet.getByRole('combobox', { name: 'Period' })).toBeVisible();
  await sheet.getByRole('button', { name: 'Close' }).click();
  await noHorizontalScroll(page);
});
