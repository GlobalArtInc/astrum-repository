import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: 'e2e',
  globalSetup: './e2e/global-setup.ts',
  fullyParallel: false,
  workers: 1,
  timeout: 30_000,
  expect: { timeout: 5_000 },
  reporter: [['list']],
  outputDir: 'test-results',
  use: {
    browserName: 'chromium',
    viewport: { width: 1440, height: 900 },
    launchOptions: { executablePath: process.env.ASTRUM_E2E_CHROMIUM ?? '/opt/pw-browsers/chromium' },
    trace: 'retain-on-failure',
    screenshot: 'only-on-failure',
  },
});
