import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import type { UiState } from '../../../api';
import { ADMIN_STATE, fakeServer, jsonResponse, renderWithApp } from '../../../test/render';
import type { Recipe, UiRepository } from '../api';
import type { RepositoryDocument } from '../model/document';
import reference from '../model/reference-documents.json';
import CreatePickerPage, { groupRecipes } from './CreatePickerPage';
import CreateRepositoryPage from './CreateRepositoryPage';
import RepositoriesPage, { filterRepositories } from './RepositoriesPage';
import RepositoryPage from './RepositoryPage';

vi.setConfig({ testTimeout: 30_000 });

const DOCUMENTS = reference as unknown as Record<string, RepositoryDocument>;
const LIST = '/service/rest/internal/ui/repositories';
const FLAGS = {
  browse: true,
  read: true,
  add: true,
  edit: true,
  delete: true,
  admin: true,
  adminEdit: true,
  adminDelete: true,
};

function row(
  name: string,
  format: string,
  type: UiRepository['type'],
  extra: Partial<UiRepository> = {},
): UiRepository {
  return {
    name,
    format,
    type,
    url: `http://localhost/repository/${name}`,
    online: true,
    blobStoreName: 'default',
    remoteUrl: type === 'proxy' ? 'https://remote.example.org/' : null,
    memberOf: [],
    componentCount: 3,
    assetCount: 7,
    totalSize: 2048,
    lastUploaded: null,
    limited: false,
    permissions: FLAGS,
    ...extra,
  };
}

const ROWS: UiRepository[] = [
  row('maven-releases', 'maven2', 'hosted', { memberOf: ['maven-public'] }),
  row('maven-central', 'maven2', 'proxy', { memberOf: ['maven-public'], online: false }),
  row('maven-public', 'maven2', 'group', { members: ['maven-releases', 'maven-central'] }),
  row('raw-files', 'raw', 'hosted', { blobStoreName: 'files' }),
  row('npm-seen', 'npm', 'hosted', {
    permissions: { ...FLAGS, admin: false, adminEdit: false, adminDelete: false },
  }),
];

afterEach(() => {
  vi.unstubAllGlobals();
});

function serve(handlers: Parameters<typeof fakeServer>[0]) {
  const server = fakeServer(handlers);
  vi.stubGlobal('fetch', server.fetch);
  return server;
}

describe('the repository list', () => {
  it('filters by text, format, type and status', () => {
    const all = { q: '', format: 'all', type: 'all', status: 'all' };
    expect(filterRepositories(ROWS, { ...all, q: 'FILES' }).map((r) => r.name)).toEqual(['raw-files']);
    expect(filterRepositories(ROWS, { ...all, q: 'remote.example' }).map((r) => r.name)).toEqual([
      'maven-central',
    ]);
    expect(filterRepositories(ROWS, { ...all, format: 'maven2', type: 'group' }).map((r) => r.name)).toEqual([
      'maven-public',
    ]);
    expect(filterRepositories(ROWS, { ...all, status: 'offline' }).map((r) => r.name)).toEqual([
      'maven-central',
    ]);
  });

  it('shows the repositories the caller administers with links to their settings', async () => {
    serve({ [`GET ${LIST}`]: () => jsonResponse(ROWS) });
    renderWithApp(<RepositoriesPage />, { path: '/admin/repository/repositories?format=maven2' });
    expect(await screen.findByRole('link', { name: 'maven-central' })).toHaveAttribute(
      'href',
      '/admin/repository/repositories/maven-central',
    );
    expect(screen.queryByText('raw-files')).not.toBeInTheDocument();
    expect(screen.queryByText('npm-seen')).not.toBeInTheDocument();
    expect(screen.getByText('3 repositories')).toBeInTheDocument();
    expect(screen.getByRole('link', { name: 'Create a repository' })).toHaveAttribute(
      'href',
      '/admin/repository/repositories/create',
    );
    await userEvent.type(screen.getByRole('searchbox', { name: /Find by name/ }), 'nothing-like-this');
    expect(await screen.findByText('No repository matches the filters')).toBeInTheDocument();
    await userEvent.click(screen.getByRole('button', { name: 'Clear the filters' }));
    expect(await screen.findByRole('link', { name: 'raw-files' })).toBeInTheDocument();
  });

  it('hides the create button from readers', async () => {
    serve({ [`GET ${LIST}`]: () => jsonResponse(ROWS) });
    const state: UiState = { ...ADMIN_STATE, permissions: ['astrum:repository-admin:*:*:read'] };
    renderWithApp(<RepositoriesPage />, { state });
    expect(await screen.findByRole('link', { name: 'raw-files' })).toBeInTheDocument();
    expect(screen.queryByRole('link', { name: 'Create a repository' })).not.toBeInTheDocument();
  });
});

const RECIPES: Recipe[] = ['maven2', 'raw'].flatMap((format) =>
  (['hosted', 'proxy', 'group'] as const).map((type) => ({
    id: `${format}-${type}`,
    format,
    apiFormat: format === 'maven2' ? 'maven' : format,
    type,
  })),
);

