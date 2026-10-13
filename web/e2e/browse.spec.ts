import type { Page } from '@playwright/test';
import { adminFetch, deployMaven, put, SEED, seedBrowse, VIEWER } from './browse-seed';
import { expect, noHorizontalScroll, signIn, test } from './fixtures';

async function enter(page: Page, password: string, user?: string) {
  await signIn(page, password, user);
  await expect(page).not.toHaveURL(/\/login/);
  await expect(page.getByRole('button', { name: /Account menu of/ })).toBeVisible();
}

test.beforeEach(async ({ page, server }) => {
  await seedBrowse(server.url, server.adminPassword);
  await page.goto('/login');
});

test('the repository list filters by format, type, status and name', async ({ page, server }) => {
  await enter(page, server.adminPassword);
  await page.getByRole('navigation', { name: 'Main menu' }).getByRole('link', { name: 'Browse' }).click();
  await expect(page).toHaveURL(/\/browse$/);
  const table = page.getByRole('table', { name: 'Repositories' });
  await expect(table.getByRole('rowheader')).toHaveCount(8);
  const filters = page.getByRole('complementary', { name: 'Filters' });
  await filters.getByRole('checkbox', { name: /Raw/ }).click();
  await expect(table.getByRole('rowheader')).toHaveCount(4);
  await filters.getByRole('checkbox', { name: /Proxy/ }).click();
  await expect(table.getByRole('rowheader')).toHaveText([SEED.rawProxy]);
  await expect(page).toHaveURL(/\?format=raw&type=proxy$/);
  await filters.getByRole('button', { name: 'Reset filters' }).click();
  await page.getByRole('searchbox', { name: 'Filter repositories by name' }).fill('maven');
  await expect(table.getByRole('rowheader')).toHaveText([SEED.maven]);
  await expect(page.getByText(`total ${8}`)).toBeVisible();
  await expect(table.getByText(`/repository/${SEED.maven}/`)).toBeVisible();
  await table.getByRole('button', { name: `Actions for ${SEED.maven}` }).click();
  await expect(page.getByRole('menuitem', { name: 'Repository Settings' })).toBeVisible();
  await page.keyboard.press('Escape');
  await expect(page.getByRole('navigation', { name: 'Pages' })).toBeVisible();
  await table.getByRole('link', { name: SEED.maven, exact: true }).click();
  await expect(page).toHaveURL(new RegExp(`/browse/${SEED.maven}$`));
  await expect(page.getByRole('heading', { level: 1, name: SEED.maven })).toBeVisible();
  await page.goto(`/browse/${SEED.rawProxy}`);
  await expect(page.getByText('A proxy keeps what clients have asked for through it')).toBeVisible();
  await expect(page.getByText('http://127.0.0.1:9/')).toBeVisible();
});

test('the tree navigates with the keyboard and opens deep links', async ({ page, server }) => {
  await enter(page, server.adminPassword);
  await page.goto(`/browse/${SEED.maven}/org/acme/demo-lib/2.0.0/demo-lib-2.0.0.jar`);
  const tree = page.getByRole('tree', { name: `Content of ${SEED.maven}` });
  const jar = tree.getByRole('treeitem', { name: /demo-lib-2\.0\.0\.jar/ });
  await expect(jar).toHaveAttribute('aria-selected', 'true');
  await expect(tree.getByRole('treeitem', { name: /^demo-lib/ }).first()).toHaveAttribute(
    'aria-expanded',
    'true',
  );
  await expect(page.getByRole('heading', { level: 2, name: 'demo-lib-2.0.0.jar' })).toBeVisible();

  await page.goto(`/browse/${SEED.raw}`);
  await expect(page.getByRole('heading', { name: 'Nothing selected' })).toBeVisible();
  const files = page.getByRole('tree', { name: `Content of ${SEED.raw}` });
  const docs = files.getByRole('treeitem', { name: /^docs/ });
  await docs.focus();
  await page.keyboard.press('ArrowRight');
  await expect(docs).toHaveAttribute('aria-expanded', 'true');
  await page.keyboard.press('ArrowDown');
  await expect(files.getByRole('treeitem', { name: /^guide/ })).toBeFocused();
  await page.keyboard.press('ArrowDown');
  await expect(files.getByRole('treeitem', { name: /readme\.txt/ })).toBeFocused();
  await page.keyboard.press('Enter');
  await expect(page).toHaveURL(new RegExp(`/browse/${SEED.raw}/docs/readme\\.txt$`));
  await expect(page.getByRole('heading', { level: 2, name: 'readme.txt' })).toBeVisible();
  await page.keyboard.press('ArrowLeft');
  await expect(docs).toBeFocused();
  await page.keyboard.press('ArrowLeft');
  await expect(docs).toHaveAttribute('aria-expanded', 'false');

  const separator = page.getByRole('separator', { name: /Width of the tree/ });
  const before = Number(await separator.getAttribute('aria-valuenow'));
  await separator.focus();
  await page.keyboard.press('ArrowRight');
  await expect(separator).toHaveAttribute('aria-valuenow', String(before + 16));

  await page.goto(`/browse/${SEED.maven}`);
  await page.getByRole('searchbox', { name: `Search in ${SEED.maven}` }).fill('demo-lib');
  const results = page.getByRole('list', { name: 'Search results' });
  await expect(results.getByRole('button')).toHaveCount(3);
  await results.getByRole('button', { name: /1\.1\.0/ }).click();
  await expect(page).toHaveURL(new RegExp(`/browse/${SEED.maven}/org/acme/demo-lib/1\\.1\\.0/$`));
  await expect(page.getByRole('heading', { level: 2, name: 'demo-lib:1.1.0' })).toBeVisible();
});

