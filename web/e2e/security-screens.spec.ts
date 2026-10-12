import { mkdirSync } from 'node:fs';
import { join } from 'node:path';
import type { Page } from '@playwright/test';
import { expect, noHorizontalScroll, signIn, test } from './fixtures';
import { AdminApi, signInAs } from './security-support';
import { startServer } from './server';

const DIR = process.env.ASTRUM_SHOTS_DIR;
const SIZES = [
  { name: '1440', width: 1440, height: 900 },
  { name: '390', width: 390, height: 844 },
] as const;
const THEMES = ['dark', 'light'] as const;
const EXISTS = [200, 201, 204, 400];

test.skip(!DIR, 'set ASTRUM_SHOTS_DIR to take screenshots');

async function prepare(admin: AdminApi): Promise<void> {
  await admin.send(
    'POST',
    '/service/rest/v1/repositories/raw/hosted',
    {
      name: 'shot-files',
      online: true,
      storage: { blobStoreName: 'default', strictContentTypeValidation: false, writePolicy: 'allow' },
      raw: { contentDisposition: 'INLINE' },
    },
    EXISTS,
  );
  for (const file of ['docs/guide.txt', 'docs/install.txt', 'bin/tool.sh']) {
    await admin.putFile('shot-files', file, `content of ${file}`);
  }
  await admin.send(
    'POST',
    '/service/rest/v1/security/content-selectors',
    {
      name: 'shot-docs',
      description: 'Documentation files',
      expression: 'format == "raw" and path =^ "/docs/"',
    },
    EXISTS,
  );
  await admin.send(
    'POST',
    '/service/rest/v1/security/roles',
    {
      id: 'shot-readers',
      name: 'Readers',
      description: 'Search and read',
      privileges: ['as-search-read'],
      roles: [],
    },
    EXISTS,
  );
  await admin.send(
    'POST',
    '/service/rest/v1/security/roles',
    {
      id: 'shot-devs',
      name: 'Developers',
      description: 'Upload and everything readers do',
      privileges: ['as-component-upload', 'as-repository-view-*-*-browse'],
      roles: ['shot-readers'],
    },
    EXISTS,
  );
  await admin.send(
    'POST',
    '/service/rest/v1/security/users',
    {
      userId: 'shot-jane',
      firstName: 'Jane',
      lastName: 'Doe',
      emailAddress: 'jane@example.org',
      password: 'shot-password',
      status: 'active',
      roles: ['shot-devs'],
    },
    EXISTS,
  );
}

async function shoot(page: Page, name: string, heading?: string) {
  if (heading) {
    await expect(page.getByRole('heading', { level: 1, name: heading })).toBeVisible();
  }
  await page.waitForLoadState('networkidle');
  await page.waitForTimeout(300);
  await noHorizontalScroll(page);
  if (DIR) {
    const dir = DIR;
    mkdirSync(dir, { recursive: true });
    await page.screenshot({ path: join(dir, `${name}.png`) });
  }
}

for (const size of SIZES) {
  for (const theme of THEMES) {
    test(`security pages at ${size.name} in ${theme}`, async ({ page, request, server }) => {
      test.setTimeout(180_000);
      const tag = `${size.name}-${theme}`;
      await prepare(new AdminApi(request, server));
      await page.setViewportSize({ width: size.width, height: size.height });
      await page.addInitScript(
        (value) => window.localStorage.setItem('astrum-theme-preference', value),
        theme,
      );
      await signInAs(page, 'admin', server.adminPassword);

      await page.goto('/admin/repository/selectors');
      await shoot(page, `selectors-${tag}`, 'Content selectors');
      await page.goto('/admin/repository/selectors/shot-docs');
      await expect(page.getByText('/docs/guide.txt')).toBeVisible();
      await shoot(page, `selector-edit-${tag}`, 'shot-docs');
      await page.goto('/admin/repository/selectors/create');
      await shoot(page, `selector-new-${tag}`, 'Create a content selector');

      await page.goto('/admin/security/privileges');
      await shoot(page, `privileges-${tag}`, 'Privileges');
      await page.goto('/admin/security/privileges/create');
      await shoot(page, `privilege-types-${tag}`, 'Create a privilege');
      await page.goto('/admin/security/privileges/create/repository-view');
      await shoot(page, `privilege-new-${tag}`, 'Create a privilege: Repository view');
      await page.goto('/admin/security/privileges/as-all');
      await shoot(page, `privilege-builtin-${tag}`, 'as-all');

      await page.goto('/admin/security/roles');
      await shoot(page, `roles-${tag}`, 'Roles');
      await page.goto('/admin/security/roles/shot-devs');
      await expect(page.getByText('shot-devs → shot-readers')).toBeVisible();
      await shoot(page, `role-edit-${tag}`, 'Developers');
      await page.goto('/admin/security/roles/create');
      await shoot(page, `role-new-${tag}`, 'Create a role');

      await page.goto('/admin/security/users');
      await shoot(page, `users-${tag}`, 'Users');
      await page.goto('/admin/security/users/shot-jane');
      await shoot(page, `user-edit-${tag}`, 'shot-jane');
      await page.getByRole('button', { name: 'Change password' }).click();
      await shoot(page, `user-password-${tag}`);
      await page.keyboard.press('Escape');
      await page.goto('/admin/security/users/create');
      await shoot(page, `user-new-${tag}`, 'Create a user');

      await page.goto('/admin/security/anonymous');
      await shoot(page, `anonymous-${tag}`, 'Anonymous access');
      await page.goto('/admin/security/realms');
      await shoot(page, `realms-${tag}`, 'Realms');
      await page.goto('/admin/security/ssrf-protection');
      await page.getByRole('textbox', { name: 'Allowed addresses and networks' }).fill('10.0.0');
      await page.getByRole('textbox', { name: 'Allowed addresses and networks' }).press('Enter');
      await shoot(page, `ssrf-${tag}`, 'SSRF protection');

      await page.goto('/settings');
      await shoot(page, `settings-${tag}`, 'Settings');
      await page.goto('/user/account');
      await shoot(page, `account-${tag}`, 'Account');
      await page.goto('/user/access-tokens');
      await page.getByRole('textbox', { name: /^Name/ }).fill(`shot-${tag}`);
      await page.getByRole('button', { name: 'Create token' }).click();
      const created = page.getByRole('dialog', { name: 'Token created' });
      await expect(created).toBeVisible();
      const box = await created.boundingBox();
      expect((box?.x ?? 0) + (box?.width ?? 0)).toBeLessThanOrEqual(size.width);
      await shoot(page, `token-created-${tag}`);
      await page.getByRole('button', { name: 'Done' }).click();
      await shoot(page, `tokens-${tag}`, 'Access tokens');

      const fresh = await startServer({
        firstRun: true,
        port: Number(process.env.ASTRUM_E2E_FRESH_PORT ?? 0),
      });
      try {
        await page.goto(`${fresh.url}/login`);
        await signIn(page, fresh.adminPassword);
        await expect(page).toHaveURL(/\/onboarding$/);
        await shoot(page, `onboarding-${tag}`, 'Set up Astrum Repository');
      } finally {
        await fresh.stop();
      }
    });
  }
}
