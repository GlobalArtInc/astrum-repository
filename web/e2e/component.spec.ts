import { SEED, seedBrowse } from './browse-seed';
import { expect, noHorizontalScroll, signIn, test } from './fixtures';

const PAGE = `/search/component/${encodeURIComponent('maven2:org.acme:demo-lib')}`;

test('the component page shows versions, files and usage', async ({ page, server }) => {
  await seedBrowse(server.url, server.adminPassword);
  await page.goto('/login');
  await signIn(page, server.adminPassword);
  await expect(page.getByRole('button', { name: /Account menu of/ })).toBeVisible();
  await page.goto(PAGE);
  await expect(page.getByRole('heading', { level: 1, name: 'demo-lib v2.0.0' })).toBeVisible();
  await expect(page.getByText('<version>2.0.0</version>')).toBeVisible();
  await expect(page.getByText(SEED.maven, { exact: true })).toBeVisible();

  const tabs = page.getByRole('navigation', { name: 'Component sections' });
  await tabs.getByRole('link', { name: /Versions/ }).click();
  const versions = page.getByRole('table', { name: 'Versions' });
  await expect(versions.getByRole('rowheader')).toHaveText(['2.0.0', '1.1.0', '1.0.0']);
  await versions.getByRole('link', { name: '1.0.0' }).click();
  await expect(page).toHaveURL(/\/versions\?version=1\.0\.0$/);
  await expect(page.getByRole('heading', { level: 1, name: 'demo-lib v1.0.0' })).toBeVisible();

  await tabs.getByRole('link', { name: /Files/ }).click();
  const files = page.getByRole('table', { name: 'Files' });
  await expect(files.getByRole('rowheader')).toHaveCount(2);
  await expect(files.getByText('demo-lib-1.0.0.jar')).toBeVisible();

  await tabs.getByRole('link', { name: /Overview/ }).click();
  await page.getByRole('combobox', { name: 'Choose a dependency snippet' }).click();
  await page.getByRole('option', { name: 'Gradle Kotlin DSL' }).click();
  await expect(page.getByText('implementation("org.acme:demo-lib:1.0.0")')).toBeVisible();

  await page.goto(`/search/component/${encodeURIComponent('docker::team/app')}`);
  await expect(
    page.getByText(`docker pull ${new URL(server.url).host}/${SEED.docker}/team/app:1.0`),
  ).toBeVisible();

  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto(`${PAGE}/files`);
  await expect(page.getByRole('heading', { level: 1, name: 'demo-lib v2.0.0' })).toBeVisible();
  await noHorizontalScroll(page);
});