test('a file shows its details and how to fetch it', async ({ page, server }) => {
  await enter(page, server.adminPassword);
  await page.goto(`/browse/${SEED.npm}`);
  const tree = page.getByRole('tree');
  await tree.getByRole('treeitem', { name: /^@team/ }).click();
  await tree.getByRole('treeitem', { name: /^tool/ }).first().click();
  await tree.getByRole('treeitem', { name: /^-/ }).click();
  await tree.getByRole('treeitem', { name: /tool-1\.0\.0\.tgz/ }).click();
  await expect(page.getByRole('heading', { level: 2, name: 'tool-1.0.0.tgz' })).toBeVisible();
  const details = page.getByRole('region', { name: 'Details' });
  await expect(details.getByText('application/x-tgz')).toBeVisible();
  await expect(details.getByText('Never')).toBeVisible();
  await expect(details.getByText(/^[0-9a-f]{40}$/)).toBeVisible();
  await expect(details.getByText(/npm install @team\/tool@1\.0\.0 --registry=/)).toBeVisible();
  await expect(details.getByRole('link', { name: 'Download' })).toHaveAttribute(
    'href',
    `${server.url}/repository/${SEED.npm}/%40team/tool/-/tool-1.0.0.tgz`,
  );
  await details.getByRole('link', { name: 'Component page' }).click();
  await expect(page).toHaveURL(/\/search\/component\/npm%3Ateam%3Atool\?version=1\.0\.0$/);
});

test('folders, components and files are deleted after a confirmation', async ({ page, server }) => {
  const call = adminFetch(server.url, server.adminPassword);
  await put(call, `/repository/${SEED.raw}/trash/one.txt`, 'one', 'text/plain');
  await put(call, `/repository/${SEED.raw}/trash/deeper/two.txt`, 'two', 'text/plain');
  await put(call, `/repository/${SEED.raw}/single.txt`, 'single', 'text/plain');
  const version = `0.1.${Date.now()}`;
  await deployMaven(call, SEED.maven, 'com.example', 'scratch', version);
  await enter(page, server.adminPassword);

  await page.goto(`/browse/${SEED.raw}/trash/`);
  await page.getByRole('button', { name: 'Delete folder' }).click();
  const folderDialog = page.getByRole('alertdialog', { name: 'Delete the folder trash?' });
  await expect(folderDialog).toContainText('including the 2 items directly inside it');
  await folderDialog.getByRole('button', { name: 'Cancel' }).click();
  await expect(folderDialog).toHaveCount(0);
  await page.getByRole('button', { name: 'Delete folder' }).click();
  await folderDialog.getByRole('button', { name: 'Delete' }).click();
  await expect(page.getByText('The folder trash is deleted', { exact: true })).toBeVisible();
  await expect(page.getByText('2 files deleted · 2 components deleted', { exact: true })).toBeVisible();
  await expect(page).toHaveURL(new RegExp(`/browse/${SEED.raw}$`));
  await expect(page.getByRole('tree').getByRole('treeitem', { name: /^trash/ })).toHaveCount(0);

  await page.goto(`/browse/${SEED.raw}/single.txt`);
  await page.getByRole('button', { name: 'Delete file' }).click();
  await page
    .getByRole('alertdialog', { name: 'Delete the file?' })
    .getByRole('button', { name: 'Delete' })
    .click();
  await expect(page.getByText('single.txt is deleted', { exact: true })).toBeVisible();
  await expect(page.getByRole('tree').getByRole('treeitem', { name: /single\.txt/ })).toHaveCount(0);

  await page.goto(`/browse/${SEED.maven}/com/example/scratch/${version}/`);
  await expect(page.getByRole('heading', { level: 2, name: `scratch:${version}` })).toBeVisible();
  await page.getByRole('button', { name: 'Delete component' }).click();
  const componentDialog = page.getByRole('alertdialog', { name: 'Delete the component?' });
  await expect(componentDialog).toContainText(`com.example:scratch:${version}`);
  await componentDialog.getByRole('button', { name: 'Delete' }).click();
  await expect(page.getByText(`com.example:scratch:${version} is deleted`, { exact: true })).toBeVisible();
  await expect(page).toHaveURL(new RegExp(`/browse/${SEED.maven}/com/example/scratch/$`));
  const gone = await call(`/repository/${SEED.maven}/com/example/scratch/${version}/scratch-${version}.jar`);
  expect(gone.status).toBe(404);
});

