import { configure, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import type { UiState } from '../../../api';
import { ADMIN_STATE, fakeServer, jsonResponse, renderWithApp } from '../../../test/render';
import type { UiBlobStore } from '../api';
import BlobStorePage from './BlobStorePage';
import BlobStoresPage from './BlobStoresPage';
import CreateBlobStorePage from './CreateBlobStorePage';

vi.setConfig({ testTimeout: 30_000 });
configure({ asyncUtilTimeout: 5_000 });

const LIST = '/service/rest/internal/ui/blobstores';

function store(name: string, extra: Partial<UiBlobStore> = {}): UiBlobStore {
  return {
    name,
    type: 'File',
    path: name,
    unavailable: false,
    blobCount: 0,
    totalSize: 0,
    availableSpace: 10 * 1024 * 1024 * 1024,
    softQuota: null,
    quotaStatus: null,
    repositories: [],
    repositoryCount: 0,
    groups: [],
    deletable: true,
    ...extra,
  };
}

const STORES: UiBlobStore[] = [
  store('default', {
    blobCount: 12,
    totalSize: 4096,
    repositories: ['raw-files'],
    repositoryCount: 1,
    deletable: false,
  }),
  store('one', { groups: ['pool'], deletable: false, blobCount: 3 }),
  store('two', { groups: ['pool'], deletable: false }),
  store('spare'),
  store('pool', {
    type: 'Group',
    members: ['one', 'two'],
    fillPolicy: 'writeToFirst',
    softQuota: { type: 'spaceUsedQuota', limit: 1024 },
    quotaStatus: { violation: true, message: 'Blob store pool uses 3 KB of its 1 KB limit' },
  }),
];

afterEach(() => {
  vi.unstubAllGlobals();
});

function serve(handlers: Parameters<typeof fakeServer>[0]) {
  const server = fakeServer({ [`GET ${LIST}`]: () => jsonResponse(STORES), ...handlers });
  vi.stubGlobal('fetch', server.fetch);
  return server;
}

function page(path: string, state: UiState = ADMIN_STATE) {
  return renderWithApp(null, {
    state,
    path,
    routes: [
      { path: '/admin/repository/blobstores', element: <BlobStoresPage /> },
      { path: '/admin/repository/blobstores/create/:type', element: <CreateBlobStorePage /> },
      { path: '/admin/repository/blobstores/:type/:name', element: <BlobStorePage /> },
    ],
  });
}

describe('the blob store list', () => {
  it('shows the old columns and opens a store from its row', async () => {
    serve({
      'GET /service/rest/v1/blobstores/group/pool': () =>
        jsonResponse({ name: 'pool', softQuota: null, members: ['one'], fillPolicy: 'writeToFirst' }),
    });
    page('/admin/repository/blobstores');
    const pool = await screen.findByRole('row', { name: 'Open pool' });
    expect(within(pool).getByText('Group')).toBeInTheDocument();
    const def = screen.getByRole('row', { name: 'Open default' });
    expect(within(def).getByText('Started')).toBeInTheDocument();
    expect(within(def).getByText('File')).toBeInTheDocument();
    expect(screen.getByRole('region', { name: 'What is a blob store?' })).toBeInTheDocument();
    await userEvent.type(screen.getByRole('searchbox', { name: 'Filter by name...' }), 'spa');
    expect(screen.queryByRole('row', { name: 'Open pool' })).not.toBeInTheDocument();
    await userEvent.clear(screen.getByRole('searchbox', { name: 'Filter by name...' }));
    await userEvent.click(screen.getByRole('row', { name: 'Open pool' }));
    expect(await screen.findByRole('heading', { name: 'Edit: pool' })).toBeInTheDocument();
  });
});

describe('a blob store page', () => {
  it('refuses to delete a used store', async () => {
    serve({
      'GET /service/rest/v1/blobstores/file/default': () =>
        jsonResponse({ softQuota: null, path: 'default' }),
    });
    page('/admin/repository/blobstores/file/default');
    expect(await screen.findByRole('button', { name: 'Delete' })).toBeDisabled();
    expect(screen.getByText('File Blob Store')).toBeInTheDocument();
    expect(await screen.findByRole('textbox', { name: /Path/ })).toHaveValue('default');
    expect(screen.getByRole('button', { name: 'Save' })).toBeDisabled();
  });

  it('shows the refusal of a member removal on the members field', async () => {
    const server = serve({
      'GET /service/rest/v1/blobstores/group/pool': () =>
        jsonResponse({
          name: 'pool',
          softQuota: { type: 'spaceUsedQuota', limit: 1024 },
          members: ['one', 'two'],
          fillPolicy: 'writeToFirst',
        }),
      'PUT /service/rest/v1/blobstores/group/pool': () =>
        jsonResponse(
          [{ id: 'members', message: 'one holds 3 blobs of the group; removing it would lose them' }],
          400,
        ),
    });
    page('/admin/repository/blobstores/group/pool');
    const members = await screen.findByRole('list', { name: 'Member stores' });
    const first = within(members).getAllByRole('listitem')[0] as HTMLElement;
    await userEvent.click(within(first).getByRole('button', { name: 'Remove' }));
    await userEvent.click(screen.getByRole('combobox', { name: /Member stores/ }));
    expect(screen.getAllByRole('option').map((option) => option.textContent)).toEqual([
      'one (File)',
      'spare (File)',
    ]);
    await userEvent.click(screen.getByRole('option', { name: 'spare (File)' }));
    await userEvent.click(screen.getByRole('button', { name: 'Save' }));
    expect(
      await screen.findByText('one holds 3 blobs of the group; removing it would lose them'),
    ).toBeInTheDocument();
    const put = server.calls.find((call) => call.method === 'PUT');
    expect(put?.body).toEqual({
      name: 'pool',
      softQuota: { type: 'spaceUsedQuota', limit: 1024 },
      members: ['two', 'spare'],
      fillPolicy: 'writeToFirst',
    });
  });

  it('is read only without the update permission', async () => {
    serve({
      'GET /service/rest/v1/blobstores/file/spare': () => jsonResponse({ softQuota: null, path: 'spare' }),
    });
    page('/admin/repository/blobstores/file/spare', {
      ...ADMIN_STATE,
      permissions: ['astrum:blobstores:read'],
    });
    expect(await screen.findByText('You can read these settings but not change them.')).toBeInTheDocument();
    expect(screen.getByRole('textbox', { name: /Path/ })).toBeDisabled();
    expect(screen.queryByRole('button', { name: 'Save' })).not.toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'Delete' })).not.toBeInTheDocument();
  });
});

