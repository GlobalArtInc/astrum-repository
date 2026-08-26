import { createHash } from 'node:crypto';
import { mkdirSync } from 'node:fs';
import { createServer } from 'node:http';
import { join } from 'node:path';
import { gzipSync } from 'node:zlib';
import type { Page } from '@playwright/test';
import { call, ensure, tar } from './content';
import { expect, noHorizontalScroll, type Server, signIn, test } from './fixtures';

test.describe.configure({ timeout: 120_000 });

const DIR = process.env.ASTRUM_SHOTS_DIR;
const POD = 'AstrumKit';
const VERSION = '1.2.0';

function shard(name: string): string[] {
  const hashed = createHash('md5').update(name).digest('hex');
  return [hashed[0] ?? '', hashed[1] ?? '', hashed[2] ?? ''];
}

const SPEC_PATH = `Specs/${shard(POD).join('/')}/${POD}/${VERSION}/${POD}.podspec.json`;

interface Cdn {
  url: string;
  close(): Promise<void>;
}

async function startCdn(): Promise<Cdn> {
  const archive = gzipSync(
    tar([
      [`kit-${VERSION}/LICENSE`, 'MIT\n'],
      [`kit-${VERSION}/Sources/Kit.swift`, 'public let kit = 1\n'],
    ]),
  );
  let base = '';
  const files = (): Record<string, Buffer | string> => ({
    'CocoaPods-version.yml': '---\nmin: 1.0.0\nlast: 1.17.0\nprefix_lengths:\n- 1\n- 1\n- 1\n',
    'all_pods.txt': `${POD}\n`,
    [`all_pods_versions_${shard(POD).join('_')}.txt`]: `${POD}/${VERSION}\n`,
    'deprecated_podspecs.txt': '',
    [SPEC_PATH]: `${JSON.stringify(
      {
        name: POD,
        version: VERSION,
        summary: 'A pod of the end-to-end tests.',
        license: 'MIT',
        source: { http: `${base}/archives/v${VERSION}/kit-${VERSION}.tar.gz` },
        source_files: 'Sources/**/*.swift',
      },
      null,
      2,
    )}\n`,
    [`archives/v${VERSION}/kit-${VERSION}.tar.gz`]: archive,
  });
  const server = createServer((request, response) => {
    const body = files()[(request.url ?? '/').slice(1)];
    if (body === undefined) {
      response.writeHead(404).end();
      return;
    }
    response.writeHead(200).end(body);
  });
  const port = await new Promise<number>((done) => {
    server.listen(0, '127.0.0.1', () => {
      const address = server.address();
      done(typeof address === 'object' && address ? address.port : 0);
    });
  });
  base = `http://127.0.0.1:${port}`;
  return {
    url: `${base}/`,
    close: () =>
      new Promise((done) => {
        server.closeAllConnections();
        server.close(() => done());
      }),
  };
}

async function signInAsAdmin(page: Page, server: Server) {
  await page.goto('/login');
  await signIn(page, server.adminPassword);
  await expect(page.getByRole('button', { name: /Account menu of/ })).toBeVisible();
}

async function allowLoopback(server: Server) {
  await ensure(
    await call(server, 'PUT', '/service/rest/v1/security/ssrf-protection', {
      enabled: true,
      allowedIPs: ['127.0.0.1'],
      allowedDomains: [],
    }),
    'allow loopback remotes',
  );
}

async function warm(server: Server, repository: string) {
  for (const path of ['CocoaPods-version.yml', SPEC_PATH, `pods/${POD}/${VERSION}/kit-${VERSION}.tar.gz`]) {
    await ensure(await call(server, 'GET', `/repository/${repository}/${path}`), path);
  }
}

