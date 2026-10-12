import { mkdirSync } from 'node:fs';
import { join } from 'node:path';
import type { Page } from '@playwright/test';
import { expect, signIn, test } from './fixtures';

const DIR = process.env.ASTRUM_SHOTS_DIR;
const SIZES = [
  { name: '1440', width: 1440, height: 900 },
  { name: '390', width: 390, height: 844 },
] as const;
const THEMES = ['dark', 'light'] as const;

test.skip(!DIR, 'set ASTRUM_SHOTS_DIR to take screenshots');

async function shoot(page: Page, name: string) {
  if (DIR) {
    mkdirSync(DIR, { recursive: true });
    await page.waitForTimeout(250);
    await page.screenshot({ path: join(DIR, `${name}.png`) });
  }
}

for (const size of SIZES) {
  for (const theme of THEMES) {
    test(`screens at ${size.name} in ${theme}`, async ({ page, server }) => {
      await page.setViewportSize({ width: size.width, height: size.height });
      await page.addInitScript(
        (value) => window.localStorage.setItem('astrum-theme-preference', value),
        theme,
      );
      await page.goto('/login');
      await expect(page.getByRole('heading', { name: 'Welcome' })).toBeVisible();
      await shoot(page, `login-${size.name}-${theme}`);
      await signIn(page, server.adminPassword);
      await expect(page.getByRole('heading', { level: 1, name: 'Overview' })).toBeVisible();
      await shoot(page, `dashboard-${size.name}-${theme}`);
      await page.goto('/settings');
      await expect(page.getByRole('heading', { level: 1, name: 'Settings' })).toBeVisible();
      await shoot(page, `settings-${size.name}-${theme}`);
      await page.goto('/no/such/page');
      await expect(page.getByRole('heading', { level: 1, name: 'Page not found' })).toBeVisible();
      await shoot(page, `not-found-${size.name}-${theme}`);
      await page.goto('/');
      await page.getByRole('button', { name: /Account menu of/ }).click();
      await shoot(page, `user-menu-${size.name}-${theme}`);
      await page.keyboard.press('Escape');
      await page.keyboard.press('Control+k');
      await expect(page.getByRole('dialog', { name: 'Search' })).toBeVisible();
      await shoot(page, `search-panel-${size.name}-${theme}`);
      await page.keyboard.press('Escape');
      if (size.width < 1024) {
        await page.getByRole('button', { name: 'Open the menu' }).click();
        await expect(page.getByRole('dialog', { name: 'Main menu' })).toBeVisible();
        await shoot(page, `drawer-${size.name}-${theme}`);
      } else {
        await page.getByRole('button', { name: 'Collapse the menu' }).click();
        await shoot(page, `menu-collapsed-${size.name}-${theme}`);
        await page.getByRole('button', { name: 'Expand the menu' }).click();
      }
    });
  }
}
