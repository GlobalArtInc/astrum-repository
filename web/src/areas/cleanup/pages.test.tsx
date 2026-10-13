import { configure, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { fakeServer, jsonResponse, renderWithApp } from '../../test/render';
import CleanupPoliciesPage from './CleanupPoliciesPage';
import CleanupPolicyPage from './CleanupPolicyPage';
import type { Policy } from './model';

vi.setConfig({ testTimeout: 30_000 });
configure({ asyncUtilTimeout: 5_000 });

const BASE = '/service/rest/internal/cleanup-policies';

const FORMATS = [
  { id: '*', name: 'All Formats', availableCriteria: ['lastDownloaded', 'lastBlobUpdated'] },
  { id: 'raw', name: 'Raw', availableCriteria: ['regex', 'lastDownloaded', 'lastBlobUpdated'] },
  {
    id: 'maven2',
    name: 'Maven',
    availableCriteria: ['regex', 'isPrerelease', 'lastDownloaded', 'lastBlobUpdated'],
  },
];

function policy(name: string, extra: Partial<Policy> = {}): Policy {
  return {
    name,
    format: 'raw',
    notes: '',
    criteriaLastBlobUpdated: null,
    criteriaLastDownloaded: 90,
    criteriaReleaseType: null,
    criteriaAssetRegex: null,
    inUseCount: 0,
    repositories: [],
    ...extra,
  };
}

const POLICIES = [
  policy('demo-cleanup', { notes: 'Three months without downloads' }),
  policy('everything', { format: '*' }),
  policy('old-snapshots', { format: 'maven2', criteriaReleaseType: 'PRERELEASES' }),
];

afterEach(() => {
  vi.unstubAllGlobals();
});

function serve(handlers: Parameters<typeof fakeServer>[0]) {
  const server = fakeServer({
    [`GET ${BASE}`]: () => jsonResponse(POLICIES),
    [`GET ${BASE}/criteria/formats`]: () => jsonResponse(FORMATS),
    'GET /service/rest/v1/repositories': () =>
      jsonResponse([
        { name: 'raw-docs', format: 'raw', type: 'hosted' },
        { name: 'raw-all', format: 'raw', type: 'group' },
        { name: 'maven-releases', format: 'maven2', type: 'hosted' },
      ]),
    ...handlers,
  });
  vi.stubGlobal('fetch', server.fetch);
  return server;
}

function page(path: string) {
  return renderWithApp(null, {
    path,
    routes: [
      { path: '/admin/repository/cleanup-policies', element: <CleanupPoliciesPage /> },
      { path: '/admin/repository/cleanup-policies/create', element: <CleanupPolicyPage /> },
      { path: '/admin/repository/cleanup-policies/:name', element: <CleanupPolicyPage /> },
    ],
  });
}

describe('the cleanup policy list', () => {
  it('explains policies, links them and filters by name, format and description', async () => {
    serve({});
    page('/admin/repository/cleanup-policies');
    expect(await screen.findByRole('button', { name: 'Open demo-cleanup' })).toBeInTheDocument();
    expect(screen.getByText('About cleanup policies')).toBeInTheDocument();
    expect(screen.getByRole('link', { name: /Learn more/ })).toHaveAttribute('target', '_blank');
    expect(screen.getByText('All Formats')).toBeInTheDocument();
    const filter = screen.getByRole('textbox', { name: /Filter by name/ });
    await userEvent.type(filter, 'maven');
    await waitFor(() =>
      expect(screen.queryByRole('button', { name: 'Open demo-cleanup' })).not.toBeInTheDocument(),
    );
    expect(screen.getByRole('button', { name: 'Open old-snapshots' })).toBeInTheDocument();
    await userEvent.clear(filter);
    await userEvent.type(filter, 'nothing here');
    expect(await screen.findByText('No cleanup policies match "nothing here"')).toBeInTheDocument();
  });

  it('opens a policy from its row', async () => {
    serve({ [`GET ${BASE}/demo-cleanup`]: () => jsonResponse(policy('demo-cleanup')) });
    page('/admin/repository/cleanup-policies');
    await userEvent.click(await screen.findByRole('button', { name: 'Open demo-cleanup' }));
    expect(await screen.findByRole('heading', { level: 1, name: 'Edit demo-cleanup' })).toBeInTheDocument();
  });
});

describe('a cleanup policy form', () => {
  it('creates a policy with the criteria of its format', async () => {
    const server = serve({
      [`POST ${BASE}`]: ({ body }) => jsonResponse(body),
      [`GET ${BASE}/nightly`]: () => jsonResponse(policy('nightly')),
    });
    page('/admin/repository/cleanup-policies/create');
    const create = await screen.findByRole('button', { name: 'Create' });
    expect(create).toBeDisabled();
    expect(screen.queryByText('Cleanup Criteria')).not.toBeInTheDocument();
    await userEvent.type(screen.getByRole('textbox', { name: /Name/ }), 'nightly');
    await userEvent.click(screen.getByRole('combobox', { name: /Format/ }));
    await userEvent.click(await screen.findByRole('option', { name: 'Raw' }));
    expect(screen.getByText('Unsaved changes')).toBeInTheDocument();
    expect(screen.queryByText('Release Type')).not.toBeInTheDocument();
    await userEvent.click(screen.getByRole('button', { name: 'Create' }));
    expect(await screen.findByText('Turn on at least one criterion')).toBeInTheDocument();
    await userEvent.click(screen.getByRole('checkbox', { name: 'Asset Name Pattern' }));
    await userEvent.type(screen.getByRole('textbox', { name: 'Asset Name Pattern' }), 'nightly/.*');
    await userEvent.click(screen.getByRole('button', { name: 'Create' }));
    await waitFor(() => expect(server.calls.some((call) => call.method === 'POST')).toBe(true));
    expect(server.calls.find((call) => call.method === 'POST')?.body).toEqual({
      name: 'nightly',
      format: 'raw',
      notes: '',
      criteriaLastBlobUpdated: null,
      criteriaLastDownloaded: null,
      criteriaReleaseType: null,
      criteriaAssetRegex: 'nightly/.*',
    });
    expect(await screen.findByText('Cleanup policy "nightly" created successfully')).toBeInTheDocument();
  });

  it('offers the release type for formats that know it', async () => {
    const server = serve({
      [`GET ${BASE}/old-snapshots`]: () => jsonResponse(POLICIES[2]),
      [`PUT ${BASE}/old-snapshots`]: ({ body }) => jsonResponse(body),
    });
    page('/admin/repository/cleanup-policies/old-snapshots');
    expect(await screen.findByText('Format: Maven')).toBeInTheDocument();
    const release = screen.getByRole('combobox', { name: /Release Type/ });
    expect(release).toHaveTextContent('Pre-Releases/Snapshots');
    await userEvent.click(release);
    await userEvent.click(await screen.findByRole('option', { name: 'Releases and pre-releases' }));
    await userEvent.click(screen.getByRole('button', { name: 'Save' }));
    await waitFor(() => expect(server.calls.some((call) => call.method === 'PUT')).toBe(true));
    expect(server.calls.find((call) => call.method === 'PUT')?.body).toMatchObject({
      criteriaReleaseType: null,
      criteriaLastDownloaded: 90,
    });
  });

  it('previews what the policy would delete in a repository of its format and counts it', async () => {
    const server = serve({
      [`GET ${BASE}/demo-cleanup`]: () => jsonResponse(POLICIES[0]),
      [`POST ${BASE}/preview/components?page=0&size=25`]: () =>
        jsonResponse({
          total: 2,
          complete: true,
          page: 0,
          size: 25,
          results: [
            {
              id: 'a',
              repository: 'raw-docs',
              format: 'raw',
              group: '/docs',
              name: '/docs/a.txt',
              version: '',
              assetCount: 1,
              size: 10,
              lastUpdated: null,
              lastDownloaded: null,
            },
            {
              id: 'b',
              repository: 'raw-docs',
              format: 'raw',
              group: '/docs',
              name: '/docs/b.txt',
              version: '',
              assetCount: 1,
              size: 20,
              lastUpdated: null,
              lastDownloaded: null,
            },
          ],
        }),
      [`POST ${BASE}/preview/summary`]: () =>
        jsonResponse({
          repository: 'raw-docs',
          componentCount: 2,
          assetCount: 2,
          totalSize: 30,
          complete: true,
        }),
    });
    page('/admin/repository/cleanup-policies/demo-cleanup');
    await userEvent.click(await screen.findByRole('combobox', { name: 'Preview Repository' }));
    const options = await screen.findAllByRole('option');
    expect(options.map((option) => option.textContent)).toEqual(['raw-docs']);
    await userEvent.click(options[0] as HTMLElement);
    const table = await screen.findByRole('table');
    expect(await within(table).findByText('/docs/a.txt')).toBeInTheDocument();
    expect(screen.getByText('1–2 of 2 matching components')).toBeInTheDocument();
    const csv = screen.getByRole('link', { name: /Download CSV Report/ });
    expect(csv.getAttribute('href')).toContain('repository=raw-docs');
    await userEvent.click(screen.getByRole('button', { name: 'Calculate' }));
    expect(await screen.findByText('Total size')).toBeInTheDocument();
    expect(screen.getByText('30 B')).toBeInTheDocument();
    expect(server.calls.find((call) => call.path.endsWith('/preview/summary'))?.body).toMatchObject({
      repository: 'raw-docs',
      criteriaLastDownloaded: 90,
    });
  });

  it('warns which repositories lose a policy before deleting it', async () => {
    const server = serve({
      [`GET ${BASE}/demo-cleanup`]: () =>
        jsonResponse({ ...POLICIES[0], repositories: ['raw-docs'], inUseCount: 1 }),
      [`DELETE ${BASE}/demo-cleanup`]: () => new Response(null, { status: 204 }),
    });
    page('/admin/repository/cleanup-policies/demo-cleanup');
    await userEvent.click(await screen.findByRole('button', { name: 'Delete' }));
    const dialog = await screen.findByRole('alertdialog');
    expect(dialog).toHaveTextContent('1 repository uses this policy.');
    await userEvent.click(within(dialog).getByRole('button', { name: 'Delete' }));
    expect(await screen.findByText('Cleanup policy "demo-cleanup" deleted successfully')).toBeInTheDocument();
    expect(server.calls.some((call) => call.method === 'DELETE')).toBe(true);
  });
});