describe('choosing what to create', () => {
  it('groups recipes by format and keeps only allowed formats', () => {
    const groups = groupRecipes(RECIPES, (format) => format === 'raw', '');
    expect(groups).toEqual([{ format: 'raw', types: ['hosted', 'proxy', 'group'] }]);
    expect(groupRecipes(RECIPES, () => true, 'mav').map((group) => group.format)).toEqual(['maven2']);
  });

  it('links each kind to its form', async () => {
    serve({ 'GET /service/rest/internal/ui/repositories/recipes': () => jsonResponse(RECIPES) });
    const state: UiState = { ...ADMIN_STATE, permissions: ['astrum:repository-admin:raw:*:*'] };
    renderWithApp(<CreatePickerPage />, { state });
    expect(await screen.findByRole('link', { name: 'Create a Raw proxy repository' })).toHaveAttribute(
      'href',
      '/admin/repository/repositories/create/raw/proxy',
    );
    expect(screen.queryByText('Maven')).not.toBeInTheDocument();
  });
});

describe('creating a repository', () => {
  it('posts the form to the format and type and opens the new repository', async () => {
    const server = serve({
      'GET /service/rest/internal/ui/repositories/recipes': () => jsonResponse(RECIPES),
      [`GET ${LIST}`]: () => jsonResponse(ROWS),
      'GET /service/rest/internal/ui/blobstores': () =>
        jsonResponse([
          { name: 'files', groups: [] },
          { name: 'default', groups: [] },
        ]),
      'POST /service/rest/v1/repositories/maven/hosted': () => new Response(null, { status: 201 }),
    });
    renderWithApp(null, {
      path: '/admin/repository/repositories/create/maven2/hosted',
      routes: [
        { path: '/admin/repository/repositories/create/:format/:type', element: <CreateRepositoryPage /> },
        { path: '/admin/repository/repositories/:name', element: <p>opened</p> },
      ],
    });
    expect(await screen.findByRole('heading', { name: 'New hosted Maven repository' })).toBeInTheDocument();
    await userEvent.type(await screen.findByRole('textbox', { name: /Name/ }), 'maven-new');
    await userEvent.click(screen.getByRole('button', { name: 'Create repository' }));
    expect(await screen.findByText('opened')).toBeInTheDocument();
    const posted = server.calls.find((call) => call.method === 'POST');
    expect(posted?.body).toMatchObject({
      name: 'maven-new',
      online: true,
      storage: { blobStoreName: 'default', writePolicy: 'ALLOW_ONCE', strictContentTypeValidation: true },
      maven: { versionPolicy: 'RELEASE', layoutPolicy: 'STRICT' },
    });
  });

  it('explains when the kind cannot be created', async () => {
    serve({
      'GET /service/rest/internal/ui/repositories/recipes': () => jsonResponse(RECIPES),
      [`GET ${LIST}`]: () => jsonResponse(ROWS),
      'GET /service/rest/internal/ui/blobstores': () => jsonResponse([]),
    });
    renderWithApp(null, {
      path: '/admin/repository/repositories/create/apt/group',
      routes: [
        { path: '/admin/repository/repositories/create/:format/:type', element: <CreateRepositoryPage /> },
      ],
    });
    expect(
      await screen.findByRole('heading', {
        level: 2,
        name: 'This kind of repository cannot be created here',
      }),
    ).toBeInTheDocument();
  });
});

function editPage(name: string, handlers: Parameters<typeof fakeServer>[0], state: UiState = ADMIN_STATE) {
  const server = serve({ [`GET ${LIST}`]: () => jsonResponse(ROWS), ...handlers });
  const result = renderWithApp(null, {
    state,
    path: `/admin/repository/repositories/${name}`,
    routes: [
      { path: '/admin/repository/repositories/:name', element: <RepositoryPage /> },
      { path: '/admin/repository/repositories', element: <p>the list</p> },
    ],
  });
  return { server, ...result };
}

function proxyDocument(): RepositoryDocument {
  return { ...(DOCUMENTS['maven/proxy/max'] as RepositoryDocument), name: 'maven-central', online: true };
}

