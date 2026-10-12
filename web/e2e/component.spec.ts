import { SEED, seedBrowse } from './browse-seed';
import { expect, noHorizontalScroll, signIn, test } from './fixtures';

const PAGE = `/search/component/${encodeURIComponent('maven2:org.acme:demo-lib')}`;

test('the component page shows versions, files and usage', async ({ page, server }) => {
  await seedBrowse(server.url, server.adminPassword);
  await page.goto('/login');
  await signIn(page, server.adminPassword);
  await expect(page.getByRole('button', { name: /Account menu of/ })).toBeVisible();
  await page.goto(PAGE);
  await expect(page.getByRole('heading', { level: 1, name: 'org.acme:demo-lib' })).toBeVisible();
  const picker = page.getByRole('combobox', { name: 'Version' });
  await expect(picker).toHaveText('2.0.0');
  await expect(page.getByRole('link', { name: SEED.maven })).toBeVisible();

  await page.getByRole('tab', { name: /Versions/ }).click();
  const versions = page.getByRole('table', { name: 'Versions' });
  await expect(versions.getByRole('rowheader')).toHaveText([/2\.0\.0/, /1\.1\.0/, /1\.0\.0/]);

  await picker.click();
  await page.getByRole('option', { name: '1.0.0' }).click();
  await expect(page).toHaveURL(/version=1\.0\.0/);
  await page.getByRole('tab', { name: /Files/ }).click();
  const files = page.getByRole('table', { name: 'Files of this version' });
  await expect(files.getByRole('rowheader')).toHaveText([
    '/org/acme/demo-lib/1.0.0/demo-lib-1.0.0.jar',
    '/org/acme/demo-lib/1.0.0/demo-lib-1.0.0.pom',
  ]);

  await page.getByRole('tab', { name: 'Usage' }).click();
  await expect(page.getByText('<version>1.0.0</version>')).toBeVisible();
  await page.getByRole('tab', { name: /Gradle Kotlin/ }).click();
  await expect(page.getByText('implementation("org.acme:demo-lib:1.0.0")')).toBeVisible();
  await expect(page).toHaveURL(/tab=usage/);

  await page.goto(`/search/component/${encodeURIComponent('docker::team/app')}?tab=usage`);
  await expect(
    page.getByText(`docker pull ${new URL(server.url).host}/${SEED.docker}/team/app:1.0`),
  ).toBeVisible();

  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto(`${PAGE}?tab=files`);
  await expect(page.getByRole('heading', { level: 1, name: 'org.acme:demo-lib' })).toBeVisible();
  await noHorizontalScroll(page);
});
