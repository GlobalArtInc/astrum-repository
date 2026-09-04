import { mkdirSync } from 'node:fs';
import { join } from 'node:path';
import type { Page } from '@playwright/test';
import { call, zip } from './content';
import { expect, noHorizontalScroll, type Server, signIn, test } from './fixtures';

test.describe.configure({ timeout: 120_000 });

const DIR = process.env.ASTRUM_SHOTS_DIR;

async function expectOk(response: Response, what: string) {
  expect(response.ok, `${what}: ${response.status} ${await response.clone().text()}`).toBe(true);
}

async function signInAsAdmin(page: Page, server: Server) {
  await page.goto('/login');
  await signIn(page, server.adminPassword);
  await expect(page.getByRole('button', { name: /Account menu of/ })).toBeVisible();
}

function content(units: Array<[string, string]>): string {
  const body = units
    .map(
      ([id, version]) =>
        `    <unit id='${id}' version='${version}'>\n      <artifacts size='1'>\n        <artifact classifier='osgi.bundle' id='${id}' version='${version}'/>\n      </artifacts>\n    </unit>\n`,
    )
    .join('');
  return `<?xml version='1.0' encoding='UTF-8'?>\n<?metadataRepository version='1.1.0'?>\n<repository name='site' type='org.eclipse.equinox.internal.p2.metadata.repository.LocalMetadataRepository' version='1'>\n  <properties size='1'>\n    <property name='p2.timestamp' value='1'/>\n  </properties>\n  <units size='${units.length}'>\n${body}  </units>\n</repository>\n`;
}

function artifacts(items: Array<[string, string, string]>): string {
  const body = items
    .map(
      ([classifier, id, version]) =>
        `    <artifact classifier='${classifier}' id='${id}' version='${version}'>\n      <properties size='1'>\n        <property name='artifact.size' value='1'/>\n      </properties>\n    </artifact>\n`,
    )
    .join('');
  return `<?xml version='1.0' encoding='UTF-8'?>\n<?artifactRepository version='1.1.0'?>\n<repository name='site' type='org.eclipse.equinox.p2.artifact.repository.simpleRepository' version='1'>\n  <properties size='2'>\n    <property name='p2.timestamp' value='1'/>\n    <property name='p2.mirrorsURL' value='https://mirrors.example.org/list.xml'/>\n  </properties>\n  <mappings size='2'>\n    <rule filter='(&amp; (classifier=osgi.bundle))' output='\${repoUrl}/plugins/\${id}_\${version}.jar'/>\n    <rule filter='(&amp; (classifier=org.eclipse.update.feature))' output='\${repoUrl}/features/\${id}_\${version}.jar'/>\n  </mappings>\n  <artifacts size='${items.length}'>\n${body}  </artifacts>\n</repository>\n`;
}

function composite(kind: 'Content' | 'Artifacts', children: string[]): string {
  const instruction = kind === 'Content' ? 'compositeMetadataRepository' : 'compositeArtifactRepository';
  const type =
    kind === 'Content'
      ? 'org.eclipse.equinox.internal.p2.metadata.repository.CompositeMetadataRepository'
      : 'org.eclipse.equinox.internal.p2.artifact.repository.CompositeArtifactRepository';
  const body = children.map((child) => `    <child location='${child}'/>\n`).join('');
  return `<?xml version='1.0' encoding='UTF-8'?>\n<?${instruction} version='1.0.0'?>\n<repository name='site' type='${type}' version='1.0.0'>\n  <properties size='1'>\n    <property name='p2.timestamp' value='1'/>\n  </properties>\n  <children size='${children.length}'>\n${body}  </children>\n</repository>\n`;
}

function plugin(id: string, version: string, name: string): Buffer {
  return zip([
    [
      'META-INF/MANIFEST.MF',
      `Manifest-Version: 1.0\nBundle-ManifestVersion: 2\nBundle-Name: %pluginName\nBundle-SymbolicName: ${id}\nBundle-Version: ${version}\n`,
    ],
    ['OSGI-INF/l10n/bundle.properties', `pluginName = ${name}\n`],
  ]);
}

function feature(id: string, label: string): Buffer {
  return zip([
    ['feature.xml', `<feature id='${id}' label='%featureName' version='1.0.0'/>`],
    ['feature.properties', `featureName=${label}\n`],
  ]);
}

async function rawSite(server: Server, name: string) {
  await expectOk(
    await call(server, 'POST', '/service/rest/v1/repositories/raw/hosted', {
      name,
      online: true,
      storage: { blobStoreName: 'default', strictContentTypeValidation: false, writePolicy: 'ALLOW' },
    }),
    `create ${name}`,
  );
}

async function put(server: Server, path: string, body: Buffer | string) {
  await expectOk(
    await call(server, 'PUT', path, typeof body === 'string' ? Buffer.from(body) : body, {
      'Content-Type': 'application/octet-stream',
    }),
    `put ${path}`,
  );
}

