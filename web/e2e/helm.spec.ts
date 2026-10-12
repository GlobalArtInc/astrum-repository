import { mkdirSync } from 'node:fs';
import { join } from 'node:path';
import { gzipSync } from 'node:zlib';
import type { Page } from '@playwright/test';
import { call, tar } from './content';
import { expect, noHorizontalScroll, type Server, signIn, test } from './fixtures';

test.describe.configure({ timeout: 120_000 });

const FILE = /^File\*?$/;
const DIR = process.env.ASTRUM_SHOTS_DIR;

function chart(name: string, version: string, description: string): Buffer {
  const metadata = [
    'apiVersion: v2',
    `name: ${name}`,
    `description: ${description}`,
    'type: application',
    `version: ${version}`,
    'appVersion: "2.4.1"',
    'keywords:',
    '  - web',
    '  - e2e',
  ].join('\n');
  return gzipSync(
    tar([
      [`${name}/Chart.yaml`, `${metadata}\n`],
      [`${name}/values.yaml`, 'replicaCount: 1\n'],
      [
        `${name}/templates/configmap.yaml`,
        'apiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: {{ .Release.Name }}\n',
      ],
    ]),
  );
}

async function expectOk(response: Response, what: string) {
  expect(response.ok, `${what}: ${response.status} ${await response.text()}`).toBe(true);
}

async function signInAsAdmin(page: Page, server: Server) {
  await page.goto('/login');
  await signIn(page, server.adminPassword);
  await expect(page.getByRole('button', { name: /Account menu of/ })).toBeVisible();
}

async function pick(page: Page, combobox: string, option: string) {
  await page.getByRole('combobox', { name: combobox }).click();
  await page.getByRole('option', { name: option, exact: true }).click();
}

async function createThroughForm(page: Page, label: string, name: string, fill?: () => Promise<void>) {
  await page.goto('/admin/repository/repositories');
  await page.getByRole('link', { name: 'Create a repository' }).first().click();
  await page.getByRole('link', { name: label }).click();
  await page.getByRole('textbox', { name: 'Name', exact: true }).fill(name);
  await fill?.();
  await page.getByRole('button', { name: 'Create repository' }).click();
  await expect(page).toHaveURL(new RegExp(`/admin/repository/repositories/${name}$`));
  await expect(page.getByText(`Repository ${name} created`, { exact: true })).toBeVisible();
}

async function allowLoopback(server: Server) {
  await expectOk(
    await call(server, 'PUT', '/service/rest/v1/security/ssrf-protection', {
      enabled: true,
      allowedIPs: ['127.0.0.1'],
      allowedDomains: [],
    }),
    'ssrf',
  );
}

test('helm repositories are made through the forms and serve one index', async ({ page, server }) => {
  await allowLoopback(server);
  await server.setAnonymous(true);
  await signInAsAdmin(page, server);
  await createThroughForm(page, 'Create a hosted Helm repository', 'charts');
  await createThroughForm(page, 'Create a Helm proxy repository', 'charts-mirror', async () => {
    await page
      .getByRole('textbox', { name: 'Remote storage URL', exact: true })
      .fill(`${server.url}/repository/charts/`);
  });
  await createThroughForm(page, 'Create a Helm group repository', 'charts-all', async () => {
    await pick(page, 'Add a repository…', 'charts');
    await pick(page, 'Add a repository…', 'charts-mirror');
  });
  const group = await call(server, 'GET', '/service/rest/v1/repositories/helm/group/charts-all');
  expect((await group.json()).group.memberNames).toEqual(['charts', 'charts-mirror']);

  await expectOk(
    await call(server, 'PUT', '/repository/charts/web-1.0.0.tgz', chart('web', '1.0.0', 'A web chart'), {
      'Content-Type': 'application/gzip',
    }),
    'put chart',
  );
  for (const repository of ['charts', 'charts-mirror', 'charts-all']) {
    const index = await (await call(server, 'GET', `/repository/${repository}/index.yaml`)).text();
    expect(index, repository).toContain('- web-1.0.0.tgz');
    expect(index, repository).toContain('description: A web chart');
  }
  const mirrored = await call(server, 'GET', '/repository/charts-all/web-1.0.0.tgz');
  expect(mirrored.status).toBe(200);
});

