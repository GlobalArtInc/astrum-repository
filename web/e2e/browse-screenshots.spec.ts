import { mkdirSync } from 'node:fs';
import { join } from 'node:path';
import type { Page } from '@playwright/test';
import { SEED, seedBrowse } from './browse-seed';
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

const COMPONENT = `/search/component/${encodeURIComponent('maven2:org.acme:demo-lib')}`;

for (const size of SIZES) {
  for (const theme of THEMES) {
    test(`browse screens at ${size.name} in ${theme}`, async ({ page, server }) => {
      test.setTimeout(120_000);
      await seedBrowse(server.url, server.adminPassword);
      await page.setViewportSize({ width: size.width, height: size.height });
      await page.addInitScript(
        (value) => window.localStorage.setItem('astrum-theme-preference', value),
        theme,
      );
      const suffix = `${size.name}-${theme}`;
      await page.goto('/login');
      await signIn(page, server.adminPassword);
      await expect(page.getByRole('heading', { level: 1, name: 'Overview' })).toBeVisible();
      await expect(page.getByRole('region', { name: 'Most downloaded' })).toBeVisible();
      await shoot(page, `dashboard-${suffix}`);

      await page.goto('/browse');
      await expect(page.getByRole('heading', { level: 1, name: 'Repositories' })).toBeVisible();
      await expect(page.getByRole('link', { name: SEED.maven, exact: true })).toBeVisible();
      await shoot(page, `browse-list-${suffix}`);
      if (size.width < 1024) {
        await page.getByRole('button', { name: 'Filters' }).click();
        await expect(page.getByRole('dialog', { name: 'Filters' })).toBeVisible();
        await shoot(page, `browse-filters-${suffix}`);
        await page.keyboard.press('Escape');
      }

      await page.goto(`/browse/${SEED.maven}`);
      await expect(page.getByRole('tree')).toBeVisible();
      await shoot(page, `repository-${suffix}`);
      await page.goto(`/browse/${SEED.maven}/org/acme/demo-lib/2.0.0/`);
      await expect(page.getByRole('heading', { level: 2, name: 'demo-lib:2.0.0' })).toBeVisible();
      await shoot(page, `repository-component-${suffix}`);
      await page.goto(`/browse/${SEED.maven}/org/acme/demo-lib/2.0.0/demo-lib-2.0.0.jar`);
      await expect(page.getByRole('heading', { level: 2, name: 'demo-lib-2.0.0.jar' })).toBeVisible();
      await shoot(page, `repository-asset-${suffix}`);
      await page.goto(`/browse/${SEED.raw}/docs/`);
      await expect(page.getByRole('heading', { level: 2, name: 'docs' })).toBeVisible();
      await shoot(page, `repository-folder-${suffix}`);
      if (size.width < 1024) {
        await page.getByRole('button', { name: 'Back to the tree' }).click();
        await expect(page.getByRole('tree')).toBeVisible();
        await shoot(page, `repository-tree-${suffix}`);
      }
      await page.goto(`/browse/${SEED.docker}/v2/team/app/manifests/1.0`);
      await expect(page.getByRole('heading', { level: 2, name: 'team/app:1.0' })).toBeVisible();
      await shoot(page, `repository-docker-${suffix}`);
      await page.goto(`/browse/${SEED.rawGroup}`);
      await expect(page.getByRole('tree')).toBeVisible();
      await page.getByRole('searchbox', { name: `Search in ${SEED.rawGroup}` }).fill('readme');
      await expect(page.getByRole('list', { name: 'Search results' })).toBeVisible();
      await shoot(page, `repository-search-${suffix}`);
      await page.goto(`/browse/${SEED.npm}`);
      await page.getByRole('button', { name: 'Usage' }).click();
      await expect(page.getByRole('dialog', { name: `Connect to ${SEED.npm}` })).toBeVisible();
      await shoot(page, `repository-usage-${suffix}`);
      await page.keyboard.press('Escape');

      await page.goto(COMPONENT);
      await expect(page.getByRole('combobox', { name: 'Version' })).toBeVisible();
      await shoot(page, `component-overview-${suffix}`);
      for (const tab of ['files', 'usage', 'versions']) {
        await page.goto(`${COMPONENT}?tab=${tab}`);
        await expect(page.getByRole('combobox', { name: 'Version' })).toBeVisible();
        await page.waitForTimeout(300);
        await shoot(page, `component-${tab}-${suffix}`);
      }
    });
  }
}
