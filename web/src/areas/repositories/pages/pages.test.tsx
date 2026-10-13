import { configure, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import type { UiState } from '../../../api';
import { makeTranslate } from '../../../i18n';
import { ADMIN_STATE, fakeServer, jsonResponse, renderWithApp } from '../../../test/render';
import type { Recipe, UiRepository } from '../api';
import type { RepositoryDocument } from '../model/document';
import reference from '../model/reference-documents.json';
import CreatePickerPage, { groupRecipes } from './CreatePickerPage';
import CreateRepositoryPage from './CreateRepositoryPage';
import RepositoriesPage, { filterRepositories, sortRepositories } from './RepositoriesPage';
import RepositoryPage from './RepositoryPage';

vi.setConfig({ testTimeout: 30_000 });
configure({ asyncUtilTimeout: 5_000 });

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
  it('filters by name, format, type and status', () => {
    const none = new Set<string>();
    const all = { q: '', formats: none, types: none, statuses: none };
    expect(filterRepositories(ROWS, { ...all, q: 'FILES' }).map((r) => r.name)).toEqual(['raw-files']);
    expect(
      filterRepositories(ROWS, { ...all, formats: new Set(['maven2']), types: new Set(['group']) }).map(
        (r) => r.name,
      ),
    ).toEqual(['maven-public']);
    expect(filterRepositories(ROWS, { ...all, statuses: new Set(['offline']) }).map((r) => r.name)).toEqual([
      'maven-central',
    ]);
  });

  it('sorts by the chosen field in both directions', () => {
    const names = (field: Parameters<typeof sortRepositories>[1], direction: 'asc' | 'desc') =>
      sortRepositories(ROWS.slice(0, 4), field, direction, makeTranslate('en')).map((r) => r.name);
    expect(names('name', 'desc')).toEqual(['raw-files', 'maven-releases', 'maven-public', 'maven-central']);
    expect(names('type', 'asc')).toEqual(['maven-public', 'maven-releases', 'raw-files', 'maven-central']);
  });

  it('shows the repositories the caller administers with filters, counts and row actions', async () => {
    serve({ [`GET ${LIST}`]: () => jsonResponse(ROWS) });
    renderWithApp(<RepositoriesPage />, { path: '/admin/repository/repositories?format=maven2' });
    expect(await screen.findByText('Showing 3 of 3')).toBeInTheDocument();
    expect(screen.getByRole('row', { name: 'Open maven-central' })).toBeInTheDocument();
    expect(screen.queryByText('raw-files')).not.toBeInTheDocument();
    expect(screen.queryByText('npm-seen')).not.toBeInTheDocument();
    expect(screen.getByRole('checkbox', { name: 'Maven' })).toBeChecked();
    expect(screen.getByRole('link', { name: 'Create Repository' })).toHaveAttribute(
      'href',
      '/admin/repository/repositories/create',
    );
    expect(screen.getAllByText('Not supported')).toHaveLength(2);
    expect(screen.getByRole('button', { name: 'Analyze' })).toBeInTheDocument();
    await userEvent.type(screen.getByRole('searchbox', { name: /Search repositories/ }), 'nothing-like-this');
    expect(await screen.findByText('No repositories match the current filters')).toBeInTheDocument();
    await userEvent.click(screen.getByRole('button', { name: 'Reset Filters' }));
    expect(await screen.findByRole('row', { name: 'Open raw-files' })).toBeInTheDocument();
    await userEvent.click(screen.getByRole('button', { name: 'Actions for raw-files' }));
    const menu = await screen.findByRole('menu');
    expect(
      within(menu)
        .getAllByRole('menuitem')
        .map((item) => item.textContent),
    ).toEqual(['Edit', 'Rebuild Index', 'Take Offline', 'Copy URL', 'Delete']);
  });

  it('hides the create button from readers', async () => {
    serve({ [`GET ${LIST}`]: () => jsonResponse(ROWS) });
    const state: UiState = { ...ADMIN_STATE, permissions: ['astrum:repository-admin:*:*:read'] };
    renderWithApp(<RepositoriesPage />, { state });
    expect(await screen.findByRole('row', { name: 'Open raw-files' })).toBeInTheDocument();
    expect(screen.queryByRole('link', { name: 'Create Repository' })).not.toBeInTheDocument();
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

  it('walks from the format to the type and on to its form', async () => {
    serve({ 'GET /service/rest/internal/ui/repositories/recipes': () => jsonResponse(RECIPES) });
    const state: UiState = { ...ADMIN_STATE, permissions: ['astrum:repository-admin:raw:*:*'] };
    renderWithApp(null, {
      state,
      path: '/admin/repository/repositories/create',
      routes: [
        { path: '/admin/repository/repositories/create', element: <CreatePickerPage /> },
        { path: '/admin/repository/repositories/create/:format/:type', element: <p>the form</p> },
      ],
    });
    expect(await screen.findByRole('heading', { name: /Choose a Format/ })).toHaveTextContent('1');
    expect(screen.queryByRole('button', { name: 'Maven' })).not.toBeInTheDocument();
    await userEvent.click(screen.getByRole('button', { name: 'Raw' }));
    expect(await screen.findByRole('heading', { name: /Choose a Type/ })).toHaveTextContent('3');
    expect(
      screen.getAllByRole('button', { name: /^Create a .*Raw/ }).map((button) => button.textContent),
    ).toEqual([
      expect.stringMatching(/^Proxy/),
      expect.stringMatching(/^Hosted/),
      expect.stringMatching(/^Group/),
    ]);
    await userEvent.click(screen.getByRole('button', { name: 'Create a Raw proxy repository' }));
    expect(await screen.findByText('the form')).toBeInTheDocument();
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
        { path: '/admin/repository/repositories', element: <p>the list</p> },
      ],
    });
    expect(
      await screen.findByRole('heading', { name: 'Create Maven Repository (Hosted)' }),
    ).toBeInTheDocument();
    const next = screen.getByRole('button', { name: 'Continue' });
    expect(next).toBeDisabled();
    await userEvent.type(await screen.findByRole('textbox', { name: /Name/ }), 'maven-new');
    await waitFor(() => expect(next).toBeEnabled());
    await userEvent.click(next);
    expect(await screen.findByText('the list')).toBeInTheDocument();
    const posted = server.calls.find((call) => call.method === 'POST');
    expect(posted?.body).toMatchObject({
      name: 'maven-new',
      online: true,
      storage: { blobStoreName: 'default', writePolicy: 'ALLOW_ONCE', strictContentTypeValidation: true },
      maven: { versionPolicy: 'RELEASE', layoutPolicy: 'STRICT', contentDisposition: 'ATTACHMENT' },
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
  it('opens on the settings with the old header and tabs', async () => {
    editPage('maven-central', {
      'GET /service/rest/v1/repositories/maven/proxy/maven-central': () => jsonResponse(proxyDocument()),
    });
    expect(await screen.findByRole('tab', { name: /Settings/, selected: true })).toBeInTheDocument();
    expect(screen.getAllByRole('tab').map((tab) => tab.textContent)).toEqual([
      expect.stringMatching(/^Summary/),
      expect.stringMatching(/^Settings/),
      expect.stringMatching(/^Audit/),
      expect.stringMatching(/^Tasks & Capabilities/),
      expect.stringMatching(/^Access & Security/),
    ]);
    expect(screen.getByRole('link', { name: 'Browse Repository' })).toHaveAttribute(
      'href',
      '/browse/maven-central',
    );
    expect(screen.getByRole('button', { name: 'Rebuild Index' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Delete' })).toBeInTheDocument();
    expect(await screen.findByRole('button', { name: 'Save Changes' })).toBeDisabled();
  });

  it('takes a repository offline after asking', async () => {
    const { server } = editPage('maven-central', {
      'GET /service/rest/v1/repositories/maven/proxy/maven-central': () => jsonResponse(proxyDocument()),
      'PUT /service/rest/v1/repositories/maven/proxy/maven-central': () =>
        new Response(null, { status: 204 }),
    });
    await screen.findByRole('button', { name: 'Save Changes' });
    await userEvent.click(await screen.findByRole('button', { name: 'Take Offline' }));
    const dialog = await screen.findByRole('alertdialog', { name: 'Take Repository Offline' });
    await userEvent.click(within(dialog).getByRole('button', { name: 'Take Offline' }));
    await waitFor(() => expect(server.calls.some((call) => call.method === 'PUT')).toBe(true));
    const put = server.calls.find((call) => call.method === 'PUT');
    expect(put?.body).toEqual({ ...proxyDocument(), online: false });
    expect(await screen.findByText('Repository "maven-central" is offline')).toBeInTheDocument();
  });

  it('invalidates the cache of a proxy', async () => {
    const { server } = editPage('maven-central', {
      'GET /service/rest/v1/repositories/maven/proxy/maven-central': () => jsonResponse(proxyDocument()),
      'POST /service/rest/v1/repositories/maven-central/invalidate-cache': () =>
        new Response(null, { status: 204 }),
    });
    await userEvent.click(await screen.findByRole('button', { name: 'Invalidate Cache' }));
    await userEvent.click(
      within(await screen.findByRole('alertdialog')).getByRole('button', { name: 'Invalidate Cache' }),
    );
    expect(await screen.findByText('The cache of "maven-central" was invalidated')).toBeInTheDocument();
    expect(server.calls.map((call) => `${call.method} ${call.path}`)).toContain(
      'POST /service/rest/v1/repositories/maven-central/invalidate-cache',
    );
  });

  it('deletes only after the name is typed', async () => {
    const { server } = editPage('maven-central', {
      'GET /service/rest/v1/repositories/maven/proxy/maven-central': () => jsonResponse(proxyDocument()),
      'DELETE /service/rest/v1/repositories/maven-central': () => new Response(null, { status: 204 }),
    });
    await userEvent.click(await screen.findByRole('button', { name: 'Delete' }));
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

  it('shows the summary, the audit and the access of a repository', async () => {
    const { server } = editPage('maven-public', {
      'GET /service/rest/v1/repositories/maven/group/maven-public': () =>
        jsonResponse({ ...(DOCUMENTS['maven/group/min'] as RepositoryDocument), name: 'maven-public' }),
      'GET /service/rest/internal/ui/blobstores': () => jsonResponse([]),
      'GET /service/rest/v1/security/privileges': () =>
        jsonResponse([
          {
            type: 'repository-view',
            name: 'view-public',
            repository: 'maven-public',
            format: 'maven2',
            actions: ['READ'],
          },
          {
            type: 'repository-view',
            name: 'view-other',
            repository: 'raw-files',
            format: 'raw',
            actions: ['READ'],
          },
        ]),
      'GET /service/rest/v1/security/roles': () =>
        jsonResponse([
          { id: 'readers', privileges: ['view-public'], roles: [] },
          { id: 'nested', privileges: [], roles: ['readers'] },
          { id: 'others', privileges: ['view-other'], roles: [] },
        ]),
      'GET /service/rest/v1/security/users': () => jsonResponse([{ userId: 'jane', roles: ['nested'] }]),
      'GET /service/rest/v1/security/anonymous': () => jsonResponse({ enabled: false, userId: 'anonymous' }),
    });
    const event = {
      id: 7,
      at: '2026-10-12T10:00:00Z',
      domain: 'repository',
      type: 'created',
      context: 'maven-public',
      initiator: 'admin',
      attributes: {},
    };
    vi.stubGlobal('fetch', (input: RequestInfo | URL, init?: RequestInit) =>
      String(input).startsWith('/service/rest/internal/audit?repository=maven-public&')
        ? Promise.resolve(jsonResponse([event]))
        : server.fetch(input, init),
    );
    await userEvent.click(await screen.findByRole('tab', { name: /Summary/ }));
    expect(await screen.findByRole('region', { name: 'Repository Metrics' })).toBeInTheDocument();
    await userEvent.click(screen.getByRole('tab', { name: /Audit/ }));
    expect(
      await screen.findByText('Repository "maven-public": created (initiator: admin)'),
    ).toBeInTheDocument();
    expect(screen.getByRole('link', { name: 'Full Audit Log' })).toHaveAttribute(
      'href',
      '/audit?repositoryName=maven-public',
    );
    await userEvent.click(screen.getByRole('tab', { name: /Access & Security/ }));
    expect(await screen.findByText('view-public')).toBeInTheDocument();
    expect(screen.queryByText('view-other')).not.toBeInTheDocument();
    expect(screen.getByText('readers')).toBeInTheDocument();
    expect(screen.getByText('nested')).toBeInTheDocument();
    expect(screen.getByText('jane')).toBeInTheDocument();
    expect(screen.getByText('Anonymous users cannot reach this repository.')).toBeInTheDocument();
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
    expect(screen.queryByRole('button', { name: 'Take Offline' })).not.toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'Delete' })).not.toBeInTheDocument();
    expect(screen.queryByRole('tab', { name: /Audit/ })).not.toBeInTheDocument();
  });

  it('says when the repository is unknown', async () => {
    editPage('nothing-here', {});
    expect(
      await screen.findByText('There is no repository named nothing-here, or you cannot see it.'),
    ).toBeInTheDocument();
  });
});
