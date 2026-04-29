import { execFileSync } from 'node:child_process';
import { expect, test } from './fixtures';
import { AdminApi, signInAs, signOut, toast } from './security-support';

function curlStatus(url: string, user: string, password: string): string {
  return execFileSync(
    'curl',
    ['-s', '-o', '/dev/null', '-w', '%{http_code}', '-u', `${user}:${password}`, url],
    {
      encoding: 'utf8',
    },
  );
}

test('the own account is edited and the password changed with the current one', async ({
  page,
  request,
  server,
}) => {
  const admin = new AdminApi(request, server);
  await admin.user('e2e-ann', 'ann-password-1', []);
  await signInAs(page, 'e2e-ann', 'ann-password-1');
  await page.getByRole('button', { name: /Account menu of/ }).click();
  await page.getByRole('menuitem', { name: 'Account' }).click();
  await expect(page).toHaveURL(/\/user\/account$/);

  const first = page.getByRole('textbox', { name: /^First name/ });
  await first.fill('Ann');
  await page.getByRole('button', { name: 'Save changes' }).click();
  await expect(toast(page, 'Your account is saved')).toBeVisible();
  await expect(page.getByRole('button', { name: 'Account menu of Ann e2e-ann' })).toBeVisible();
  await page.reload();
  await expect(first).toHaveValue('Ann');

  await page.getByLabel('Current Password').fill('not-my-password');
  await page.getByLabel(/^New Password/).fill('ann-password-2');
  await page.getByLabel('Repeat New Password').fill('ann-password-2');
  await page.getByRole('button', { name: 'Change Password' }).click();
  await expect(page.getByText('The current password is wrong')).toBeVisible();
  await expect(page.getByLabel('Current Password')).toHaveAttribute('aria-invalid', 'true');

  await page.getByLabel('Current Password').fill('ann-password-1');
  await page.getByRole('button', { name: 'Change Password' }).click();
  await expect(toast(page, 'Your password is changed. Your other sessions have ended.')).toBeVisible();
  await expect(page.getByLabel('Current Password')).toHaveValue('');

  await signOut(page);
  await signInAs(page, 'e2e-ann', 'ann-password-2');
});

test('an access token is created, copied, used with curl and revoked', async ({
  page,
  context,
  request,
  server,
}) => {
  const admin = new AdminApi(request, server);
  await admin.user('e2e-tok', 'tok-password-1', []);
  await context.grantPermissions(['clipboard-read', 'clipboard-write'], { origin: server.url });
  await signInAs(page, 'e2e-tok', 'tok-password-1');
  await page.getByRole('button', { name: /Account menu of/ }).click();
  await page.getByRole('menuitem', { name: 'Access tokens' }).click();
  await expect(page.getByText('You do not have any access tokens yet.')).toBeVisible();

  await page.getByRole('textbox', { name: 'Name', exact: true }).fill('e2e-ci');
  await page.getByRole('combobox', { name: 'Access' }).click();
  await page.getByRole('option', { name: 'Read and write' }).click();
  await page.getByRole('combobox', { name: 'Lifetime' }).click();
  await page.getByRole('option', { name: '30 days' }).click();
  await page.getByRole('button', { name: 'Create Token' }).click();
  const dialog = page.getByRole('dialog', { name: 'Token Created' });
  await expect(dialog.getByText(/It is shown this one time only/)).toBeVisible();
  const token = (await dialog.getByTestId('created-token').textContent()) ?? '';
  expect(token).toMatch(/^astrum_[A-Za-z0-9]{40}$/);
  await dialog.getByRole('button', { name: 'Copy' }).first().click();
  expect(await page.evaluate(() => navigator.clipboard.readText())).toBe(token);
  await dialog.getByRole('tab', { name: /^curl/ }).click();
  await expect(dialog.getByText(`curl -u e2e-tok:${token} -O`, { exact: false })).toBeVisible();
  await dialog.getByRole('button', { name: 'Done' }).click();
  await expect(page.getByText(token)).toHaveCount(0);

  const row = page.getByRole('row', { name: /e2e-ci/ });
  await expect(row).toContainText('Read and write');
  await expect(row).toContainText('Not used yet');
  expect(curlStatus(`${server.url}/service/rest/internal/ui/user`, 'e2e-tok', token)).toBe('200');
  expect(curlStatus(`${server.url}/service/rest/internal/ui/user`, 'e2e-tok', 'astrum_wrong')).toBe('401');

  expect(curlStatus(`${server.url}/service/rest/internal/access-tokens`, 'e2e-tok', token)).toBe('403');

  await page.reload();
  await row.getByRole('button', { name: 'Revoke e2e-ci' }).click();
  await page.getByRole('alertdialog').getByRole('button', { name: 'Revoke' }).click();
  await expect(toast(page, 'Token “e2e-ci” is revoked')).toBeVisible();
  await expect(page.getByText('You do not have any access tokens yet.')).toBeVisible();
  expect(curlStatus(`${server.url}/service/rest/internal/ui/user`, 'e2e-tok', token)).toBe('401');
});
