import { createHash, randomUUID } from 'node:crypto';
import { existsSync, mkdirSync, mkdtempSync, rmSync, utimesSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import type { Page } from '@playwright/test';
import { call, ensure, hosted, put } from './content';
import { expect, type Server, signIn, test } from './fixtures';

test.describe.configure({ timeout: 120_000 });

const SHOTS = process.env.ASTRUM_SHOTS_DIR;
const SIZES = [
  { name: '1440', width: 1440, height: 900 },
  { name: '390', width: 390, height: 844 },
] as const;
const THEMES = ['dark', 'light'] as const;

interface UiTask {
  id: string;
  name: string;
  type: string;
  lastRun: { state: string; message: string } | null;
}

async function signInAsAdmin(page: Page, password: string) {
  await page.goto('/login');
  await signIn(page, password);
  await expect(page.getByRole('button', { name: /Account menu of/ })).toBeVisible();
}

async function tasks(server: Server): Promise<UiTask[]> {
  const response = await call(server, 'GET', '/service/rest/internal/ui/tasks');
  await ensure(response, 'list tasks');
  return (await response.json()) as UiTask[];
}

async function taskOfType(server: Server, type: string): Promise<UiTask> {
  const found = (await tasks(server)).find((task) => task.type === type);
  expect(found, `a task of type ${type}`).toBeTruthy();
  return found as UiTask;
}

async function runFromItsPage(page: Page, server: Server, task: UiTask): Promise<UiTask> {
  const before = task.lastRun;
  await page.goto(`/admin/system/tasks/${task.id}`);
  await expect(page.getByRole('heading', { level: 1, name: task.name })).toBeVisible();
  await page.getByRole('button', { name: 'Run Now' }).click();
  await page.getByRole('alertdialog').getByRole('button', { name: 'Run' }).click();
  await expect(page.getByText(`Task "${task.name}" started`, { exact: true })).toBeVisible();
  let finished: UiTask = task;
  await expect
    .poll(
      async () => {
        finished = (await tasks(server)).find((candidate) => candidate.id === task.id) as UiTask;
        return finished.lastRun !== null && finished.lastRun !== before ? finished.lastRun.state : 'waiting';
      },
      { timeout: 30_000, intervals: [500] },
    )
    .toBe('done');
  return finished;
}

async function status(server: Server, path: string): Promise<number> {
  return (await call(server, 'GET', path)).status;
}

function sha256(data: Buffer): string {
  return `sha256:${createHash('sha256').update(data).digest('hex')}`;
}

async function pushBlob(server: Server, prefix: string, data: Buffer): Promise<string> {
  const digest = sha256(data);
  await ensure(
    await call(server, 'POST', `${prefix}/blobs/uploads/?digest=${digest}`, data, {
      'Content-Type': 'application/octet-stream',
    }),
    `blob ${digest}`,
  );
  return digest;
}

test('a cleanup policy is previewed, applied to a repository and run by the cleanup task', async ({
  page,
  server,
}) => {
  await hosted(server, 'raw', 'cleanup-raw', { raw: { contentDisposition: 'ATTACHMENT' } });
  await put(server, '/repository/cleanup-raw/nightly/build-1.txt', 'first nightly build');
  await put(server, '/repository/cleanup-raw/nightly/build-2.txt', 'second nightly build');
  await put(server, '/repository/cleanup-raw/docs/keep.txt', 'documentation stays');
  await signInAsAdmin(page, server.adminPassword);

  await page.goto('/admin/repository/cleanup-policies');
  await expect(page.getByRole('heading', { level: 1, name: 'Cleanup Policies' })).toBeVisible();
  await page.getByRole('link', { name: 'Create Cleanup Policy' }).click();
  await page.getByRole('textbox', { name: /^Name/ }).fill('nightly-builds');
  await page.getByRole('combobox', { name: /^Format/ }).click();
  await page.getByRole('option', { name: 'Raw', exact: true }).click();
  await page.getByRole('textbox', { name: /^Description/ }).fill('Nightly build files');
  await page.getByRole('checkbox', { name: 'Asset Name Pattern' }).click();
  await page.getByRole('textbox', { name: 'Asset Name Pattern' }).fill('nightly/.*');
  await page.getByRole('combobox', { name: 'Preview Repository' }).click();
  await page.getByRole('option', { name: 'cleanup-raw', exact: true }).click();
  const preview = page.getByRole('table');
  await expect(preview.getByText('/nightly/build-1.txt')).toBeVisible();
  await expect(preview.getByText('/nightly/build-2.txt')).toBeVisible();
  await expect(preview.getByText('/docs/keep.txt')).toHaveCount(0);
  await page.getByRole('button', { name: 'Calculate' }).click();
  await expect(page.getByText(/If it ran now, this policy would delete from/)).toBeVisible();
  const csv = await page.getByRole('link', { name: /Download CSV Report/ }).getAttribute('href');
  const report = await call(server, 'GET', csv ?? '');
  await ensure(report, 'csv');
  expect(await report.text()).toContain('build-2.txt');
  await page.getByRole('button', { name: 'Create', exact: true }).click();
  await expect(
    page.getByText('Cleanup policy "nightly-builds" created successfully', { exact: true }),
  ).toBeVisible();

  await page.goto('/admin/repository/repositories/cleanup-raw');
  const available = page.getByRole('listbox', { name: 'Available Policies' });
  await available.getByRole('option', { name: 'nightly-builds' }).dblclick();
  await expect(page.getByRole('listbox', { name: 'Applied Policies' }).getByRole('option')).toHaveText([
    'nightly-builds',
  ]);
  await page.getByRole('button', { name: 'Save Changes', exact: true }).click();
  await expect
    .poll(async () => {
      const response = await call(server, 'GET', '/service/rest/v1/repositories/raw/hosted/cleanup-raw');
      return ((await response.json()) as { cleanup?: { policyNames?: string[] } }).cleanup?.policyNames;
    })
    .toEqual(['nightly-builds']);

  const cleanup = await taskOfType(server, 'repository.cleanup');
  const done = await runFromItsPage(page, server, cleanup);
  expect(done.lastRun?.message).toContain('deleted 2 components');
  await page.getByRole('tab', { name: 'History' }).click();
  await expect(page.getByText('Execution History')).toBeVisible();
  await page.getByRole('button', { name: 'Show log' }).first().click();
  await expect(page.getByRole('log')).toContainText('nightly');

  expect(await status(server, '/repository/cleanup-raw/nightly/build-1.txt')).toBe(404);
  expect(await status(server, '/repository/cleanup-raw/nightly/build-2.txt')).toBe(404);
  expect(await status(server, '/repository/cleanup-raw/docs/keep.txt')).toBe(200);
});

test('a compaction task created in the wizard removes old blobs nothing refers to', async ({
  page,
  server,
}) => {
  const root = mkdtempSync(join(tmpdir(), 'astrum-e2e-compact-'));
  try {
    await ensure(
      await call(server, 'POST', '/service/rest/v1/blobstores/file', { name: 'compact-me', path: root }),
      'create store',
    );
    await hosted(server, 'raw', 'compact-raw', {
      storage: { blobStoreName: 'compact-me', strictContentTypeValidation: false, writePolicy: 'ALLOW' },
      raw: { contentDisposition: 'ATTACHMENT' },
    });
    await put(server, '/repository/compact-raw/kept.txt', 'still referenced');
    const orphan = randomUUID();
    const dir = join(root, 'content', orphan.slice(0, 2), orphan.slice(2, 4));
    mkdirSync(dir, { recursive: true });
    const orphanPath = join(dir, orphan);
    writeFileSync(orphanPath, 'nothing refers to me');
    const old = new Date(Date.now() - 3 * 24 * 60 * 60 * 1000);
    utimesSync(orphanPath, old, old);

    await signInAsAdmin(page, server.adminPassword);
    await page.goto('/admin/system/tasks');
    await page.getByRole('link', { name: 'Create Task' }).click();
    await page.getByRole('button', { name: 'Admin - Compact blob store' }).click();
    await page.getByRole('button', { name: 'Continue' }).click();
    await expect(page.getByRole('heading', { name: 'Configure' })).toBeVisible();
    await page.getByRole('textbox', { name: /^Name/ }).fill('Compact compact-me');
    await page.getByRole('combobox', { name: /Blob Store/i }).click();
    await page.getByRole('option', { name: 'compact-me', exact: true }).click();
    await page.getByRole('button', { name: 'Continue' }).click();
    await expect(page.getByRole('heading', { name: 'Schedule' })).toBeVisible();
    await page.getByRole('button', { name: 'Create', exact: true }).click();
    await expect(page.getByText('Task "Compact compact-me" created', { exact: true })).toBeVisible();
    await expect(page.getByRole('heading', { level: 1, name: 'Compact compact-me' })).toBeVisible();

    const created = (await tasks(server)).find((task) => task.name === 'Compact compact-me') as UiTask;
    const done = await runFromItsPage(page, server, created);
    expect(done.lastRun?.message).toMatch(/deleted 1 unused blob\b/);
    expect(existsSync(orphanPath)).toBe(false);
    expect(await status(server, '/repository/compact-raw/kept.txt')).toBe(200);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test('docker garbage collection removes an image once its tag is gone', async ({ page, server }) => {
  await hosted(server, 'docker', 'gc-docker', {
    docker: { v1Enabled: false, forceBasicAuth: true, pathEnabled: true },
  });
  const prefix = '/v2/gc-docker/team/app';
  const config = Buffer.from('{"os":"linux","architecture":"amd64"}');
  const layer = Buffer.from('a layer only this image uses');
  const configDigest = await pushBlob(server, prefix, config);
  const layerDigest = await pushBlob(server, prefix, layer);
  const manifest = JSON.stringify({
    schemaVersion: 2,
    mediaType: 'application/vnd.oci.image.manifest.v1+json',
    config: {
      mediaType: 'application/vnd.oci.image.config.v1+json',
      digest: configDigest,
      size: config.length,
    },
    layers: [
      { mediaType: 'application/vnd.oci.image.layer.v1.tar+gzip', digest: layerDigest, size: layer.length },
    ],
  });
  await ensure(
    await call(server, 'PUT', `${prefix}/manifests/1.0`, Buffer.from(manifest), {
      'Content-Type': 'application/vnd.oci.image.manifest.v1+json',
    }),
    'manifest',
  );
  const manifestDigest = sha256(Buffer.from(manifest));
  await ensure(await call(server, 'DELETE', `${prefix}/manifests/1.0`), 'delete tag');
  expect(await status(server, `${prefix}/manifests/${manifestDigest}`)).toBe(200);

  const gc = await taskOfType(server, 'repository.docker.gc');
  await signInAsAdmin(page, server.adminPassword);
  await page.goto(`/admin/system/tasks/${gc.id}?tab=settings`);
  const offset = page
    .getByRole('spinbutton', { name: /Deploy Offset/i })
    .or(page.getByRole('textbox', { name: /Deploy Offset/i }));
  await offset.fill('0');
  await expect(page.getByText('Unsaved changes')).toBeVisible();
  await page.getByRole('button', { name: 'Save', exact: true }).click();
  await expect(page.getByText(`Task "${gc.name}" updated`, { exact: true })).toBeVisible();

  const done = await runFromItsPage(page, server, gc);
  expect(done.lastRun?.state).toBe('done');
  expect(await status(server, `${prefix}/manifests/${manifestDigest}`)).toBe(404);
  expect(await status(server, `${prefix}/blobs/${layerDigest}`)).toBe(404);
});

test('a task is rescheduled, run from the list and deleted', async ({ page, server }) => {
  const created = await call(server, 'POST', '/service/rest/v1/tasks', {
    type: 'blobstore.metrics.reconcile',
    name: 'Recount default',
    enabled: true,
    alertEmail: null,
    notificationCondition: 'FAILURE',
    frequency: { schedule: 'manual' },
    properties: { blobstoreName: 'default' },
  });
  await ensure(created, 'create task');
  const { id } = (await created.json()) as { id: string };
  await signInAsAdmin(page, server.adminPassword);

  await page.goto(`/admin/system/tasks/${id}?tab=schedule`);
  await page.getByRole('combobox', { name: /^Schedule/ }).click();
  await page.getByRole('option', { name: 'Daily', exact: true }).click();
  await page.getByRole('button', { name: 'Save', exact: true }).click();
  await expect(page.getByText('Task "Recount default" updated', { exact: true })).toBeVisible();
  await expect
    .poll(async () => {
      const response = await call(server, 'GET', `/service/rest/v1/tasks/${id}`);
      const task = (await response.json()) as { schedule: string; nextRun: string | null };
      return task.schedule === 'daily' && task.nextRun !== null;
    })
    .toBe(true);

  await page.goto('/admin/system/tasks');
  const row = page.getByRole('button', { name: 'Open Recount default' });
  await row.hover();
  await row.getByRole('button', { name: 'Run Recount default' }).click();
  await expect(page.getByText('Task "Recount default" started', { exact: true })).toBeVisible();
  await expect
    .poll(async () => (await tasks(server)).find((task) => task.id === id)?.lastRun?.state ?? 'none', {
      timeout: 30_000,
    })
    .toBe('done');

  await row.hover();
  await row.getByRole('button', { name: 'Delete Recount default' }).click();
  const dialog = page.getByRole('alertdialog');
  await expect(dialog.getByRole('button', { name: 'Delete', exact: true })).toBeDisabled();
  await dialog.getByRole('textbox').fill('Delete');
  await dialog.getByRole('button', { name: 'Delete', exact: true }).click();
  await expect(page.getByText('Task "Recount default" deleted', { exact: true })).toBeVisible();
  await expect(page.getByRole('button', { name: 'Open Recount default' })).toHaveCount(0);
  expect(await status(server, `/service/rest/v1/tasks/${id}`)).toBe(404);
});

test('a cleanup policy is changed and deleted, and its repositories let it go', async ({ page, server }) => {
  await hosted(server, 'raw', 'policy-raw', { raw: { contentDisposition: 'ATTACHMENT' } });
  await ensure(
    await call(server, 'POST', '/service/rest/internal/cleanup-policies', {
      name: 'stale-raw',
      format: 'raw',
      notes: '',
      criteriaLastBlobUpdated: 30,
      criteriaLastDownloaded: null,
      criteriaReleaseType: null,
      criteriaAssetRegex: null,
    }),
    'policy',
  );
  const repository = await call(server, 'GET', '/service/rest/v1/repositories/raw/hosted/policy-raw');
  await ensure(
    await call(server, 'PUT', '/service/rest/v1/repositories/raw/hosted/policy-raw', {
      ...((await repository.json()) as object),
      cleanup: { policyNames: ['stale-raw'] },
    }),
    'assign policy',
  );
  await signInAsAdmin(page, server.adminPassword);

  await page.goto('/admin/repository/cleanup-policies');
  await page.getByRole('button', { name: 'Open stale-raw' }).click();
  await expect(page.getByRole('heading', { level: 1, name: 'Edit stale-raw' })).toBeVisible();
  await page.getByRole('textbox', { name: /^Description/ }).fill('Raw files untouched for a month');
  await page.getByRole('button', { name: 'Save', exact: true }).click();
  await expect(
    page.getByText('Cleanup policy "stale-raw" updated successfully', { exact: true }),
  ).toBeVisible();
  await page.goto('/admin/repository/cleanup-policies');
  await expect(page.getByText('Raw files untouched for a month')).toBeVisible();

  await page.getByRole('button', { name: 'Open stale-raw' }).click();
  await page.getByRole('button', { name: 'Delete', exact: true }).click();
  const dialog = page.getByRole('alertdialog');
  await expect(dialog).toContainText('1 repository');
  await dialog.getByRole('button', { name: 'Delete', exact: true }).click();
  await expect(
    page.getByText('Cleanup policy "stale-raw" deleted successfully', { exact: true }),
  ).toBeVisible();
  await expect(page.getByRole('heading', { level: 1, name: 'Cleanup Policies' })).toBeVisible();
  await expect(page.getByRole('button', { name: 'Open stale-raw' })).toHaveCount(0);
  const after = await call(server, 'GET', '/service/rest/v1/repositories/raw/hosted/policy-raw');
  expect(
    ((await after.json()) as { cleanup?: { policyNames?: string[] } }).cleanup?.policyNames ?? [],
  ).toEqual([]);
});

test.describe('screenshots', () => {
  test.skip(!SHOTS, 'set ASTRUM_SHOTS_DIR to take screenshots');

  for (const size of SIZES) {
    for (const theme of THEMES) {
      test(`task and cleanup screens at ${size.name} in ${theme}`, async ({ page, server }) => {
        const shoot = async (name: string) => {
          if (SHOTS) {
            mkdirSync(SHOTS, { recursive: true });
            await page.waitForTimeout(300);
            await page.screenshot({ path: join(SHOTS, `${name}-${size.name}-${theme}.png`) });
          }
        };
        await hosted(server, 'raw', 'shots-raw', { raw: { contentDisposition: 'ATTACHMENT' } });
        await put(server, '/repository/shots-raw/nightly/build-1.txt', 'first');
        const existing = await call(server, 'GET', '/service/rest/internal/cleanup-policies/shots-nightly');
        if (!existing.ok) {
          await ensure(
            await call(server, 'POST', '/service/rest/internal/cleanup-policies', {
              name: 'shots-nightly',
              format: 'raw',
              notes: 'Nightly build files',
              criteriaLastBlobUpdated: null,
              criteriaLastDownloaded: null,
              criteriaReleaseType: null,
              criteriaAssetRegex: 'nightly/.*',
            }),
            'policy',
          );
        }
        await page.setViewportSize({ width: size.width, height: size.height });
        await page.addInitScript(
          (value) => window.localStorage.setItem('astrum-theme-preference', value),
          theme,
        );
        await signInAsAdmin(page, server.adminPassword);
        await page.goto('/admin/system/tasks');
        await expect(page.getByRole('heading', { level: 1, name: 'Tasks' })).toBeVisible();
        await expect(page.getByText('Compact blob stores').first()).toBeVisible();
        await shoot('tasks');
        await page.goto('/admin/system/tasks/create');
        await page.getByRole('button', { name: 'Admin - Compact blob store' }).click();
        await shoot('task-create-type');
        await page.getByRole('button', { name: 'Continue' }).click();
        await expect(page.getByRole('heading', { name: 'Configure' })).toBeVisible();
        await shoot('task-create-settings');
        const compact = await taskOfType(server, 'blobstore.compact');
        await page.goto(`/admin/system/tasks/${compact.id}`);
        await expect(page.getByRole('heading', { level: 1, name: compact.name })).toBeVisible();
        await shoot('task-summary');
        await page.getByRole('tab', { name: 'Settings' }).click();
        await shoot('task-settings');
        await page.getByRole('tab', { name: 'Schedule' }).click();
        await shoot('task-schedule');
        await page.getByRole('tab', { name: 'History' }).click();
        await shoot('task-history');
        await page.goto('/admin/repository/cleanup-policies');
        await expect(page.getByRole('heading', { level: 1, name: 'Cleanup Policies' })).toBeVisible();
        await shoot('cleanup-policies');
        await page.goto('/admin/repository/cleanup-policies/shots-nightly');
        await page.getByRole('combobox', { name: 'Preview Repository' }).click();
        await page.getByRole('option', { name: 'shots-raw', exact: true }).click();
        await expect(page.getByRole('table').getByText('/nightly/build-1.txt')).toBeVisible();
        await shoot('cleanup-policy');
        await page.getByRole('button', { name: 'Calculate' }).scrollIntoViewIfNeeded();
        await page.getByRole('button', { name: 'Calculate' }).click();
        await expect(page.getByText('Total size')).toBeVisible();
        await shoot('cleanup-preview');
        await page.goto('/admin/repository/repositories/shots-raw');
        await page.getByRole('listbox', { name: 'Available Policies' }).scrollIntoViewIfNeeded();
        await shoot('repository-cleanup');
        await page.goto('/settings');
        await shoot('settings-hub');
      });
    }
  }
});
