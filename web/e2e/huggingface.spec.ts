import type { Page } from '@playwright/test';
import { call } from './content';
import { expect, noHorizontalScroll, type Server, signIn, test } from './fixtures';

test.describe.configure({ timeout: 120_000 });

async function signInAsAdmin(page: Page, server: Server) {
  await page.goto('/login');
  await signIn(page, server.adminPassword);
  await expect(page.getByRole('button', { name: /Account menu of/ })).toBeVisible();
}

test('a hugging face proxy is made through the form and offers its criteria', async ({ page, server }) => {
  await signInAsAdmin(page, server);
  await page.goto('/admin/repository/repositories');
  await page.getByRole('link', { name: 'Create Repository' }).first().click();
  await page.getByRole('button', { name: 'Hugging Face', exact: true }).click();
  await page.getByRole('button', { name: /Hugging Face proxy/ }).click();
  await page.getByRole('textbox', { name: 'Name', exact: true }).fill('hub');
  await page.getByRole('textbox', { name: 'Remote Storage', exact: true }).fill('https://huggingface.co');
  await page.getByRole('button', { name: 'Continue' }).click();
  await expect(page).toHaveURL(/\/admin\/repository\/repositories$/);
  const stored = await (
    await call(server, 'GET', '/service/rest/v1/repositories/huggingface/proxy/hub')
  ).json();
  expect(stored.proxy.remoteUrl).toBe('https://huggingface.co');
  expect(stored.format).toBe('huggingface');

  await page.goto('/search/custom');
  await page.getByRole('combobox', { name: 'Field' }).click();
  await expect(page.getByRole('option', { name: 'Hugging Face Branch or Tag' })).toBeVisible();
  await page.getByRole('option', { name: 'Hugging Face File' }).click();
  await page.getByRole('combobox', { name: 'Condition' }).click();
  await page.getByRole('option', { name: 'Contains' }).click();
  await page.getByRole('textbox', { name: 'Value' }).fill('config');
  await page.getByRole('button', { name: 'Search', exact: true }).click();
  await expect(page).toHaveURL(/\/search\/custom\?huggingface\.file=\*config\*$/);

  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/browse/hub');
  await expect(page.getByRole('heading', { level: 1, name: 'hub' })).toBeVisible();
  await noHorizontalScroll(page);
});
