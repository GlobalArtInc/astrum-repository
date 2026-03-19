import type { APIRequestContext, Page } from '@playwright/test';
import { expect, signIn, test } from './fixtures';

test('a stranger signs in and out', async ({ page, server }) => {
  await page.goto('/settings');
  await expect(page).toHaveURL(/\/login\?returnTo=%2Fsettings$/);
  await expect(page.locator('html')).toHaveAttribute('data-theme', 'light');
  await expect(page).toHaveTitle('Sign in - GlobalArt Astrum Repository');
  await expect(page.getByRole('link', { name: 'Continue without signing in' })).toHaveCount(0);

  await signIn(page, 'wrong-password');
  await expect(page.getByRole('alert')).toContainText('The user name or password is wrong');

  await signIn(page, server.adminPassword);
  await expect(page).toHaveURL(/\/settings$/);
  await expect(page.getByRole('heading', { level: 1, name: 'Settings' })).toBeVisible();
  await expect(page.locator('html')).toHaveAttribute('data-theme', 'dark');

  await page.getByRole('button', { name: /Account menu of/ }).click();
  await page.getByRole('menuitem', { name: 'Sign out' }).click();
  await expect(page).toHaveURL(/\/login$/);
  await expect(page.getByRole('heading', { name: 'Welcome' })).toBeVisible();
  const cookies = await page.context().cookies();
  expect(cookies.find((cookie) => cookie.name === 'astrum_session')).toBeUndefined();
});

test('anonymous access opens the shell without signing in', async ({ page, server }) => {
  await server.setAnonymous(true);
  await page.goto('/');
  await expect(page.getByRole('heading', { level: 1, name: 'Overview' })).toBeVisible();
  await expect(page.getByRole('link', { name: 'Sign in' })).toBeVisible();
  await expect(
    page.getByRole('navigation', { name: 'Main menu' }).getByRole('link', { name: 'Settings' }),
  ).toHaveCount(0);

  await page.getByRole('link', { name: 'Sign in' }).click();
  await expect(page).toHaveURL(/\/login$/);
  await page.getByRole('link', { name: 'Continue without signing in' }).click();
  await expect(page.getByRole('heading', { level: 1, name: 'Overview' })).toBeVisible();

  await server.setAnonymous(false);
  await page.goto('/');
  await expect(page).toHaveURL(/\/login$/);
});

async function endSession(page: Page, request: APIRequestContext, url: string) {
  const cookies = await page.context().cookies();
  const session = cookies.find((cookie) => cookie.name === 'astrum_session')?.value;
  const csrf = cookies.find((cookie) => cookie.name === 'astrum_csrf')?.value;
  expect(session && csrf).toBeTruthy();
  const ended = await request.delete(`${url}/service/rest/internal/session`, {
    headers: { Cookie: `astrum_session=${session}; astrum_csrf=${csrf}`, 'X-Astrum-CSRF': csrf ?? '' },
  });
  expect(ended.status()).toBe(204);
  await page.evaluate(() => document.dispatchEvent(new Event('visibilitychange', { bubbles: true })));
}

test('an ended session leads back to the login page', async ({ page, server, request }) => {
  await page.goto('/login?returnTo=%2F%3Ftab%3Dx');
  await signIn(page, server.adminPassword);
  await expect(page.getByRole('heading', { level: 1, name: 'Overview' })).toBeVisible();
  await endSession(page, request, server.url);
  await expect(page).toHaveURL(/\/login\?returnTo=%2F%3Ftab%3Dx$/);
  await expect(page.getByRole('status')).toContainText('Your session has ended');
  await signIn(page, server.adminPassword);
  await expect(page).toHaveURL(/\/\?tab=x$/);
});

test('with anonymous access an ended session asks in a dialog', async ({ page, server, request }) => {
  await server.setAnonymous(true);
  await page.goto('/login');
  await signIn(page, server.adminPassword);
  await expect(page.getByRole('button', { name: /Account menu of/ })).toBeVisible();
  await endSession(page, request, server.url);
  const dialog = page.getByRole('alertdialog', { name: 'Your session has ended' });
  await expect(dialog).toBeVisible();
  await expect(page.getByText('There are no repositories you may browse yet.')).toBeVisible();
  await dialog.getByRole('button', { name: 'Sign in' }).click();
  await expect(page).toHaveURL(/\/login$/);
  await signIn(page, server.adminPassword);
  await expect(page.getByRole('button', { name: /Account menu of/ })).toBeVisible();
});
