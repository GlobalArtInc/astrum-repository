import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import type { UiState } from '../../../api';
import { ADMIN_STATE, fakeServer, jsonResponse, renderWithApp } from '../../../test/render';
import type { UiBlobStore } from '../api';
import BlobStorePage from './BlobStorePage';
import BlobStoresPage from './BlobStoresPage';
import CreateBlobStorePage from './CreateBlobStorePage';

vi.setConfig({ testTimeout: 30_000 });

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
  it('shows usage, quotas and links', async () => {
    serve({});
    page('/admin/repository/blobstores');
    expect(await screen.findByRole('link', { name: 'pool' })).toHaveAttribute(
      'href',
      '/admin/repository/blobstores/group/pool',
    );
    const pool = screen.getByRole('link', { name: 'pool' }).closest('tr') as HTMLElement;
    expect(within(pool).getByText('Exceeded')).toBeInTheDocument();
    const def = screen.getByRole('link', { name: 'default' }).closest('tr') as HTMLElement;
    expect(within(def).getByText('1 repository')).toBeInTheDocument();
    const one = screen.getByRole('link', { name: 'one' }).closest('tr') as HTMLElement;
    expect(within(one).getByText('member of pool')).toBeInTheDocument();
    await userEvent.click(within(one).getByRole('button', { name: 'Actions' }));
    expect(await screen.findByRole('menuitem', { name: 'Delete' })).toHaveAttribute('data-disabled');
  });

  it('deletes an unused store after asking', async () => {
    const server = serve({
      'DELETE /service/rest/v1/blobstores/spare': () => new Response(null, { status: 204 }),
    });
    page('/admin/repository/blobstores');
    const spare = (await screen.findByRole('link', { name: 'spare' })).closest('tr') as HTMLElement;
    await userEvent.click(within(spare).getByRole('button', { name: 'Actions' }));
    await userEvent.click(await screen.findByRole('menuitem', { name: 'Delete' }));
    await userEvent.click(
      within(await screen.findByRole('alertdialog')).getByRole('button', { name: 'Delete' }),
    );
    expect(await screen.findByText('Blob store spare deleted')).toBeInTheDocument();
    expect(server.calls.some((call) => call.method === 'DELETE')).toBe(true);
  });
});

describe('a blob store page', () => {
  it('explains why a used store cannot be deleted', async () => {
    serve({
      'GET /service/rest/v1/blobstores/file/default': () =>
        jsonResponse({ softQuota: null, path: 'default' }),
    });
    page('/admin/repository/blobstores/file/default');
    expect(await screen.findByRole('button', { name: 'Delete' })).toBeDisabled();
    expect(
      screen.getByText('It cannot be deleted while it is in use. The repositories raw-files use it.'),
    ).toBeInTheDocument();
    expect(await screen.findByRole('textbox', { name: /Path/ })).toHaveValue('default');
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
    await userEvent.click(await screen.findByRole('button', { name: 'Remove one' }));
    await userEvent.click(screen.getByRole('combobox', { name: 'Add a blob store…' }));
    expect(screen.getAllByRole('option').map((option) => option.textContent)).toEqual(['one', 'spare']);
    await userEvent.click(screen.getByRole('option', { name: 'spare' }));
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
  it('fills the path from the name and opens the new store', async () => {
    const server = serve({
      'POST /service/rest/v1/blobstores/file': () => new Response(null, { status: 204 }),
      'GET /service/rest/v1/blobstores/file/archive': () =>
        jsonResponse({ softQuota: null, path: 'archive' }),
    });
    page('/admin/repository/blobstores/create/file');
    await userEvent.type(await screen.findByRole('textbox', { name: /Name/ }), 'archive');
    expect(screen.getByRole('textbox', { name: /Path/ })).toHaveValue('archive');
    await userEvent.click(screen.getByRole('switch', { name: 'Enable a soft quota' }));
    await userEvent.type(screen.getByRole('spinbutton', { name: /Limit/ }), '2');
    await userEvent.click(screen.getByRole('button', { name: 'Create blob store' }));
    await waitFor(() => expect(server.calls.some((call) => call.method === 'POST')).toBe(true));
    expect(server.calls.find((call) => call.method === 'POST')?.body).toEqual({
      name: 'archive',
      path: 'archive',
      softQuota: { type: 'spaceUsedQuota', limit: 2 * 1024 * 1024 },
    });
    expect(await screen.findByText('Blob store archive created')).toBeInTheDocument();
  });
});
