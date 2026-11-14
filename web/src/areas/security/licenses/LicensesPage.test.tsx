import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { fakeServer, jsonResponse, renderWithApp } from '../../../test/render';
import LicensesPage from './LicensesPage';

const LICENCES = '/service/rest/v1/security/licenses';
const REPOSITORIES = '/service/rest/v1/repositories';

afterEach(() => {
  vi.unstubAllGlobals();
});

function licence(name: string, state: string, verdict: string) {
  return {
    repository: 'files',
    format: 'raw',
    namespace: '',
    name,
    version: '1',
    declared: 'MIT',
    expression: 'MIT',
    ids: ['MIT'],
    state,
    verdict,
  };
}

describe('the licence report page', () => {
  it('lists the components with their state and verdict as badges', async () => {
    const server = fakeServer({
      [`GET ${REPOSITORIES}`]: () => jsonResponse([]),
      [`GET ${LICENCES}`]: () =>
        jsonResponse({ items: [licence('alpha', 'spdx', 'pass')], continuationToken: null }),
    });
    vi.stubGlobal('fetch', server.fetch);
    renderWithApp(<LicensesPage />);
    await screen.findByText('alpha');
    const table = screen.getByRole('table', { name: 'Licences of components' });
    expect(within(table).getByText('alpha')).toBeInTheDocument();
    expect(within(table).getByText('SPDX')).toBeInTheDocument();
    expect(within(table).getByText('Pass')).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'Load more' })).not.toBeInTheDocument();
  });

  it('shows the empty state when no licence matches', async () => {
    const server = fakeServer({
      [`GET ${REPOSITORIES}`]: () => jsonResponse([]),
      [`GET ${LICENCES}`]: () => jsonResponse({ items: [], continuationToken: null }),
    });
    vi.stubGlobal('fetch', server.fetch);
    renderWithApp(<LicensesPage />);
    expect(await screen.findByText('No licences match')).toBeInTheDocument();
  });

  it('load more appends the next page with its token', async () => {
    const server = fakeServer({
      [`GET ${REPOSITORIES}`]: () => jsonResponse([]),
      [`GET ${LICENCES}`]: () =>
        jsonResponse({ items: [licence('alpha', 'spdx', 'pass')], continuationToken: 'page-2' }),
      [`GET ${LICENCES}?continuationToken=page-2`]: () =>
        jsonResponse({ items: [licence('beta', 'mapped', 'reported')], continuationToken: null }),
    });
    vi.stubGlobal('fetch', server.fetch);
    renderWithApp(<LicensesPage />);
    await screen.findByText('alpha');
    await userEvent.click(await screen.findByRole('button', { name: 'Load more' }));
    expect(await screen.findByText('beta')).toBeInTheDocument();
    expect(screen.getByText('alpha')).toBeInTheDocument();
    await waitFor(() =>
      expect(server.calls.some((call) => call.path === `${LICENCES}?continuationToken=page-2`)).toBe(true),
    );
    expect(screen.queryByRole('button', { name: 'Load more' })).not.toBeInTheDocument();
  });

  it('the export link carries the licence filter', async () => {
    const server = fakeServer({
      [`GET ${REPOSITORIES}`]: () => jsonResponse([]),
      [`GET ${LICENCES}`]: () => jsonResponse({ items: [], continuationToken: null }),
      [`GET ${LICENCES}?license=MIT`]: () => jsonResponse({ items: [], continuationToken: null }),
    });
    vi.stubGlobal('fetch', server.fetch);
    renderWithApp(<LicensesPage />);
    const exportLink = screen.getByRole('link', { name: 'Export CSV' });
    expect(exportLink).toHaveAttribute('href', `${LICENCES}/export.csv`);
    await userEvent.type(screen.getByRole('textbox', { name: 'Licence' }), 'MIT');
    expect(screen.getByRole('link', { name: 'Export CSV' })).toHaveAttribute(
      'href',
      `${LICENCES}/export.csv?license=MIT`,
    );
  });
});