async function upstream(server: Server) {
  await expectOk(
    await call(server, 'PUT', '/service/rest/v1/security/ssrf-protection', {
      enabled: true,
      allowedIPs: ['127.0.0.1'],
      allowedDomains: [],
    }),
    'ssrf',
  );
  await server.setAnonymous(true);
  await rawSite(server, 'p2-site');
  await rawSite(server, 'p2-elsewhere');
  const children = ['simple', `${server.url}/repository/p2-elsewhere/tools`];
  await put(
    server,
    '/repository/p2-site/compositeContent.jar',
    zip([['compositeContent.xml', composite('Content', children)]]),
  );
  await put(
    server,
    '/repository/p2-site/compositeArtifacts.jar',
    zip([['compositeArtifacts.xml', composite('Artifacts', children)]]),
  );
  await put(
    server,
    '/repository/p2-site/p2.index',
    'version=1\nmetadata.repository.factory.order=compositeContent.xml,\\!\nartifact.repository.factory.order=compositeArtifacts.xml,\\!\n',
  );
  await put(
    server,
    '/repository/p2-site/simple/content.jar',
    zip([['content.xml', content([['org.example.core', '1.0.0.v1']])]]),
  );
  await put(
    server,
    '/repository/p2-site/simple/artifacts.jar',
    zip([
      [
        'artifacts.xml',
        artifacts([
          ['osgi.bundle', 'org.example.core', '1.0.0.v1'],
          ['org.eclipse.update.feature', 'org.example.feature', '1.0.0'],
        ]),
      ],
    ]),
  );
  await put(
    server,
    '/repository/p2-site/simple/plugins/org.example.core_1.0.0.v1.jar',
    plugin('org.example.core', '1.0.0.v1', 'Example Core'),
  );
  await put(
    server,
    '/repository/p2-site/simple/features/org.example.feature_1.0.0.jar',
    feature('org.example.feature', 'Example Feature'),
  );
  await put(
    server,
    '/repository/p2-elsewhere/tools/content.jar',
    zip([['content.xml', content([['org.other.tool', '3.1.0']])]]),
  );
  await put(
    server,
    '/repository/p2-elsewhere/tools/artifacts.jar',
    zip([['artifacts.xml', artifacts([['osgi.bundle', 'org.other.tool', '3.1.0']])]]),
  );
  await put(
    server,
    '/repository/p2-elsewhere/tools/plugins/org.other.tool_3.1.0.jar',
    plugin('org.other.tool', '3.1.0', 'Other Tool'),
  );
}

async function readThroughTheProxy(server: Server, proxy: string): Promise<string> {
  const listed = await call(server, 'GET', `/repository/${proxy}/compositeArtifacts.jar`);
  await expectOk(listed, 'composite');
  const elsewhere = `.remote/http/${new URL(server.url).host}/repository/p2-elsewhere/tools`;
  for (const path of [
    'p2.index',
    'compositeContent.jar',
    'simple/content.jar',
    'simple/artifacts.jar',
    'simple/plugins/org.example.core_1.0.0.v1.jar',
    'simple/features/org.example.feature_1.0.0.jar',
    `${elsewhere}/artifacts.jar`,
    `${elsewhere}/plugins/org.other.tool_3.1.0.jar`,
  ]) {
    await expectOk(await call(server, 'GET', `/repository/${proxy}/${path}`), path);
  }
  return elsewhere;
}

