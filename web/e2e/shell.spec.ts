import { expect, noHorizontalScroll, signIn, test } from './fixtures';

test.beforeEach(async ({ page, server }) => {
  await page.goto('/login');
  await signIn(page, server.adminPassword);
  await expect(page.getByRole('heading', { level: 1, name: 'Overview' })).toBeVisible();
});

test('the server sends the UI with its security headers', async ({ request, server }) => {
  const page = await request.get(`${server.url}/browse/some/where`);
  expect(page.status()).toBe(200);
  expect(page.headers()['content-security-policy']).toContain("default-src 'self'");
  expect(page.headers()['content-security-policy']).toContain("frame-ancestors 'none'");
  expect(page.headers()['x-content-type-options']).toBe('nosniff');
  expect(page.headers()['referrer-policy']).toBe('same-origin');
  expect(await page.text()).toContain('<div id="root">');
  const api = await request.get(`${server.url}/service/rest/v1/nothing`);
  expect(api.status()).toBe(404);
});

test('theme and language change and stay after a reload', async ({ page }) => {
  await expect(page.locator('html')).toHaveAttribute('data-theme', 'dark');
  await page.getByRole('button', { name: 'Theme' }).click();
  await page.getByRole('menuitemradio', { name: 'Light' }).click();
  await expect(page.locator('html')).toHaveAttribute('data-theme', 'light');

  await page.getByRole('button', { name: 'Language' }).click();
  await page.getByRole('menuitemradio', { name: 'Русский' }).click();
  const menu = page.getByRole('navigation', { name: 'Основное меню' });
  await expect(menu.getByRole('link', { name: 'Обзор' })).toBeVisible();
  await expect(page.locator('html')).toHaveAttribute('lang', 'ru');

  await page.reload();
  await expect(page.locator('html')).toHaveAttribute('data-theme', 'light');
  await expect(page.getByRole('heading', { level: 1, name: 'Обзор' })).toBeVisible();
  const cookies = await page.context().cookies();
  expect(cookies.find((cookie) => cookie.name === 'astrum-language')?.value).toBe('ru');
});

test('the menu collapses and remembers it', async ({ page }) => {
  const menu = page.getByRole('navigation', { name: 'Main menu' });
  await expect(menu.getByText('Dashboard')).toBeVisible();
  await page.getByRole('button', { name: 'Collapse the menu' }).click();
  await expect(menu.getByText('Dashboard')).toHaveCount(0);
  await page.reload();
  await expect(page.getByRole('button', { name: 'Expand the menu' })).toBeVisible();
  await expect(menu.getByRole('link', { name: 'Dashboard' })).toBeVisible();
});

test('the search panel opens with the keyboard', async ({ page }) => {
  await page.keyboard.press('Control+k');
  const panel = page.getByRole('dialog', { name: 'Search' });
  await expect(panel).toBeVisible();
  await expect(panel.getByRole('combobox', { name: 'Search' })).toBeFocused();
  await page.keyboard.press('Escape');
  await expect(panel).toHaveCount(0);
});

test('a phone gets a drawer that keeps the focus', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await noHorizontalScroll(page);
  await page.getByRole('button', { name: 'Open the menu' }).click();
  const drawer = page.getByRole('dialog', { name: 'Main menu' });
  await expect(drawer).toBeVisible();
  for (let step = 0; step < 8; step += 1) {
    await page.keyboard.press('Tab');
    const inside = await drawer.evaluate((node) => node.contains(document.activeElement));
    expect(inside).toBe(true);
  }
  await page.keyboard.press('Escape');
  if ((await drawer.count()) > 0) {
    await page.keyboard.press('Escape');
  }
  await expect(drawer).toHaveCount(0);
  await page.getByRole('button', { name: 'Open the menu' }).click();
  await drawer.getByRole('link', { name: 'Dashboard' }).click();
  await expect(drawer).toHaveCount(0);
});

test('old hash addresses lead to their new paths', async ({ page }) => {
  await page.goto('/#admin');
  await expect(page).toHaveURL(/\/settings$/);
  await page.goto('/#browse/browse:maven-releases:org%2Fexample');
  await expect(page).toHaveURL(/\/browse\/maven-releases\/org\/example$/);
  await page.goto('/#browse/welcome');
  await expect(page).toHaveURL(/\/$/);
  await expect(page.getByRole('heading', { level: 1, name: 'Overview' })).toBeVisible();
  const returnTo = Buffer.from('#browse/browse').toString('base64');
  await page.goto(`/#/login?returnTo=${returnTo}`);
  await expect(page).toHaveURL(/\/browse$/);
});

test('a missing page keeps its address', async ({ page }) => {
  await page.goto('/no/such/page?x=1');
  await expect(page.getByRole('heading', { level: 1, name: 'Page not found' })).toBeVisible();
  await expect(page.getByText('/no/such/page?x=1')).toBeVisible();
  await expect(page).toHaveURL(/\/no\/such\/page\?x=1$/);
  await expect(page).toHaveTitle('Page not found - Astrum Repository');
  await page.getByRole('link', { name: 'Go to the dashboard' }).click();
  await expect(page).toHaveURL(/\/$/);
});
