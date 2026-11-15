import { expect, test } from './fixtures';
import { signInAs, toast } from './security-support';

test.beforeEach(async ({ page, server }) => {
  await signInAs(page, 'admin', server.adminPassword);
});

test('an address added to the content list is saved and read back', async ({ page }) => {
  await page.goto('/admin/security/ip-allow');
  await expect(page.getByRole('heading', { name: 'IP allow list' })).toBeVisible();
  const content = page.getByRole('textbox', { name: 'Content', exact: true });
  await content.fill('10.20.30.0/24');
  await content.press('Enter');
  await page.getByRole('button', { name: 'Save', exact: true }).click();
  await expect(toast(page, 'The allow lists are saved')).toBeVisible();
  await page.reload();
  await expect(page.getByRole('list', { name: 'Content' })).toContainText('10.20.30.0/24');
});

test('a management list without the address in use is refused with a message', async ({ page }) => {
  await page.goto('/admin/security/ip-allow');
  const management = page.getByRole('textbox', { name: 'Management', exact: true });
  await management.fill('10.9.9.9');
  await management.press('Enter');
  await page.getByRole('button', { name: 'Save', exact: true }).click();
  await expect(page.getByText(/The management list must contain your address/)).toBeVisible();
  await expect(toast(page, 'The allow lists are saved')).toHaveCount(0);
});
