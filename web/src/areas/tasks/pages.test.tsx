import { configure, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { fakeServer, jsonResponse, renderWithApp } from '../../test/render';
import CreateTaskPage from './CreateTaskPage';
import type { FormField, Run, Task, Template } from './model';
import TaskPage from './TaskPage';
import TasksPage from './TasksPage';

vi.setConfig({ testTimeout: 30_000 });
configure({ asyncUtilTimeout: 5_000 });

const UI = '/service/rest/internal/ui/tasks';
const V1 = '/service/rest/v1/tasks';

function field(change: Partial<FormField>): FormField {
  return {
    id: 'value',
    type: 'string',
    label: 'Value',
    helpText: null,
    required: false,
    initialValue: null,
    minValue: null,
    maxValue: null,
    storeApi: null,
    storeFilters: null,
    attributes: {},
    ...change,
  };
}

const COMPACT: Template = {
  type: 'blobstore.compact',
  name: 'Admin - Compact blob store',
  properties: {},
  creatable: true,
  formFields: [
    field({
      id: 'blobstoreName',
      type: 'combobox',
      label: 'Blob store',
      required: true,
      storeApi: 'blobstores',
      attributes: { allowAll: true },
    }),
    field({
      id: 'blobsOlderThan',
      type: 'number',
      label: 'Blobs older than',
      minValue: '0',
      initialValue: '1',
    }),
  ],
};

const CLEANUP: Template = {
  type: 'repository.cleanup',
  name: 'Admin - Cleanup repositories using their associated policies',
  properties: {},
  creatable: false,
  singleton: true,
  formFields: [],
};

function run(change: Partial<Run>): Run {
  return {
    id: 1,
    state: 'done',
    trigger: 'manual',
    initiator: 'admin',
    node: 'host:1',
    message: 'freed 2 blobs',
    counts: {},
    cancelRequested: false,
    queuedAt: '2026-10-12T10:00:00Z',
    startedAt: '2026-10-12T10:00:01Z',
    finishedAt: '2026-10-12T10:00:03Z',
    durationMs: 2000,
    ...change,
  };
}

function task(change: Partial<Task>): Task {
  return {
    id: 't1',
    name: 'Compact blob stores',
    type: 'blobstore.compact',
    typeName: 'Admin - Compact blob store',
    known: true,
    enabled: true,
    frequency: {
      schedule: 'cron',
      startDate: null,
      timeZoneOffset: null,
      recurringDays: null,
      cronExpression: '0 0 2 * * ?',
    },
    properties: { blobstoreName: '*', blobsOlderThan: '1' },
    alertEmail: null,
    notificationCondition: 'FAILURE',
    nextRun: '2026-10-13T02:00:00Z',
    state: 'waiting',
    current: null,
    lastRun: run({}),
    createdAt: '2026-10-12T09:00:00Z',
    updatedAt: '2026-10-12T09:00:00Z',
    template: COMPACT,
    ...change,
  };
}

const TASKS = [
  task({}),
  task({
    id: 't2',
    name: 'Cleanup service',
    type: 'repository.cleanup',
    typeName: CLEANUP.name,
    lastRun: null,
    template: CLEANUP,
    properties: {},
  }),
  task({ id: 't3', name: 'Paused', enabled: false, lastRun: run({ state: 'failed', message: 'broken' }) }),
];

afterEach(() => {
  vi.unstubAllGlobals();
});

function serve(handlers: Parameters<typeof fakeServer>[0]) {
  const server = fakeServer({
    [`GET ${UI}`]: () => jsonResponse(TASKS),
    [`GET ${UI}/types`]: () => jsonResponse([COMPACT, CLEANUP]),
    [`GET ${UI}/choices`]: () =>
      jsonResponse({ repositories: [], blobStores: [{ name: 'default', group: false }] }),
    ...handlers,
  });
  vi.stubGlobal('fetch', server.fetch);
  return server;
}

function page(path: string) {
  return renderWithApp(null, {
    path,
    routes: [
      { path: '/admin/system/tasks', element: <TasksPage /> },
      { path: '/admin/system/tasks/create', element: <CreateTaskPage /> },
      { path: '/admin/system/tasks/:id', element: <TaskPage /> },
    ],
  });
}

describe('the task list', () => {
  it('shows status, schedule and result, filters by status and runs a task at once', async () => {
    const server = serve({ [`POST ${V1}/t1/run`]: () => new Response(null, { status: 204 }) });
    page('/admin/system/tasks');
    const row = await screen.findByRole('button', { name: 'Open Compact blob stores' });
    expect(within(row).getByText('OK')).toBeInTheDocument();
    expect(within(row).getByText('Cron')).toBeInTheDocument();
    const paused = screen.getByRole('button', { name: 'Open Paused' });
    expect(within(paused).getByText('Disabled')).toBeInTheDocument();
    expect(within(paused).getByText('Failed')).toBeInTheDocument();
    expect(within(paused).queryByRole('button', { name: 'Run Paused' })).not.toBeInTheDocument();
    await userEvent.click(screen.getByRole('checkbox', { name: /Disabled/ }));
    await waitFor(() =>
      expect(screen.queryByRole('button', { name: 'Open Cleanup service' })).not.toBeInTheDocument(),
    );
    await userEvent.click(screen.getByRole('checkbox', { name: /Disabled/ }));
    const again = await screen.findByRole('button', { name: 'Open Compact blob stores' });
    await userEvent.click(within(again).getByRole('button', { name: 'Run Compact blob stores' }));
    expect(await screen.findByText('Task "Compact blob stores" started')).toBeInTheDocument();
    expect(server.calls.some((call) => call.method === 'POST' && call.path === `${V1}/t1/run`)).toBe(true);
    expect(screen.queryByRole('alertdialog')).not.toBeInTheDocument();
  });

  it('opens a task from its row', async () => {
    serve({ [`GET ${UI}/t1`]: () => jsonResponse(TASKS[0]) });
    page('/admin/system/tasks');
    await userEvent.click(await screen.findByRole('button', { name: 'Open Compact blob stores' }));
    expect(await screen.findByRole('heading', { level: 1, name: 'Compact blob stores' })).toBeInTheDocument();
  });
});

describe('creating a task', () => {
  it('walks through type, settings and schedule and posts the task', async () => {
    const server = serve({
      [`POST ${V1}`]: ({ body }) => jsonResponse({ ...(body as object), id: 't9' }, 201),
      [`GET ${UI}/t9`]: () => jsonResponse(task({ id: 't9', name: 'Weekly compaction' })),
    });
    page('/admin/system/tasks/create');
    const types = await screen.findByRole('table');
    expect(within(types).queryByText(CLEANUP.name)).not.toBeInTheDocument();
    const next = screen.getByRole('button', { name: 'Continue' });
    expect(next).toBeDisabled();
    await userEvent.click(within(types).getByRole('button', { name: COMPACT.name }));
    await userEvent.click(next);
    expect(await screen.findByRole('heading', { name: 'Configure' })).toBeInTheDocument();
    await userEvent.type(screen.getByRole('textbox', { name: /Name/ }), 'Weekly compaction');
    expect(screen.getByRole('button', { name: 'Continue' })).toBeDisabled();
    await userEvent.click(screen.getByRole('combobox', { name: /Blob Store/i }));
    await userEvent.click(await screen.findByRole('option', { name: 'default' }));
    await userEvent.click(screen.getByRole('button', { name: 'Continue' }));
    expect(await screen.findByRole('heading', { name: 'Schedule' })).toBeInTheDocument();
    await userEvent.click(screen.getByRole('combobox', { name: /Schedule/ }));
    await userEvent.click(await screen.findByRole('option', { name: 'Weekly' }));
    await userEvent.click(screen.getByRole('button', { name: 'Create' }));
    expect(await screen.findByText('Select at least one day')).toBeInTheDocument();
    await userEvent.click(screen.getByRole('checkbox', { name: 'Sunday' }));
    await userEvent.click(screen.getByRole('button', { name: 'Create' }));
    await waitFor(() => expect(server.calls.some((call) => call.method === 'POST')).toBe(true));
    expect(server.calls.find((call) => call.method === 'POST')?.body).toMatchObject({
      type: 'blobstore.compact',
      name: 'Weekly compaction',
      enabled: true,
      properties: { blobstoreName: 'default', blobsOlderThan: '1' },
      frequency: { schedule: 'weekly', recurringDays: [1] },
    });
    expect(await screen.findByText('Task "Weekly compaction" created')).toBeInTheDocument();
  });
});

describe('a task page', () => {
  it('opens the settings from the summary and saves only what changed', async () => {
    const server = serve({
      [`GET ${UI}/t1`]: () => jsonResponse(TASKS[0]),
      [`PUT ${V1}/t1`]: ({ body }) => jsonResponse(body),
    });
    page('/admin/system/tasks/t1');
    expect(await screen.findByRole('heading', { name: 'Compact blob stores' })).toBeInTheDocument();
    expect(screen.getByText('Admin - Compact blob store')).toBeInTheDocument();
    await userEvent.click(screen.getByRole('button', { name: 'Edit' }));
    expect(await screen.findByRole('heading', { name: 'Basic Settings' })).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'Edit' })).not.toBeInTheDocument();
    expect(screen.getByRole('textbox', { name: /^Name/ })).toBeDisabled();
    const save = screen.getByRole('button', { name: 'Save' });
    expect(save).toBeDisabled();
    await userEvent.type(screen.getByRole('textbox', { name: /Notification Email/ }), 'ops@example.org');
    expect(screen.getByText('Unsaved changes')).toBeInTheDocument();
    await userEvent.click(save);
    await waitFor(() => expect(server.calls.some((call) => call.method === 'PUT')).toBe(true));
    expect(server.calls.find((call) => call.method === 'PUT')?.body).toMatchObject({
      name: 'Compact blob stores',
      alertEmail: 'ops@example.org',
      frequency: { schedule: 'cron', cronExpression: '0 0 2 * * ?' },
    });
    expect(await screen.findByText('Task "Compact blob stores" updated')).toBeInTheDocument();
  });

  it('lists the runs and shows the log of one', async () => {
    serve({
      [`GET ${UI}/t1`]: () => jsonResponse(TASKS[0]),
      [`GET ${UI}/t1/runs`]: () => jsonResponse([run({ id: 7 })]),
      [`GET ${UI}/t1/runs/7/log`]: () =>
        jsonResponse({
          run: run({ id: 7 }),
          finished: true,
          more: false,
          lines: [{ line: 1, at: '2026-10-12T10:00:02Z', level: 'INFO', message: 'compacting default' }],
        }),
    });
    page('/admin/system/tasks/t1?tab=history');
    expect(await screen.findByText('Execution History')).toBeInTheDocument();
    expect(await screen.findByText('freed 2 blobs')).toBeInTheDocument();
    await userEvent.click(screen.getByRole('button', { name: 'Show log' }));
    expect(await screen.findByRole('log')).toHaveTextContent('compacting default');
  });
});