test('someone who only sees a group browses its members through it', async ({ page }) => {
  await enter(page, VIEWER.password, VIEWER.user);
  await page.getByRole('navigation', { name: 'Main menu' }).getByRole('link', { name: 'Browse' }).click();
  const table = page.getByRole('table', { name: 'Repositories' });
  await expect(table.getByRole('rowheader')).toHaveText([SEED.rawGroup]);
  await table.getByRole('link', { name: SEED.rawGroup }).click();
  const tree = page.getByRole('tree');
  await expect(tree.getByRole('treeitem', { level: 1 })).toHaveText([
    /docs/,
    /shared/,
    /tools/,
    /hello\.txt/,
  ]);
  await tree.getByRole('treeitem', { name: /^docs/ }).click();
  await tree.getByRole('treeitem', { name: /readme\.txt/ }).click();
  await expect(page.getByRole('heading', { level: 2, name: 'readme.txt' })).toBeVisible();
  const details = page.getByRole('region', { name: 'Details' });
  await expect(details.getByText(`${SEED.raw} (through ${SEED.rawGroup})`)).toBeVisible();
  await expect(details.getByText(/^15\sB$/)).toBeVisible();
  await expect(details.getByRole('link', { name: 'Download' })).toHaveAttribute(
    'href',
    new RegExp(`/repository/${SEED.rawGroup}/docs/readme\\.txt$`),
  );
  await expect(details.getByRole('button', { name: 'Delete file' })).toHaveCount(0);
  await tree.getByRole('treeitem', { name: /^shared/ }).click();
  await expect(page.getByRole('button', { name: 'Delete folder' })).toHaveCount(0);
  await page.goto(`/browse/${SEED.raw}`);
  await expect(page.getByRole('heading', { level: 1, name: 'Repository not found' })).toBeVisible();
});

test('a phone shows the tree and the details as two steps', async ({ page, server }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await enter(page, server.adminPassword);
  await page.goto('/browse');
  await expect(page.getByRole('list', { name: 'Repositories' }).getByRole('listitem')).toHaveCount(8);
  await expect(page.getByRole('table')).toHaveCount(0);
  await noHorizontalScroll(page);
  await page.getByRole('link', { name: SEED.raw, exact: true }).click();
  const tree = page.getByRole('tree');
  await tree.getByRole('treeitem', { name: /^docs/ }).click();
  await noHorizontalScroll(page);
  await tree.getByRole('treeitem', { name: /readme\.txt/ }).click();
  await expect(page.getByRole('heading', { level: 2, name: 'readme.txt' })).toBeVisible();
  await expect(page.getByRole('tree')).toHaveCount(0);
  await noHorizontalScroll(page);
  await page.getByRole('button', { name: 'Back to the tree' }).click();
  await tree.getByRole('button', { name: 'Details of docs' }).click();
  await expect(page.getByRole('heading', { level: 2, name: 'docs' })).toBeVisible();
  await noHorizontalScroll(page);
  await page.getByRole('button', { name: 'Usage' }).click();
  const box = await page.getByRole('dialog', { name: `Connect to ${SEED.raw}` }).boundingBox();
  expect((box?.x ?? 0) + (box?.width ?? 1000)).toBeLessThanOrEqual(390);
});
