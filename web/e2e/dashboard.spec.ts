import { adminFetch, REPOSITORY_COUNT, SEED, seedBrowse } from './browse-seed';
import { expect, noHorizontalScroll, signIn, test } from './fixtures';

interface Numbers {
  components: number;
  assets: number;
  downloads: { total: number };
}

test('the dashboard shows the numbers of the server', async ({ page, server }) => {
  await seedBrowse(server.url, server.adminPassword);
  const call = adminFetch(server.url, server.adminPassword);
  const numbers = (await (await call('/service/rest/internal/ui/dashboard')).json()) as Numbers;
  await page.goto('/login');
  await signIn(page, server.adminPassword);
  await expect(page.getByRole('button', { name: /Account menu of/ })).toBeVisible();
  await expect(page.getByRole('heading', { level: 1, name: 'Overview' })).toBeVisible();
  const tile = (label: string) => page.locator('p', { hasText: new RegExp(`^${label}$`) }).locator('..');
  await expect(tile('Repositories')).toContainText(String(REPOSITORY_COUNT));
  await expect(tile('Repositories')).toContainText('Hosted 6 · Proxy 1 · Group 1');
  await expect(tile('Components')).toContainText(new Intl.NumberFormat('en').format(numbers.components));
  await expect(tile('Files')).toContainText(new Intl.NumberFormat('en').format(numbers.assets));
  await expect(page.getByText('Repositories by format')).toBeVisible();
  await expect(page.getByText('Hosted 2 · Proxy 1 · Group 1')).toBeVisible();
  await expect(page.getByRole('region', { name: 'Blob stores' })).toContainText('default');
  const recent = page.getByRole('region', { name: 'Recently uploaded' });
  await expect(recent.getByRole('link', { name: 'team/app' })).toBeVisible();

  await expect
    .poll(
      async () => {
        await page.reload();
        return page.getByRole('region', { name: 'Most downloaded' }).getByRole('link').allTextContents();
      },
      { timeout: 25_000, intervals: [1_000, 2_000, 3_000] },
    )
    .toContain('/docs/readme.txt');
  await page.getByRole('radio', { name: 'Files' }).click();
  await page
    .getByRole('region', { name: 'Most downloaded' })
    .getByRole('link', { name: '/docs/readme.txt' })
    .click();
  await expect(page).toHaveURL(new RegExp(`/browse/${SEED.raw}/docs/readme\\.txt$`));

  await page.goto('/');
  await page.getByRole('link', { name: 'Browse', exact: true }).last().click();
  await expect(page).toHaveURL(/\/browse$/);

  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/');
  await expect(tile('Repositories')).toBeVisible();
  await noHorizontalScroll(page);
});