async function createProxyThroughTheForm(page: Page, server: Server, name: string) {
  await page.goto('/admin/repository/repositories');
  await page.getByRole('link', { name: 'Create Repository' }).first().click();
  await page.getByRole('button', { name: 'P2', exact: true }).click();
  await page.getByRole('button', { name: /P2 proxy/ }).click();
  await expect(page.getByText(/https:\/\/download\.eclipse\.org\/releases\//)).toBeVisible();
  await page.getByRole('textbox', { name: 'Name', exact: true }).fill(name);
  await page.getByRole('textbox', { name: /Remote Storage/ }).fill(`${server.url}/repository/p2-site/`);
  await page.getByRole('button', { name: 'Continue' }).click();
  await expect(page).toHaveURL(/\/admin\/repository\/repositories$/);
  await expect(page.getByText(`Repository "${name}" created`, { exact: true })).toBeVisible();
}

test('a P2 proxy made through the form serves a composite site and shows its usage', async ({
  page,
  server,
}) => {
  await upstream(server);
  await signInAsAdmin(page, server);
  await createProxyThroughTheForm(page, server, 'eclipse');
  const stored = await (await call(server, 'GET', '/service/rest/v1/repositories/p2/proxy/eclipse')).json();
  expect(stored.proxy.remoteUrl).toBe(`${server.url}/repository/p2-site/`);
  const elsewhere = await readThroughTheProxy(server, 'eclipse');

  await page.goto('/browse/eclipse');
  const tree = page.getByRole('tree');
  await expect(tree.getByText('.remote', { exact: true })).toBeVisible();
  await expect(tree.getByText('compositeContent.jar', { exact: true })).toBeVisible();
  await tree.getByText('simple', { exact: true }).click();
  await tree.getByText('plugins', { exact: true }).click();
  await tree.getByText('org.example.core', { exact: true }).click();
  await tree.getByText('1.0.0.v1', { exact: true }).click();
  await tree.getByText('org.example.core_1.0.0.v1.jar', { exact: true }).click();
  await expect(page).toHaveURL(
    /\/browse\/eclipse\/simple\/plugins\/org\.example\.core\/1\.0\.0\.v1\/org\.example\.core_1\.0\.0\.v1\.jar$/,
  );
  const url = `${server.url}/repository/eclipse/`;
  await expect(page.getByRole('tab', { name: 'Eclipse' })).toHaveAttribute('aria-selected', 'true');
  await expect(page.getByText(`Location: ${url}`)).toBeVisible();
  await page.getByRole('tab', { name: 'Tycho' }).click();
  await expect(page.getByText('<layout>p2</layout>')).toBeVisible();
  await page.getByRole('tab', { name: 'p2 director' }).click();
  await expect(page.getByText('-installIU org.example.core/1.0.0.v1')).toBeVisible();

  await page.goto(`/browse/eclipse/${elsewhere}/plugins/org.other.tool/3.1.0/org.other.tool_3.1.0.jar`);
  await expect(page.getByRole('tab', { name: 'Eclipse' })).toBeVisible();

  await page.goto(`/search/component/${encodeURIComponent('p2:features:org.example.feature')}?version=1.0.0`);
  await expect(page.getByText('-installIU org.example.feature.feature.group/1.0.0')).toBeVisible();

  await page.goto('/search/custom');
  await page.getByRole('combobox', { name: 'Field' }).click();
  await page.getByRole('option', { name: 'P2 Plugin Name' }).click();
  await page.getByRole('combobox', { name: 'Condition' }).click();
  await page.getByRole('option', { name: 'Contains' }).click();
  await page.getByRole('textbox', { name: 'Value' }).fill('Core');
  await page.getByRole('button', { name: 'Search', exact: true }).click();
  await expect(page).toHaveURL(/\/search\/custom\?p2\.pluginName=\*Core\*$/);
  await expect(page.getByText('org.example.core').first()).toBeVisible();

  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/browse/eclipse/simple/plugins/org.example.core/1.0.0.v1/org.example.core_1.0.0.v1.jar');
  await expect(page.getByRole('tab', { name: 'Eclipse' })).toBeVisible();
  await noHorizontalScroll(page);
});

async function shoot(page: Page, name: string) {
  if (DIR) {
    mkdirSync(DIR, { recursive: true });
    await page.waitForTimeout(300);
    await page.screenshot({ path: join(DIR, `${name}.png`) });
  }
}

test('p2 screens', async ({ page, server }) => {
  test.skip(!DIR, 'set ASTRUM_SHOTS_DIR to take screenshots');
  test.setTimeout(240_000);
  await upstream(server);
  await expectOk(
    await call(server, 'POST', '/service/rest/v1/repositories/p2/proxy', {
      name: 'eclipse-shots',
      online: true,
      storage: { blobStoreName: 'default', strictContentTypeValidation: true },
      proxy: { remoteUrl: `${server.url}/repository/p2-site/`, contentMaxAge: 1440, metadataMaxAge: 1440 },
      negativeCache: { enabled: true, timeToLive: 1440 },
      httpClient: { blocked: false, autoBlock: true },
    }),
    'create',
  );
  await readThroughTheProxy(server, 'eclipse-shots');
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
      await shotPage.goto('/admin/repository/repositories/create/p2/proxy');
      await expect(shotPage.getByRole('textbox', { name: 'Name', exact: true })).toBeVisible();
      await shoot(shotPage, `p2-create-proxy-${suffix}`);
      await shotPage.goto(
        '/browse/eclipse-shots/simple/plugins/org.example.core/1.0.0.v1/org.example.core_1.0.0.v1.jar',
      );
      await expect(shotPage.getByRole('tab', { name: 'Eclipse' })).toBeVisible();
      await shoot(shotPage, `p2-browse-plugin-${suffix}`);
      await shotPage.getByRole('tab', { name: 'Tycho' }).click();
      await shoot(shotPage, `p2-browse-tycho-${suffix}`);
      await shotPage.goto(
        `/search/component/${encodeURIComponent('p2:features:org.example.feature')}?version=1.0.0`,
      );
      await expect(shotPage.getByText('-installIU org.example.feature.feature.group/1.0.0')).toBeVisible();
      await shoot(shotPage, `p2-component-${suffix}`);
      await context.close();
    }
  }
});
