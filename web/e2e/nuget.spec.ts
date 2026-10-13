import { mkdirSync } from 'node:fs';
import { join } from 'node:path';
import type { APIRequestContext, Page } from '@playwright/test';
import { ADMIN, expect, noHorizontalScroll, signIn, test } from './fixtures';
import { toast } from './security-support';

const DIR = process.env.ASTRUM_SHOTS_DIR;
const SIZES = [
  { name: '1440', width: 1440, height: 900 },
  { name: '390', width: 390, height: 844 },
] as const;
const THEMES = ['dark', 'light'] as const;
const HOSTED = 'nuget-hosted';

function basic(password: string, user = ADMIN): string {
  return `Basic ${Buffer.from(`${user}:${password}`).toString('base64')}`;
}

const CRC = Array.from({ length: 256 }, (_, n) => {
  let c = n;
  for (let k = 0; k < 8; k += 1) {
    c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
  }
  return c >>> 0;
});

function crc32(data: Buffer): number {
  let crc = 0xffffffff;
  for (const byte of data) {
    crc = (CRC[(crc ^ byte) & 0xff] ?? 0) ^ (crc >>> 8);
  }
  return (crc ^ 0xffffffff) >>> 0;
}

function zip(files: Array<[string, string]>): Buffer {
  const parts: Buffer[] = [];
  const central: Buffer[] = [];
  let offset = 0;
  for (const [name, text] of files) {
    const data = Buffer.from(text);
    const fileName = Buffer.from(name);
    const local = Buffer.alloc(30);
    local.writeUInt32LE(0x04034b50, 0);
    local.writeUInt16LE(20, 4);
    local.writeUInt32LE(crc32(data), 14);
    local.writeUInt32LE(data.length, 18);
    local.writeUInt32LE(data.length, 22);
    local.writeUInt16LE(fileName.length, 26);
    const entry = Buffer.alloc(46);
    entry.writeUInt32LE(0x02014b50, 0);
    entry.writeUInt16LE(20, 4);
    entry.writeUInt16LE(20, 6);
    entry.writeUInt32LE(crc32(data), 16);
    entry.writeUInt32LE(data.length, 20);
    entry.writeUInt32LE(data.length, 24);
    entry.writeUInt16LE(fileName.length, 28);
    entry.writeUInt32LE(offset, 42);
    parts.push(local, fileName, data);
    central.push(entry, fileName);
    offset += local.length + fileName.length + data.length;
  }
  const directory = Buffer.concat(central);
  const end = Buffer.alloc(22);
  end.writeUInt32LE(0x06054b50, 0);
  end.writeUInt16LE(files.length, 8);
  end.writeUInt16LE(files.length, 10);
  end.writeUInt32LE(directory.length, 12);
  end.writeUInt32LE(offset, 16);
  return Buffer.concat([...parts, directory, end]);
}

function nupkg(id: string, version: string): Buffer {
  const nuspec = `<?xml version="1.0" encoding="utf-8"?>
<package xmlns="http://schemas.microsoft.com/packaging/2013/05/nuspec.xsd">
  <metadata>
    <id>${id}</id>
    <version>${version}</version>
    <authors>Astrum Team</authors>
    <description>A package pushed by the end-to-end tests.</description>
    <tags>astrum e2e</tags>
  </metadata>
</package>`;
  return zip([
    [`${id}.nuspec`, nuspec],
    ['lib/net8.0/Probe.dll', `${id} ${version}`],
  ]);
}

async function push(
  request: APIRequestContext,
  url: string,
  headers: Record<string, string>,
  bytes: Buffer,
): Promise<number> {
  const response = await request.put(`${url}/repository/${HOSTED}/`, {
    headers,
    multipart: { package: { name: 'package.nupkg', mimeType: 'application/octet-stream', buffer: bytes } },
  });
  return response.status();
}

async function seed(request: APIRequestContext, url: string, password: string, realm: boolean) {
  const headers = { Authorization: basic(password) };
  const created = await request.post(`${url}/service/rest/v1/repositories/nuget/hosted`, {
    headers,
    data: {
      name: HOSTED,
      online: true,
      storage: { blobStoreName: 'default', strictContentTypeValidation: true, writePolicy: 'ALLOW' },
    },
  });
  expect([201, 400]).toContain(created.status());
  const realms = realm
    ? ['AstrumAuthenticatingRealm', 'AccessTokenRealm', 'NuGetApiKey']
    : ['AstrumAuthenticatingRealm', 'AccessTokenRealm'];
  const active = await request.put(`${url}/service/rest/v1/security/realms/active`, {
    headers,
    data: realms,
  });
  expect(active.ok()).toBe(true);
  expect(await push(request, url, headers, nupkg('Astrum.Probe', '1.0.0'))).toBe(201);
}

