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
  await expect(page.getByRole('heading', { level: 1, name: 'Audit log' })).toBeVisible();
  const table = page.getByRole('table');
  await expect(table.getByRole('row').nth(1)).toContainText('audit-dropped');
  await expect(table.getByRole('row').nth(1)).toContainText('Deleted');

  await page.getByRole('combobox', { name: 'Domain' }).click();
  await page.getByRole('option', { name: 'Repository', exact: true }).click();
  await page.getByRole('combobox', { name: 'Type' }).click();
  await page.getByRole('option', { name: 'Deleted' }).click();
  await expect(page).toHaveURL(/\/audit\?domain=repository&type=deleted$/);
  await expect(table.getByRole('row')).toHaveCount(2);
  await expect(table).toContainText('audit-dropped');

  await page.goto('/audit?repositoryName=audit-kept');
  await expect(page.getByRole('combobox', { name: 'Repository' })).toHaveText('audit-kept');
  await expect(table.getByRole('row')).toHaveCount(3);
  await expect(table).toContainText('Component');
  await expect(table).toContainText('/a:/a/gone.txt:');
  await page
    .getByText(/^\d+ attributes?$/)
    .first()
    .click();
  await expect(table.getByRole('definition').filter({ hasText: 'audit-kept' }).first()).toBeVisible();

  await page.getByRole('searchbox', { name: 'Search' }).fill('dropped');
  await expect(page).toHaveURL(/q=dropped/);
  await expect(page.getByText('No events match the filters')).toBeVisible();
  await page.getByRole('button', { name: 'Reset filters' }).click();
  await expect(page).toHaveURL(/\/audit$/);
  await page.getByRole('combobox', { name: 'Initiator' }).click();
  await page.getByRole('option', { name: 'admin' }).click();
  await expect(page).toHaveURL(/initiator=admin/);
  const today = new Date().toISOString().slice(0, 10);
  await page.getByLabel('From', { exact: true }).fill(today);
  await expect(page).toHaveURL(new RegExp(`since=${today}`));
  await expect(table).toContainText('audit-dropped');
  await page.getByLabel('From', { exact: true }).fill('2000-01-01');
  await page.getByLabel('To', { exact: true }).fill('2000-01-02');
  await expect(page.getByText('No events match the filters')).toBeVisible();
});

test('the audit log fits a phone', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/audit');
  await expect(page.getByRole('heading', { level: 1, name: 'Audit log' })).toBeVisible();
  await page.getByRole('button', { name: /^Filters/ }).click();
  await expect(page.getByRole('combobox', { name: 'Domain' })).toBeVisible();
  await noHorizontalScroll(page);
});