describe('creating a blob store', () => {
  it('walks through the basic, credential and advanced steps', async () => {
    const server = serve({
      'POST /service/rest/v1/blobstores/file': () => new Response(null, { status: 204 }),
    });
    page('/admin/repository/blobstores/create/file');
    const next = await screen.findByRole('button', { name: 'Continue' });
    expect(next).toBeDisabled();
    await userEvent.type(await screen.findByRole('textbox', { name: /Name/ }), 'archive');
    await userEvent.type(screen.getByRole('textbox', { name: /Path/ }), 'archive');
    await userEvent.click(next);
    expect(await screen.findByText('No Credentials Needed')).toBeInTheDocument();
    await userEvent.click(screen.getByRole('button', { name: 'Continue' }));
    await userEvent.click(await screen.findByRole('checkbox', { name: 'Enable Soft Quota' }));
    await userEvent.type(screen.getByRole('spinbutton', { name: /Constraint Limit/ }), '2');
    await userEvent.click(screen.getByRole('button', { name: 'Create Blob Store' }));
    await waitFor(() => expect(server.calls.some((call) => call.method === 'POST')).toBe(true));
    expect(server.calls.find((call) => call.method === 'POST')?.body).toEqual({
      name: 'archive',
      path: 'archive',
      softQuota: { type: 'spaceUsedQuota', limit: 2 * 1024 * 1024 },
    });
    expect(await screen.findByText('Blob store "archive" created')).toBeInTheDocument();
  });
});