test('a chart goes up through the upload page and shows in browse with its usage', async ({
  page,
  server,
}) => {
  await expectOk(
    await call(server, 'POST', '/service/rest/v1/repositories/helm/hosted', {
      name: 'charts-upload',
      online: true,
      storage: { blobStoreName: 'default', strictContentTypeValidation: true, writePolicy: 'ALLOW_ONCE' },
    }),
    'create',
  );
  await signInAsAdmin(page, server);
  await page.goto('/upload/charts-upload');
  await expect(page.getByRole('heading', { level: 1, name: 'Upload to charts-upload' })).toBeVisible();
  await page.getByLabel(FILE).setInputFiles({
    name: 'demo-0.5.0.tgz',
    mimeType: 'application/gzip',
    buffer: chart('demo', '0.5.0', 'A chart from the upload page'),
  });
  await page.getByRole('button', { name: 'Upload', exact: true }).click();
  await expect(page.getByRole('link', { name: 'Open demo 0.5.0' })).toBeVisible();
  const index = await (await call(server, 'GET', '/repository/charts-upload/index.yaml')).text();
  expect(index).toContain('description: A chart from the upload page');

  await page.goto('/upload/charts-upload');
  await page.getByLabel(FILE).setInputFiles({
    name: 'demo-0.5.0.tgz',
    mimeType: 'application/gzip',
    buffer: chart('demo', '0.5.0', 'Again'),
  });
  await page.getByRole('button', { name: 'Upload', exact: true }).click();
  await expect(page.getByText(/redeploy is not allowed/)).toBeVisible();

  await page.goto('/browse/charts-upload');
  await page.getByRole('tree').getByText('demo-0.5.0.tgz').click();
  await expect(page).toHaveURL(/\/browse\/charts-upload\/demo-0\.5\.0\.tgz$/);
  const repositoryUrl = `${server.url}/repository/charts-upload/`;
  await expect(page.getByRole('tab', { name: 'helm repo add' })).toHaveAttribute('aria-selected', 'true');
  await expect(page.getByText('Register the repository once, then install charts from it')).toBeVisible();
  await expect(page.getByText(`helm repo add charts-upload ${repositoryUrl}`)).toBeVisible();
  await page.getByRole('tab', { name: 'helm install' }).click();
  await expect(page.getByText('helm install demo charts-upload/demo --version 0.5.0')).toBeVisible();
  await page.getByRole('tab', { name: 'Chart.yaml' }).click();
  await expect(page.getByText(`repository: ${repositoryUrl}`)).toBeVisible();
  await page.getByRole('tab', { name: 'Upload' }).click();
  await expect(
    page.getByText(`curl -u <user> --upload-file demo-0.5.0.tgz ${repositoryUrl}demo-0.5.0.tgz`),
  ).toBeVisible();
  await expect(page.getByText('Uploading needs a user with write access to this repository')).toBeVisible();

  await page.getByRole('link', { name: 'Component page' }).click();
  await expect(page).toHaveURL(/\/search\/component\/helm%3A%3Ademo\?version=0\.5\.0/);
  await expect(page.getByText('A chart from the upload page').first()).toBeVisible();
  await page.getByRole('tab', { name: 'Usage' }).click();
  await expect(page.getByText(`helm repo add charts-upload ${repositoryUrl}`)).toBeVisible();
  await page.getByRole('tab', { name: 'helm install' }).click();
  await expect(page.getByText('helm install demo charts-upload/demo --version 0.5.0')).toBeVisible();

  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/browse/charts-upload/demo-0.5.0.tgz');
  await expect(page.getByRole('tab', { name: 'helm repo add' })).toBeVisible();
  await noHorizontalScroll(page);
});