async function login(page: Page, password: string) {
  await page.goto('/login');
  await signIn(page, password);
  await expect(page.getByRole('button', { name: /Account menu of/ })).toBeVisible();
}

async function shoot(page: Page, name: string) {
  if (DIR) {
    mkdirSync(DIR, { recursive: true });
    await page.waitForTimeout(300);
    await page.screenshot({ path: join(DIR, `${name}.png`) });
  }
}

async function openKeyPage(page: Page) {
  await page.getByRole('button', { name: /Account menu of/ }).click();
  await page.getByRole('menuitem', { name: 'NuGet API Key' }).click();
  await expect(page).toHaveURL(/\/user\/nuget-api-key$/);
  await expect(page.getByRole('heading', { level: 1, name: 'NuGet API Key' })).toBeVisible();
}

test('the NuGet API key is revealed, used by a client and reset', async ({ page, request, server }) => {
  await seed(request, server.url, server.adminPassword, true);
  await login(page, server.adminPassword);
  await openKeyPage(page);
  await expect(page.getByText('--password {API_KEY}', { exact: false })).toBeVisible();

  await page.getByRole('button', { name: 'Access API Key' }).click();
  const auth = page.getByRole('dialog', { name: 'Authenticate' });
  await auth.getByLabel('Password').fill('not-the-password');
  await auth.getByRole('button', { name: 'Authenticate' }).click();
  await expect(page.getByText('Authentication failed', { exact: true })).toBeVisible();
  await auth.getByLabel('Password').fill(server.adminPassword);
  await auth.getByRole('button', { name: 'Authenticate' }).click();
  const shown = page.getByRole('dialog', { name: 'NuGet API Key' });
  const key = (await shown.getByTestId('nuget-api-key').textContent()) ?? '';
  expect(key).toMatch(/^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/);
  await expect(shown.getByText(`--password ${key}`, { exact: false })).toBeVisible();
  await shown.getByRole('button', { name: 'Close' }).click();
  await expect(page.getByText(`--password ${key}`, { exact: false })).toBeVisible();

  expect(await push(request, server.url, { 'X-NuGet-ApiKey': key }, nupkg('Pushed.WithKey', '1.0.0'))).toBe(
    201,
  );

  await page.getByRole('button', { name: 'Reset API Key' }).click();
  await page
    .getByRole('alertdialog', { name: 'Reset NuGet API Key' })
    .getByRole('button', { name: 'Reset Key' })
    .click();
  const again = page.getByRole('dialog', { name: 'Authenticate' });
  await expect(again.getByText('Please authenticate to reset your NuGet API key.')).toBeVisible();
  await again.getByLabel('Password').fill(server.adminPassword);
  await again.getByRole('button', { name: 'Authenticate' }).click();
  await expect(toast(page, 'API key reset. Generate a new one.')).toBeVisible();
  await expect(page.getByText('--password {API_KEY}', { exact: false })).toBeVisible();
  expect(await push(request, server.url, { 'X-NuGet-ApiKey': key }, nupkg('Pushed.WithKey', '2.0.0'))).toBe(
    401,
  );

  await page.goto('/#user/nugetapitoken');
  await expect(page).toHaveURL(/\/user\/nuget-api-key$/);
  await page.setViewportSize({ width: 390, height: 844 });
  await expect(page.getByRole('heading', { level: 1, name: 'NuGet API Key' })).toBeVisible();
  await noHorizontalScroll(page);
});

test('the key page stays hidden while the realm is off', async ({ page, request, server }) => {
  await seed(request, server.url, server.adminPassword, false);
  await login(page, server.adminPassword);
  await page.getByRole('button', { name: /Account menu of/ }).click();
  await expect(page.getByRole('menuitem', { name: 'Access tokens' })).toBeVisible();
  await expect(page.getByRole('menuitem', { name: 'NuGet API Key' })).toHaveCount(0);
  await page.keyboard.press('Escape');
  await page.goto('/user/nuget-api-key');
  await expect(page.getByRole('heading', { level: 1, name: 'Overview' })).toBeVisible();
});

