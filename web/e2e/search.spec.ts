import { SEARCH_REPOSITORIES, seedSearch, TOOLS } from './content';
import { expect, noHorizontalScroll, signIn, test } from './fixtures';

test.beforeEach(async ({ page, server }) => {
  await seedSearch(server);
  await page.goto('/login');
  await signIn(page, server.adminPassword);
  await expect(page.getByRole('heading', { level: 1, name: 'Overview' })).toBeVisible();
});

function results(page: import('@playwright/test').Page) {
  return page
    .getByRole('region', { name: 'Search results' })
    .getByRole('button', { name: /^Open .* in its repository$/ });
}

test('search finds components as you type and opens them', async ({ page }) => {
  await page.getByRole('navigation', { name: 'Main menu' }).getByRole('link', { name: 'Search' }).click();
  await expect(page.getByRole('heading', { level: 1, name: 'Search' })).toBeVisible();
  await expect(page.getByText('Search all repositories')).toBeVisible();
  const query = page.getByRole('textbox', { name: 'Search components' });
  await expect(query).toBeFocused();
  await query.fill('tool');
  await expect(page).toHaveURL(/\/search\?q=tool$/);
  await expect(page.getByText(`Shown: ${TOOLS + 2}`)).toBeVisible();
  await expect(results(page)).toHaveCount(TOOLS + 2);
  await expect(page.getByRole('button', { name: 'Show More' })).toHaveCount(0);
  await expect(
    page.getByRole('button', { name: 'Open /tools/1/tool-1.bin in its repository' }),
  ).toBeVisible();
  await page.getByRole('button', { name: 'Versions of astrum-tools in all repositories' }).first().click();
  await expect(page).toHaveURL(/\/search\/component\/pypi%3A%3Aastrum-tools\?version=1\.1\.0$/);
  await page.goBack();
  await expect(query).toHaveValue('tool');
  await page.getByRole('button', { name: 'Open /tools/1/tool-1.bin in its repository' }).click();
  await expect(page).toHaveURL(/\/browse\/search-raw\/tools\/1\/tool-1\.bin$/);
  await page.goBack();
  await query.fill('tool-1*.bin');
  await expect(page).toHaveURL(/q=tool-1\*\.bin/);
  await expect(page.getByText('Shown: 4')).toBeVisible();
  await query.fill('no-such-thing');
  await expect(page.getByText('Nothing found')).toBeVisible();
});

test('search filters by format and repository and sorts on the server', async ({ page }) => {
  await page.goto('/search?format=maven2');
  await expect(page.getByText('Shown: 4')).toBeVisible();
  await expect(results(page).first()).toContainText('ledger');
  await page.getByRole('combobox', { name: 'Sort' }).click();
  await expect(page.getByRole('option')).toHaveText([
    'Newest first',
    'Oldest first',
    'Name, A to Z',
    'Name, Z to A',
    'Repository, A to Z',
    'Repository, Z to A',
  ]);
  await page.getByRole('option', { name: 'Name, A to Z' }).click();
  await expect(page).toHaveURL(/format=maven2&sort=name$/);
  await expect(results(page).first()).toContainText('astrum-client');
  await page.getByRole('combobox', { name: 'Sort' }).click();
  await page.getByRole('option', { name: 'Name, Z to A' }).click();
  await expect(page).toHaveURL(/sort=name&direction=desc$/);
  await expect(results(page).first()).toContainText('ledger');
  await page.getByRole('combobox', { name: 'Repository' }).click();
  await expect(page.getByRole('option', { name: SEARCH_REPOSITORIES.npm })).toHaveCount(0);
  await page.getByRole('option', { name: SEARCH_REPOSITORIES.maven }).click();
  await expect(page).toHaveURL(new RegExp(`repository=${SEARCH_REPOSITORIES.maven}`));
  await page.getByRole('combobox', { name: 'Format' }).click();
  await page.getByRole('option', { name: 'npm' }).click();
  await expect(page).toHaveURL(/\?format=npm&sort=name&direction=desc$/);
  await expect(page.getByText('Shown: 4')).toBeVisible();
  await page.getByRole('combobox', { name: 'Sort' }).click();
  await page.getByRole('option', { name: 'Oldest first' }).click();
  await expect(results(page).first()).toContainText('@globalart/ui-kit');
  await expect(results(page).first()).toContainText('4.2.0');
});