test('the criteria search finds charts by their metadata', async ({ page, server }) => {
  await expectOk(
    await call(server, 'POST', '/service/rest/v1/repositories/helm/hosted', {
      name: 'charts-search',
      online: true,
      storage: { blobStoreName: 'default', strictContentTypeValidation: true, writePolicy: 'ALLOW' },
    }),
    'create',
  );
  const charts: [string, string][] = [
    ['gateway', 'An API gateway'],
    ['ledger', 'A ledger service'],
  ];
  for (const [name, description] of charts) {
    await expectOk(
      await call(
        server,
        'PUT',
        `/repository/charts-search/${name}-1.0.0.tgz`,
        chart(name, '1.0.0', description),
      ),
      name,
    );
  }
  await signInAsAdmin(page, server);
  await page.goto('/search/custom');
  await page.getByRole('combobox', { name: 'Criterion' }).click();
  await page.getByRole('option', { name: 'Helm description' }).click();
  await page.getByRole('combobox', { name: 'Condition' }).click();
  await page.getByRole('option', { name: 'contains' }).click();
  await page.getByLabel('Value of Helm description').fill('gateway');
  await page.getByRole('button', { name: 'Search', exact: true }).click();
  await expect(page).toHaveURL(/\/search\/custom\?helm\.description=\*gateway\*$/);
  await expect(page.getByRole('table').getByRole('rowheader')).toHaveCount(1);
  await expect(page.getByRole('table').getByRole('rowheader').first()).toContainText('gateway');
  await page.goto('/search/custom?helm.appVersion=2.4.1&repository=charts-search');
  await expect(page.getByRole('table').getByRole('rowheader')).toHaveCount(2);
});

async function shoot(page: Page, name: string) {
  if (DIR) {
    mkdirSync(DIR, { recursive: true });
    await page.waitForTimeout(300);
    await page.screenshot({ path: join(DIR, `${name}.png`) });
  }
}

test('helm screens', async ({ page, server }) => {
  test.skip(!DIR, 'set ASTRUM_SHOTS_DIR to take screenshots');
  test.setTimeout(240_000);
  await expectOk(
    await call(server, 'POST', '/service/rest/v1/repositories/helm/hosted', {
      name: 'charts-shots',
      online: true,
      storage: { blobStoreName: 'default', strictContentTypeValidation: true, writePolicy: 'ALLOW' },
    }),
    'create',
  );
  await expectOk(
    await call(
      server,
      'PUT',
      '/repository/charts-shots/demo-0.5.0.tgz',
      chart('demo', '0.5.0', 'Demo chart'),
    ),
    'put',
  );
  for (const size of [
    { name: '1440', width: 1440, height: 900 },
    { name: '390', width: 390, height: 844 },
  ]) {
    for (const theme of ['dark', 'light']) {
      const context = await page
        .context()
        .browser()
        ?.newContext({
          baseURL: server.url,
          viewport: { width: size.width, height: size.height },
        });
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
      await shotPage.goto('/admin/repository/repositories/create/helm/hosted');
      await expect(shotPage.getByRole('textbox', { name: 'Name', exact: true })).toBeVisible();
      await shoot(shotPage, `create-hosted-${suffix}`);
      await shotPage.goto('/admin/repository/repositories/create/helm/group');
      await expect(shotPage.getByRole('textbox', { name: 'Name', exact: true })).toBeVisible();
      await shoot(shotPage, `create-group-${suffix}`);
      await shotPage.goto('/upload/charts-shots');
      await expect(shotPage.getByLabel(FILE)).toBeAttached();
      await shoot(shotPage, `upload-${suffix}`);
      await shotPage.goto('/browse/charts-shots/demo-0.5.0.tgz');
      await expect(shotPage.getByRole('tab', { name: 'helm repo add' })).toBeVisible();
      await shoot(shotPage, `browse-chart-${suffix}`);
      await shotPage.getByRole('tab', { name: 'Chart.yaml' }).click();
      await shoot(shotPage, `browse-chart-dependency-${suffix}`);
      await shotPage.goto(`/search/component/${encodeURIComponent('helm::demo')}?version=0.5.0&tab=usage`);
      await expect(shotPage.getByRole('tab', { name: 'helm install' })).toBeVisible();
      await shoot(shotPage, `component-usage-${suffix}`);
      await shotPage.goto('/search/custom?helm.keywords=*web*');
      await expect(shotPage.getByRole('link', { name: 'demo', exact: true }).first()).toBeVisible();
      await shoot(shotPage, `criteria-${suffix}`);
      await context.close();
    }
  }
});
