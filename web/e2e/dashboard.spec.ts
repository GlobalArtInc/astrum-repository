import { adminFetch, REPOSITORY_COUNT, SEED, seedBrowse } from './browse-seed';
import { expect, noHorizontalScroll, signIn, test } from './fixtures';

interface Usage {
  totals: { components: number; assets: number; downloadsInPeriod: number };
}

test('the overview shows how the repositories were used', async ({ page, server }) => {
  await seedBrowse(server.url, server.adminPassword);
  const call = adminFetch(server.url, server.adminPassword);
  const usage = (await (await call('/service/rest/internal/ui/usage-dashboard?days=30')).json()) as Usage;
  expect(usage.totals.downloadsInPeriod).toBe(4);
  const number = (value: number) => new Intl.NumberFormat('en').format(value);

  await page.goto('/login');
  await signIn(page, server.adminPassword);
  await expect(page.getByRole('heading', { level: 1, name: 'Overview' })).toBeVisible();
  await expect(page.getByText('How the repositories were used over the last 30 days')).toBeVisible();
  await expect(page.getByText('today 4 · avg 0 a day')).toBeVisible();
  await expect(page.getByText(`in ${REPOSITORY_COUNT} repositories`)).toBeVisible();
  await expect(page.getByText('hosted 6 · proxy 1 · group 1')).toBeVisible();
  await expect(page.getByText(number(usage.totals.components), { exact: true })).toBeVisible();
  await expect(page.getByText(number(usage.totals.assets), { exact: true })).toBeVisible();
  await expect(page.getByText(/^Last 30 days · 4 in total · peak 4 \(/)).toBeVisible();
  await expect(page.getByRole('img', { name: /Bars of the downloads/ })).toBeVisible();

  const top = page.getByRole('region', { name: 'Most downloaded' });
  await expect(top.getByRole('row')).toHaveCount(3);
  await top.getByRole('link', { name: 'docs/readme.txt' }).click();
  await expect(page).toHaveURL(new RegExp(`/browse/${SEED.raw}/docs/readme\\.txt$`));
  await page.goBack();

  const repositories = page.getByRole('region', { name: 'Repositories' });
  await expect(repositories.getByRole('link')).toHaveText([SEED.raw, SEED.maven]);
  await repositories.getByRole('radio', { name: 'By size' }).click();
  await expect(repositories.getByRole('link').first()).toBeVisible();

  const recent = page.getByRole('region', { name: 'Recently published' });
  await expect(recent.getByRole('link', { name: 'team/app' })).toBeVisible();
  await expect(recent.getByText(`${SEED.docker} · admin`)).toBeVisible();
  const unused = page.getByRole('region', { name: 'Not downloaded for 90 days' });
  await expect(unused.getByRole('link', { name: 'Configure cleanup policies' })).toHaveAttribute(
    'href',
    '/admin/repository/cleanup-policies',
  );

  await page.getByRole('radio', { name: '7 days' }).click();
  await expect(page.getByText('How the repositories were used over the last 7 days')).toBeVisible();
  await expect(page.getByText('today 4 · avg 1 a day')).toBeVisible();
  await expect(top.getByRole('radio', { name: 'Last 7 days' })).toBeVisible();

  await page.getByRole('combobox', { name: 'Repository' }).click();
  await page.getByRole('option', { name: SEED.maven }).click();
  await expect(page.getByText(`How ${SEED.maven} was used over the last 7 days`)).toBeVisible();
  await expect(page.getByText('in 1 repository')).toBeVisible();
  await expect(top.getByRole('link')).toHaveText(['org.acme:demo-lib', 'All components in search']);

  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/');
  await expect(page.getByRole('region', { name: 'Downloads per day' })).toBeVisible();
  await noHorizontalScroll(page);
});
