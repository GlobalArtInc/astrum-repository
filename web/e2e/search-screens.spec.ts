import { mkdirSync } from 'node:fs';
import { join } from 'node:path';
import type { Page } from '@playwright/test';
import { hosted, seedSearch } from './content';
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
    await page.waitForTimeout(300);
    await page.screenshot({ path: join(DIR, `${name}.png`) });
  }
}

for (const size of SIZES) {
  for (const theme of THEMES) {
    test(`search, upload and audit screens at ${size.name} in ${theme}`, async ({ page, server }) => {
      test.setTimeout(90_000);
      await seedSearch(server);
      await hosted(server, 'raw', 'upload-raw', { raw: { contentDisposition: 'ATTACHMENT' } });
      await page.setViewportSize({ width: size.width, height: size.height });
      await page.addInitScript(
        (value) => window.localStorage.setItem('astrum-theme-preference', value),
        theme,
      );
      await page.goto('/login');
      await signIn(page, server.adminPassword);
      await expect(page.getByRole('heading', { level: 1, name: 'Overview' })).toBeVisible();
      const suffix = `${size.name}-${theme}`;

      await page.goto('/search');
      await expect(page.getByText('Search all repositories')).toBeVisible();
      await shoot(page, `search-start-${suffix}`);
      await page.goto('/search?q=tool');
      await expect(page.getByText(/^Shown: \d+$/)).toBeVisible();
      await shoot(page, `search-results-${suffix}`);
      await page.goto('/search?q=nothing-like-this');
      await expect(page.getByText('Nothing found')).toBeVisible();
      await shoot(page, `search-empty-${suffix}`);

      await page.goto('/search/custom');
      await expect(page.getByText('Build a query')).toBeVisible();
      await shoot(page, `criteria-start-${suffix}`);
      await page.goto('/search/custom?group=com.*');
      await expect(page.getByRole('button', { name: /^Open astrum-client/ }).first()).toBeVisible();
      await shoot(page, `criteria-components-${suffix}`);

      await page.goto('/browse/search-npm');
      await page.keyboard.press('Control+k');
      await page.getByRole('combobox', { name: 'Search components' }).fill('astrum');
      await expect(page.getByRole('group', { name: 'Components' })).toBeVisible();
      await shoot(page, `search-panel-${suffix}`);
      await page.keyboard.press('Escape');

      await page.goto('/upload');
      await expect(page.getByRole('link', { name: 'upload-raw' })).toBeVisible();
      await shoot(page, `upload-list-${suffix}`);
      await page.goto('/upload/upload-raw');
      await page.getByLabel(/Directory/).fill('/tools/linux');
      await page.getByLabel(/^File\*?$/).setInputFiles({
        name: 'agent-linux-amd64',
        mimeType: 'application/octet-stream',
        buffer: Buffer.alloc(2048, 7),
      });
      await shoot(page, `upload-raw-${suffix}`);
      await page.goto('/upload/search-maven');
      await page.getByRole('button', { name: 'Upload', exact: true }).click();
      await expect(page.getByText('Fix the marked fields and try again.')).toBeVisible();
      await page.getByLabel(/Group ID/).scrollIntoViewIfNeeded();
      await shoot(page, `upload-maven-errors-${suffix}`);

      await page.goto('/audit');
      await expect(page.getByRole('heading', { level: 1, name: 'Audit log' })).toBeVisible();
      await page
        .getByText(/^\d+ attributes?$/)
        .first()
        .click();
      await shoot(page, `audit-${suffix}`);
      if (size.width < 640) {
        await page.getByRole('button', { name: /^Filters/ }).click();
        await shoot(page, `audit-filters-${suffix}`);
      }
    });
  }
}
