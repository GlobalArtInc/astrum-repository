import { call, SEARCH_REPOSITORIES, seedSearch, TOOLS } from './content';
import { expect, noHorizontalScroll, signIn, test } from './fixtures';

test.beforeEach(async ({ page, server }) => {
  await seedSearch(server);
  await page.goto('/login');
  await signIn(page, server.adminPassword);
  await expect(page.getByRole('heading', { level: 1, name: 'Overview' })).toBeVisible();
});

function firstRowName(page: import('@playwright/test').Page) {
  return page.getByRole('table').getByRole('rowheader').first();
}

test('search finds components as you type and loads more', async ({ page }) => {
  await page.getByRole('navigation', { name: 'Main menu' }).getByRole('link', { name: 'Search' }).click();
  await expect(page.getByRole('heading', { level: 1, name: 'Search' })).toBeVisible();
  await expect(page.getByText('Search all repositories')).toBeVisible();
  await page.getByLabel('Name, group or version').fill('tool');
  await expect(page).toHaveURL(/\/search\?q=tool$/);
  await expect(page.getByText('10 shown, more can be loaded')).toBeVisible();
  await page.getByRole('button', { name: 'Show more' }).click();
  await expect(page.getByText(`${TOOLS + 2} components found`)).toBeVisible();
  await expect(page.getByRole('link', { name: /\/tools\/1\/tool-1\.bin/ })).toBeVisible();
  const component = page.getByRole('link', { name: 'astrum-tools' }).first();
  await expect(component).toHaveAttribute(
    'href',
    /^\/search\/component\/pypi%3A%3Aastrum-tools\?version=1\.1\.0$/,
  );
  await page.reload();
  await expect(page.getByLabel('Name, group or version')).toHaveValue('tool');
  await page.getByLabel('Name, group or version').fill('tool-1*.bin');
  await expect(page).toHaveURL(/q=tool-1\*\.bin/);
  await expect(page.getByText('4 components found')).toBeVisible();
  await page.getByLabel('Name, group or version').fill('no-such-thing');
  await expect(page.getByText('Nothing found')).toBeVisible();
});

test('search filters by format and repository and sorts on the server', async ({ page }) => {
  await page.goto('/search?format=maven2');
  await expect(page.getByText('4 components found')).toBeVisible();
  await expect(firstRowName(page)).toContainText('ledger');
  await page.getByRole('combobox', { name: 'Sort' }).click();
  await page.getByRole('option', { name: 'Name, A to Z' }).click();
  await expect(page).toHaveURL(/format=maven2&sort=name$/);
  await expect(firstRowName(page)).toContainText('astrum-client');
  await page.getByRole('combobox', { name: 'Sort' }).click();
  await page.getByRole('option', { name: 'Version, descending' }).click();
  await expect(page).toHaveURL(/sort=version&direction=desc$/);
  await expect(firstRowName(page)).toContainText('ledger');
  await expect(page.getByRole('table').getByRole('row').nth(2)).toContainText('1.4.0');
  await page.getByRole('combobox', { name: 'Repository' }).click();
  await expect(page.getByRole('option', { name: SEARCH_REPOSITORIES.npm })).toHaveCount(0);
  await page.getByRole('option', { name: SEARCH_REPOSITORIES.maven }).click();
  await expect(page).toHaveURL(new RegExp(`repository=${SEARCH_REPOSITORIES.maven}`));
  await page.getByRole('combobox', { name: 'Format' }).click();
  await page.getByRole('option', { name: 'npm' }).click();
  await expect(page).toHaveURL(/\?format=npm&sort=version&direction=desc$/);
  await expect(page.getByText('4 components found')).toBeVisible();
  await page.getByRole('combobox', { name: 'Sort' }).click();
  await page.getByRole('option', { name: 'Oldest first' }).click();
  await expect(firstRowName(page)).toContainText('ui-kit');
  await expect(page.getByRole('table').getByRole('row').nth(1)).toContainText('4.2.0');
  await expect(page.getByRole('table').getByRole('row').nth(1)).toContainText('globalart');
});