test('a CocoaPods proxy is made through the form, caches pods and shows their usage', async ({
  page,
  server,
}) => {
  const cdn = await startCdn();
  try {
    await allowLoopback(server);
    await signInAsAdmin(page, server);
    await page.goto('/admin/repository/repositories');
    await page.getByRole('link', { name: 'Create Repository' }).first().click();
    await page.getByRole('button', { name: 'CocoaPods', exact: true }).click();
    await page.getByRole('button', { name: /CocoaPods proxy/ }).click();
    await expect(page).toHaveURL(/\/create\/cocoapods\/proxy$/);
    await expect(page.getByText(/https:\/\/cdn\.cocoapods\.org\//)).toBeVisible();
    await page.getByRole('textbox', { name: 'Name', exact: true }).fill('trunk');
    await page.getByRole('textbox', { name: /Remote Storage/ }).fill(cdn.url);
    await page.getByRole('button', { name: 'Continue' }).click();
    await expect(page).toHaveURL(/\/admin\/repository\/repositories$/);
    const stored = await (
      await call(server, 'GET', '/service/rest/v1/repositories/cocoapods/proxy/trunk')
    ).json();
    expect(stored.proxy.remoteUrl).toBe(cdn.url);

    const spec = await (await call(server, 'GET', `/repository/trunk/${SPEC_PATH}`)).json();
    expect(spec.source.http).toBe(
      `${server.url}/repository/trunk/pods/${POD}/${VERSION}/kit-${VERSION}.tar.gz`,
    );
    await warm(server, 'trunk');

    await page.goto('/browse/trunk');
    const tree = page.getByRole('tree');
    await expect(tree.getByText('CocoaPods-version.yml', { exact: true })).toBeVisible();
    await tree.getByText(POD, { exact: true }).click();
    await tree.getByText(VERSION, { exact: true }).click();
    await tree.getByText(`kit-${VERSION}.tar.gz`).click();
    const url = `${server.url}/repository/trunk/`;
    await expect(page.getByRole('tab', { name: 'Podfile' })).toHaveAttribute('aria-selected', 'true');
    await expect(page.getByText(`source '${url}'`)).toBeVisible();
    await expect(page.getByText(`pod '${POD}', '${VERSION}'`)).toBeVisible();
    await page.getByRole('tab', { name: 'pod repo add-cdn' }).click();
    await expect(page.getByText(`pod repo add-cdn trunk ${url}`)).toBeVisible();

    await page.goto(`/search/component/${encodeURIComponent(`cocoapods::${POD}`)}?version=${VERSION}`);
    await expect(page.getByText(`pod '${POD}', '${VERSION}'`)).toBeVisible();

    await page.setViewportSize({ width: 390, height: 844 });
    await page.goto(`/browse/trunk/${POD}/${VERSION}/kit-${VERSION}.tar.gz`);
    await expect(page.getByRole('tab', { name: 'Podfile' })).toBeVisible();
    await noHorizontalScroll(page);
  } finally {
    await cdn.close();
  }
});

async function shoot(page: Page, name: string) {
  if (DIR) {
    mkdirSync(DIR, { recursive: true });
    await page.waitForTimeout(300);
    await page.screenshot({ path: join(DIR, `${name}.png`) });
  }
}

test('cocoapods screens', async ({ page, server }) => {
  test.skip(!DIR, 'set ASTRUM_SHOTS_DIR to take screenshots');
  test.setTimeout(240_000);
  const cdn = await startCdn();
  try {
    await allowLoopback(server);
    await ensure(
      await call(server, 'POST', '/service/rest/v1/repositories/cocoapods/proxy', {
        name: 'pods-shots',
        online: true,
        storage: { blobStoreName: 'default', strictContentTypeValidation: true },
        proxy: { remoteUrl: cdn.url, contentMaxAge: 1440, metadataMaxAge: 1440 },
        negativeCache: { enabled: true, timeToLive: 1440 },
        httpClient: { blocked: false, autoBlock: true },
      }),
      'create',
    );
    await warm(server, 'pods-shots');
    for (const size of [
      { name: '1440', width: 1440, height: 900 },
      { name: '390', width: 390, height: 844 },
    ]) {
      for (const theme of ['dark', 'light']) {
        const context = await page
          .context()
          .browser()
          ?.newContext({ baseURL: server.url, viewport: { width: size.width, height: size.height } });
        if (!context) {
          throw new Error('no browser');
        }
        await context.addInitScript(
          (value) => window.localStorage.setItem('astrum-theme-preference', value),
          theme,
        );
        const shotPage = await context.newPage();
        await signInAsAdmin(shotPage, server);
        const suffix = `${size.name}-${theme}`;
        await shotPage.goto('/admin/repository/repositories/create/cocoapods/proxy');
        await expect(shotPage.getByRole('textbox', { name: 'Name', exact: true })).toBeVisible();
        await shoot(shotPage, `cocoapods-create-${suffix}`);
        await shotPage.goto(`/browse/pods-shots/${POD}/${VERSION}/kit-${VERSION}.tar.gz`);
        await expect(shotPage.getByRole('tab', { name: 'Podfile' })).toBeVisible();
        await shoot(shotPage, `cocoapods-browse-archive-${suffix}`);
        await shotPage.goto(
          `/search/component/${encodeURIComponent(`cocoapods::${POD}`)}?version=${VERSION}`,
        );
        await expect(shotPage.getByText(`pod '${POD}', '${VERSION}'`)).toBeVisible();
        await shoot(shotPage, `cocoapods-component-${suffix}`);
        await context.close();
      }
    }
  } finally {
    await cdn.close();
  }
});