test('NuGet packages show their usage and are found by their id', async ({ page, request, server }) => {
  await seed(request, server.url, server.adminPassword, false);
  await login(page, server.adminPassword);
  await page.goto(`/browse/${HOSTED}/Astrum.Probe/1.0.0/`);
  await expect(page.getByRole('heading', { level: 2, name: 'Astrum.Probe:1.0.0' })).toBeVisible();
  const index = `${server.url}/repository/${HOSTED}/index.json`;
  await expect(
    page.getByText(`dotnet add package Astrum.Probe --version 1.0.0 --source ${index}`),
  ).toBeVisible();
  await page.getByRole('tab', { name: /^NuGet\.Config/ }).click();
  await expect(page.getByText('Put into NuGet.Config next to the solution')).toBeVisible();
  await page.getByRole('tab', { name: /^dotnet nuget push/ }).click();
  await expect(page.getByText('The API key is the NuGet API key of your user on this server')).toBeVisible();
  await expect(
    page.getByText(
      `dotnet nuget push Astrum.Probe.1.0.0.nupkg --source ${server.url}/repository/${HOSTED}/ --api-key <api-key>`,
    ),
  ).toBeVisible();

  await page.goto('/search/custom?nuget.id=Astrum.Probe');
  await expect(page.getByRole('button', { name: 'Open Astrum.Probe in the repository' })).toBeVisible();
});

test('a NuGet proxy is created with its protocol version and query cache age', async ({
  page,
  request,
  server,
}) => {
  await login(page, server.adminPassword);
  await page.goto('/admin/repository/repositories/create/nuget/proxy');
  const section = page.getByRole('region', { name: 'NuGet' });
  await expect(section.getByText('NuGet proxy repository configuration')).toBeVisible();
  await page.getByRole('textbox', { name: 'Name', exact: true }).fill('nuget-org');
  await page
    .getByRole('textbox', { name: 'Remote Storage', exact: true })
    .fill('https://api.nuget.invalid/v3/index.json');
  await section.getByRole('combobox', { name: 'Protocol Version' }).click();
  await page.getByRole('option', { name: 'NuGet V2', exact: true }).click();
  await section.getByRole('spinbutton', { name: /^Metadata Query Cache Age/ }).fill('600');
  await page.getByRole('button', { name: 'Continue' }).click();
  await expect(page).toHaveURL(/\/admin\/repository\/repositories$/);
  const response = await request.get(`${server.url}/service/rest/v1/repositories/nuget/proxy/nuget-org`, {
    headers: { Authorization: basic(server.adminPassword) },
  });
  expect((await response.json()).nugetProxy).toEqual({ queryCacheItemMaxAge: 600, nugetVersion: 'V2' });
});

test.describe('screens', () => {
  test.skip(!DIR, 'set ASTRUM_SHOTS_DIR to take screenshots');
  for (const size of SIZES) {
    for (const theme of THEMES) {
      test(`NuGet screens at ${size.name} in ${theme}`, async ({ page, request, server }) => {
        test.setTimeout(120_000);
        await seed(request, server.url, server.adminPassword, true);
        await page.setViewportSize({ width: size.width, height: size.height });
        await page.addInitScript(
          (value) => window.localStorage.setItem('astrum-theme-preference', value),
          theme,
        );
        const suffix = `${size.name}-${theme}`;
        await login(page, server.adminPassword);
        await openKeyPage(page);
        await shoot(page, `nuget-key-${suffix}`);
        await page.getByRole('button', { name: 'Access API Key' }).click();
        await shoot(page, `nuget-key-authenticate-${suffix}`);
        const auth = page.getByRole('dialog', { name: 'Authenticate' });
        await auth.getByLabel('Password').fill(server.adminPassword);
        await auth.getByRole('button', { name: 'Authenticate' }).click();
        await expect(page.getByRole('dialog', { name: 'NuGet API Key' })).toBeVisible();
        await shoot(page, `nuget-key-shown-${suffix}`);
        await page
          .getByRole('dialog', { name: 'NuGet API Key' })
          .getByRole('button', { name: 'Close' })
          .click();
        await page.getByRole('button', { name: 'Reset API Key' }).click();
        await shoot(page, `nuget-key-reset-${suffix}`);
        await page.keyboard.press('Escape');

        await page.goto(`/browse/${HOSTED}/Astrum.Probe/1.0.0/`);
        await expect(page.getByRole('heading', { level: 2, name: 'Astrum.Probe:1.0.0' })).toBeVisible();
        await shoot(page, `nuget-usage-${suffix}`);
        await page.getByRole('tab', { name: /^dotnet nuget push/ }).click();
        await shoot(page, `nuget-usage-push-${suffix}`);

        await page.goto('/admin/repository/repositories/create/nuget/proxy');
        await expect(page.getByRole('region', { name: 'NuGet' })).toBeVisible();
        await shoot(page, `nuget-proxy-form-${suffix}`);
      });
    }
  }
});
