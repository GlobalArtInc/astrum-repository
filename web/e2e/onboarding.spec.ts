import { expect, signIn, test } from './fixtures';
import { signOut, stranger } from './security-support';
import { startServer } from './server';

test('a fresh server walks the administrator through the first steps', async ({ page, browser }) => {
  test.setTimeout(120_000);
  const fresh = await startServer({ firstRun: true, port: Number(process.env.ASTRUM_E2E_FRESH_PORT ?? 0) });
  try {
    await page.goto(`${fresh.url}/login?returnTo=%2Fsettings`);
    await expect(page.getByText(/admin\.password/)).toBeVisible();
    await signIn(page, fresh.adminPassword);
    await expect(page).toHaveURL(/\/onboarding\?returnTo=%2Fsettings$/);
    await expect(page.getByRole('heading', { level: 1, name: 'Set up Astrum Repository' })).toBeVisible();
    await expect(page.getByText(/admin\.password\. The file is deleted/)).toBeVisible();

    await page.getByRole('button', { name: 'Skip for now' }).click();
    await expect(page).toHaveURL(/\/settings$/);
    await page.goto(`${fresh.url}/`);
    await expect(page.getByRole('heading', { level: 1, name: 'Overview' })).toBeVisible();
    await signOut(page);
    await signIn(page, fresh.adminPassword);
    await expect(page).toHaveURL(/\/onboarding$/);

    await page.getByLabel('Current password').fill('not-the-first-password');
    await page.getByLabel(/^New password/).fill('my-own-admin-password');
    await page.getByLabel('Repeat the new password').fill('my-own-admin-password');
    await page.getByRole('button', { name: 'Change password' }).click();
    await expect(page.getByText('The current password is wrong')).toBeVisible();
    await page.getByLabel('Current password').fill(fresh.adminPassword);
    await page.getByRole('button', { name: 'Change password' }).click();

    await expect(page.getByRole('heading', { name: 'Anonymous access' })).toBeVisible();
    await page.getByRole('radio', { name: /Allow anonymous access/ }).click();
    await page.getByRole('button', { name: 'Continue' }).click();
    await expect(page.getByRole('heading', { name: 'All set' })).toBeVisible();
    await page.getByRole('button', { name: 'Go to the dashboard' }).click();
    await expect(page).toHaveURL(`${fresh.url}/`);
    await expect(page.getByRole('heading', { level: 1, name: 'Overview' })).toBeVisible();

    const outsider = await stranger(browser, fresh.url);
    await outsider.page.goto('/');
    await expect(outsider.page.getByRole('heading', { level: 1, name: 'Overview' })).toBeVisible();
    await outsider.context.close();

    await signOut(page);
    await page.goto(`${fresh.url}/login`);
    await expect(page.getByText(/admin\.password/)).toHaveCount(0);
    await signIn(page, 'my-own-admin-password');
    await expect(page.getByRole('heading', { level: 1, name: 'Overview' })).toBeVisible();
    await expect(page).toHaveURL(`${fresh.url}/`);
  } finally {
    await fresh.stop();
  }
});