test('criteria search combines conditions and opens components', async ({ page }) => {
  await page.goto('/search/custom');
  await expect(page.getByRole('heading', { level: 1, name: 'Advanced Search' })).toBeVisible();
  await expect(page.getByText('Build a query')).toBeVisible();
  await page.getByRole('combobox', { name: 'Field' }).click();
  await expect(page.getByRole('option').first()).toHaveText('Keyword');
  await expect(page.getByRole('option').nth(5)).toHaveText('Version');
  await expect(page.getByRole('option', { name: 'SHA-1 Checksum' })).toBeVisible();
  await expect(page.getByRole('option', { name: 'Maven Artifact ID' })).toBeAttached();
  await page.getByRole('option', { name: 'Group', exact: true }).click();
  await page.getByRole('combobox', { name: 'Condition' }).click();
  await page.getByRole('option', { name: 'Starts with' }).click();
  await page.getByRole('textbox', { name: 'Value' }).fill('com.glob');
  await page.getByRole('button', { name: 'Add Condition' }).click();
  await page.getByRole('combobox', { name: 'Field' }).nth(1).click();
  await page.getByRole('option', { name: 'Version', exact: true }).click();
  await page.getByRole('textbox', { name: 'Value' }).nth(1).fill('1.3.0');
  await page.getByRole('button', { name: 'Search', exact: true }).click();
  await expect(page).toHaveURL(/\/search\/custom\?group=com\.glob\*&version=1\.3\.0$/);
  await expect(page.getByText('1 component found')).toBeVisible();
  const row = page.getByRole('button', { name: 'Open astrum-client in the repository' });
  await expect(row).toContainText('com.globalart');
  await row.click();
  await expect(page).toHaveURL(
    /\/browse\/search-maven\/com\/globalart\/astrum-client\/1\.3\.0\/astrum-client-1\.3\.0\.jar$/,
  );

  await page.goto('/search/custom?name=report*');
  await expect(page.getByRole('combobox', { name: 'Condition' })).toHaveText('Starts with');
  await expect(page.getByRole('button', { name: 'Open report-builder in the repository' })).toBeVisible();
  await page.getByRole('button', { name: 'Clear All' }).click();
  await expect(page).toHaveURL(/\/search\/custom$/);
  await page.getByRole('button', { name: 'Search', exact: true }).click();
  await expect(page.getByText(/^\d+ components found$/)).toBeVisible();
});

test('the header search panel works with the keyboard', async ({ page }) => {
  await page.keyboard.press('Control+k');
  const panel = page.getByRole('listbox', { name: 'Search suggestions' });
  const input = page.getByRole('combobox', { name: 'Search components' });
  await expect(input).toBeFocused();
  await input.fill('astrum');
  await expect(panel.getByRole('group', { name: 'Components' }).getByRole('option')).toHaveCount(5);
  await expect(panel.getByRole('option', { name: /astrum-client.*1\.4\.0/ })).toBeVisible();
  await page.keyboard.press('ArrowDown');
  await expect(panel.getByRole('option', { name: 'all repositories' })).toHaveAttribute(
    'aria-selected',
    'true',
  );
  await page.keyboard.press('Enter');
  await expect(panel).toHaveCount(0);
  await expect(page).toHaveURL(/\/search\?q=astrum$/);
  await expect(page.getByText('Shown: 5')).toBeVisible();

  await page.keyboard.press('Control+k');
  await expect(panel.getByRole('group', { name: 'Recent searches' }).getByRole('option')).toHaveText([
    'astrum',
  ]);
  await input.fill('search-');
  await expect(panel.getByRole('group', { name: 'Repositories' }).getByRole('option')).toHaveCount(5);
  await input.fill('search-np');
  await expect(panel.getByRole('option', { name: /search-npm/ })).toBeVisible();
  await page.keyboard.press('ArrowDown');
  await page.keyboard.press('ArrowDown');
  await expect(panel.getByRole('option', { name: /search-npm/ })).toHaveAttribute('aria-selected', 'true');
  await page.keyboard.press('Enter');
  await expect(page).toHaveURL(/\/browse\/search-npm$/);

  await page.keyboard.press('Control+k');
  await input.fill('kit');
  await expect(panel.getByRole('option', { name: 'this repository' })).toBeVisible();
  await page.keyboard.press('ArrowDown');
  await page.keyboard.press('ArrowDown');
  await page.keyboard.press('Enter');
  await expect(page).toHaveURL(/\/search\?q=kit&repository=search-npm$/);
  await expect(page.getByText('Shown: 2')).toBeVisible();

  await page.keyboard.press('Control+k');
  await input.fill('ledger');
  await panel.getByRole('option', { name: /ledger/ }).click();
  await expect(page).toHaveURL(/\/search\/component\/maven2%3Aorg\.acme%3Aledger\?version=1\.8\.0$/);
});

test('search pages fit a phone', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/search?q=tool');
  await expect(page.getByText(`Shown: ${TOOLS + 2}`)).toBeVisible();
  await expect(results(page)).toHaveCount(TOOLS + 2);
  await noHorizontalScroll(page);
  await page.goto('/search/custom?group=com.*');
  await expect(page.getByRole('button', { name: /^Open astrum-client/ }).first()).toBeVisible();
  await noHorizontalScroll(page);
  await page.getByRole('banner').getByRole('button', { name: 'Search', exact: true }).click();
  const panel = page.getByRole('dialog', { name: 'Search' });
  await expect(panel).toBeVisible();
  const box = await panel.boundingBox();
  expect(box?.width).toBeLessThanOrEqual(390);
  await panel.getByRole('combobox', { name: 'Search components' }).fill('left');
  await expect(panel.getByRole('option', { name: /left-pad/ })).toBeVisible();
  await noHorizontalScroll(page);
});