test('criteria search uses format keys and finds files with download links', async ({ page, server }) => {
  await page.goto('/search');
  await page.getByRole('link', { name: 'Search by criteria' }).click();
  await expect(page).toHaveURL(/\/search\/custom$/);
  await page.getByRole('combobox', { name: 'Criterion' }).click();
  await page.getByRole('option', { name: 'Maven group ID' }).click();
  await page.getByRole('combobox', { name: 'Condition' }).click();
  await page.getByRole('option', { name: 'starts with' }).click();
  await page.getByLabel('Value of Maven group ID').fill('com.glob');
  await page.getByRole('button', { name: 'Add a criterion' }).click();
  await page.getByRole('combobox', { name: 'Criterion' }).nth(1).click();
  await page.getByRole('option', { name: 'Maven extension' }).click();
  await page.getByLabel('Value of Maven extension').fill('pom');
  await page.getByRole('button', { name: 'Search', exact: true }).click();
  await expect(page).toHaveURL(/\/search\/custom\?maven\.groupId=com\.glob\*&maven\.extension=pom$/);
  await expect(page.getByRole('table').getByRole('rowheader')).toHaveCount(3);
  await expect(firstRowName(page)).toContainText('astrum-client');

  await page.getByRole('radio', { name: 'Files' }).click();
  await page.getByRole('button', { name: 'Search', exact: true }).click();
  await expect(page).toHaveURL(/mode=assets$/);
  const links = page.getByRole('table').getByRole('link', { name: /\.pom$/ });
  await expect(links).toHaveCount(3);
  const href = await links.first().getAttribute('href');
  expect(href).toMatch(
    /\/repository\/search-maven\/com\/globalart\/astrum-client\/1\.\d\.0\/astrum-client-1\.\d\.0\.pom$/,
  );
  const download = await page.request.get(href ?? '');
  expect(download.status()).toBe(200);
  expect(await download.text()).toContain('<artifactId>astrum-client</artifactId>');
  await expect(page.getByRole('table').getByRole('link', { name: 'search-maven' }).first()).toHaveAttribute(
    'href',
    /^\/browse\/search-maven\/com\/globalart\/astrum-client\/1\.\d\.0$/,
  );

  await page.goto('/search/custom?npm.scope=globalart&version=*.1*');
  await expect(page.getByLabel('Value of npm scope')).toHaveValue('globalart');
  await expect(page.getByRole('table').getByRole('rowheader')).toHaveCount(2);
  await page.goto('/search/custom?pypi.summary=*reports*');
  await expect(page.getByRole('combobox', { name: 'Condition' })).toHaveText('contains');
  await expect(page.getByRole('table').getByRole('rowheader')).toHaveCount(1);
  await expect(firstRowName(page)).toContainText('report-builder');

  const reply = await call(
    server,
    'GET',
    '/service/rest/v1/search?repository=search-raw&name=/tools/3/tool-3.bin',
  );
  const sha1 = (await reply.json()).items[0].assets[0].checksum.sha1 as string;
  await page.goto(`/search/custom?sha1=${sha1}&mode=assets`);
  await expect(page.getByRole('link', { name: '/tools/3/tool-3.bin' })).toBeVisible();
});

test('the header search panel works with the keyboard', async ({ page }) => {
  await page.keyboard.press('Control+k');
  const panel = page.getByRole('dialog', { name: 'Search' });
  const input = panel.getByRole('combobox', { name: 'Search' });
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
  await expect(page.getByText('5 components found')).toBeVisible();

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
  await expect(panel.getByRole('option', { name: 'this repository, search-npm' })).toBeVisible();
  await page.keyboard.press('ArrowDown');
  await page.keyboard.press('ArrowDown');
  await page.keyboard.press('Enter');
  await expect(page).toHaveURL(/\/search\?q=kit&repository=search-npm$/);
  await expect(page.getByText('2 components found')).toBeVisible();

  await page.keyboard.press('Control+k');
  await input.fill('ledger');
  await panel.getByRole('option', { name: /ledger/ }).click();
  await expect(page).toHaveURL(/\/search\/component\/maven2%3Aorg\.acme%3Aledger\?version=1\.8\.0$/);
});

test('search pages fit a phone', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/search?q=tool');
  await expect(page.getByText('10 shown, more can be loaded')).toBeVisible();
  await noHorizontalScroll(page);
  await page.goto('/search/custom?maven.groupId=com.*&mode=assets');
  await expect(page.getByRole('link', { name: /\.jar$/ }).first()).toBeVisible();
  await noHorizontalScroll(page);
  await page.getByRole('banner').getByRole('button', { name: 'Search', exact: true }).click();
  const panel = page.getByRole('dialog', { name: 'Search' });
  await expect(panel).toBeVisible();
  const box = await panel.boundingBox();
  expect(box?.width).toBe(390);
  expect(box?.height).toBe(844);
  await panel.getByRole('combobox', { name: 'Search' }).fill('left');
  await expect(panel.getByRole('option', { name: /left-pad/ })).toBeVisible();
  await noHorizontalScroll(page);
});