describe('the repository page', () => {
  it('takes a repository offline after asking', async () => {
    const { server } = editPage('maven-central', {
      'GET /service/rest/v1/repositories/maven/proxy/maven-central': () => jsonResponse(proxyDocument()),
      'PUT /service/rest/v1/repositories/maven/proxy/maven-central': () =>
        new Response(null, { status: 204 }),
    });
    await userEvent.click(await screen.findByRole('button', { name: 'Take offline' }));
    const dialog = await screen.findByRole('alertdialog', { name: 'Take maven-central offline?' });
    await userEvent.click(within(dialog).getByRole('button', { name: 'Take offline' }));
    await waitFor(() => expect(server.calls.some((call) => call.method === 'PUT')).toBe(true));
    const put = server.calls.find((call) => call.method === 'PUT');
    expect(put?.body).toEqual({ ...proxyDocument(), online: false });
    expect(await screen.findByText('maven-central is offline')).toBeInTheDocument();
  });

  it('invalidates the cache of a proxy', async () => {
    const { server } = editPage('maven-central', {
      'GET /service/rest/v1/repositories/maven/proxy/maven-central': () => jsonResponse(proxyDocument()),
      'POST /service/rest/v1/repositories/maven-central/invalidate-cache': () =>
        new Response(null, { status: 204 }),
    });
    await userEvent.click(await screen.findByRole('button', { name: 'More actions' }));
    await userEvent.click(await screen.findByRole('menuitem', { name: 'Invalidate cache' }));
    await userEvent.click(
      within(await screen.findByRole('alertdialog')).getByRole('button', { name: 'Invalidate cache' }),
    );
    expect(await screen.findByText('The cache of maven-central was invalidated')).toBeInTheDocument();
    expect(server.calls.map((call) => `${call.method} ${call.path}`)).toContain(
      'POST /service/rest/v1/repositories/maven-central/invalidate-cache',
    );
  });

  it('deletes only after the name is typed', async () => {
    const { server } = editPage('maven-central', {
      'GET /service/rest/v1/repositories/maven/proxy/maven-central': () => jsonResponse(proxyDocument()),
      'DELETE /service/rest/v1/repositories/maven-central': () => new Response(null, { status: 204 }),
    });
    await userEvent.click(await screen.findByRole('button', { name: 'More actions' }));
    await userEvent.click(await screen.findByRole('menuitem', { name: 'Delete' }));
    const dialog = await screen.findByRole('alertdialog', { name: 'Delete maven-central?' });
    expect(dialog).toHaveTextContent('everything it cached');
    expect(dialog).toHaveTextContent('taken out of these groups: maven-public');
    const confirm = within(dialog).getByRole('button', { name: 'Delete the repository' });
    expect(confirm).toBeDisabled();
    await userEvent.type(
      within(dialog).getByRole('textbox', { name: 'Type maven-central to confirm' }),
      'maven-central',
    );
    await userEvent.click(confirm);
    expect(await screen.findByText('the list')).toBeInTheDocument();
    expect(server.calls.some((call) => call.method === 'DELETE')).toBe(true);
  });

  it('shows membership and the audit of a group', async () => {
    editPage('maven-public', {
      'GET /service/rest/v1/repositories/maven/group/maven-public': () =>
        jsonResponse({
          ...(DOCUMENTS['maven/group/min'] as RepositoryDocument),
          name: 'maven-public',
          group: { memberNames: ['maven-releases', 'maven-central', 'hidden-one'] },
        }),
      'GET /service/rest/internal/audit?repository=maven-public&limit=20': () =>
        jsonResponse([
          {
            id: 7,
            at: '2026-10-12T10:00:00Z',
            domain: 'repository',
            type: 'created',
            context: 'maven-public',
            initiator: 'admin',
            attributes: {},
          },
        ]),
    });
    await userEvent.click(await screen.findByRole('tab', { name: /Membership/ }));
    const members = await screen.findByRole('list', { name: 'Members' });
    expect(
      within(members)
        .getAllByRole('listitem')
        .map((item) => item.textContent),
    ).toEqual([
      expect.stringContaining('maven-releases'),
      expect.stringContaining('maven-central'),
      expect.stringContaining('hidden-one'),
    ]);
    expect(within(members).getByText('Not visible to you')).toBeInTheDocument();
    expect(screen.getByText('No group contains this repository.')).toBeInTheDocument();
    await userEvent.click(screen.getByRole('tab', { name: /Audit/ }));
    expect(await screen.findByText('Created')).toBeInTheDocument();
    expect(screen.getByRole('link', { name: 'Open the audit log' })).toHaveAttribute(
      'href',
      '/audit?repositoryName=maven-public',
    );
  });

  it('gives readers the settings without actions', async () => {
    const readers = ROWS.map((repository) => ({
      ...repository,
      permissions: { ...FLAGS, add: false, edit: false, delete: false, adminEdit: false, adminDelete: false },
    }));
    editPage(
      'raw-files',
      {
        [`GET ${LIST}`]: () => jsonResponse(readers),
        'GET /service/rest/v1/repositories/raw/hosted/raw-files': () =>
          jsonResponse({ ...(DOCUMENTS['raw/hosted/max'] as RepositoryDocument), name: 'raw-files' }),
      },
      { ...ADMIN_STATE, permissions: ['astrum:repository-admin:*:*:read'] },
    );
    expect(await screen.findByText('You can read these settings but not change them.')).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'Take offline' })).not.toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'More actions' })).not.toBeInTheDocument();
    expect(screen.queryByRole('tab', { name: /Audit/ })).not.toBeInTheDocument();
  });

  it('says when the repository is unknown', async () => {
    editPage('nothing-here', {});
    expect(
      await screen.findByText('There is no repository named nothing-here, or you cannot see it.'),
    ).toBeInTheDocument();
  });
});
