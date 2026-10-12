import { screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { ANONYMOUS_STATE, fakeServer, jsonResponse, renderWithApp } from '../../../test/render';
import { setMedia } from '../../../test/setup';
import { COMPACT, PHONE } from '../../../ui';
import { FLAGS, repository } from '../test-fixtures';
import RepositoryListPage, { VIEW_KEY } from './RepositoryListPage';

const ITEMS = [
  repository('maven-releases', 'maven2', 'hosted', { componentCount: 12, totalSize: 4096 }),
  repository('maven-central', 'maven2', 'proxy', { remoteUrl: 'https://repo1.maven.org/maven2/' }),
  repository('npm-all', 'npm', 'group', { members: ['npm-internal'], componentCount: 0 }),
  repository('raw-old', 'raw', 'hosted', {
    online: false,
    permissions: { ...FLAGS, admin: false },
  }),
];

function serve(items = ITEMS) {
  vi.stubGlobal(
    'fetch',
    fakeServer({ 'GET /service/rest/internal/ui/repositories': () => jsonResponse(items) }).fetch,
  );
}

function page(path = '/browse') {
  return renderWithApp(null, { path, routes: [{ path: 'browse', element: <RepositoryListPage /> }] });
}

const names = () => screen.getAllByRole('rowheader').map((cell) => cell.textContent);

afterEach(() => {
  vi.unstubAllGlobals();
});

describe('the repository list', () => {
  it('filters by format and type and keeps the filters in the address', { timeout: 15_000 }, async () => {
    serve();
    const { router } = page();
    await screen.findByRole('table');
    expect(names()).toEqual(['maven-central', 'maven-releases', 'npm-all', 'raw-old']);
    const filters = screen.getByRole('complementary', { name: 'Filters' });
    await userEvent.click(within(filters).getByRole('checkbox', { name: /Maven/ }));
    expect(names()).toEqual(['maven-central', 'maven-releases']);
    expect(router.state.location.search).toBe('?format=maven2');
    await userEvent.click(within(filters).getByRole('checkbox', { name: /Hosted/ }));
    expect(names()).toEqual(['maven-releases']);
  });

  it('filters by status and name', { timeout: 15_000 }, async () => {
    serve();
    page('/browse?status=offline');
    await screen.findByRole('table');
    expect(names()).toEqual(['raw-old']);
    const filters = screen.getByRole('complementary', { name: 'Filters' });
    await userEvent.click(within(filters).getByRole('button', { name: 'Reset filters' }));
    await userEvent.type(screen.getByRole('searchbox', { name: 'Filter repositories by name' }), 'npm');
    expect(names()).toEqual(['npm-all']);
    expect(screen.getByRole('status')).toHaveTextContent('1 of 4 shown');
  });

  it('reads filters and order from the address', async () => {
    serve();
    page('/browse?type=hosted,proxy&sort=components&direction=desc');
    await screen.findByRole('table');
    expect(names()).toEqual(['maven-releases', 'maven-central', 'raw-old']);
    expect(screen.getByRole('columnheader', { name: /Components/ })).toHaveAttribute(
      'aria-sort',
      'descending',
    );
  });

  it('switches to cards and remembers it', async () => {
    serve();
    page();
    await screen.findByRole('table');
    await userEvent.click(screen.getByRole('radio', { name: 'Cards' }));
    const cards = screen.getByRole('list', { name: 'Repositories' });
    expect(within(cards).getAllByRole('listitem')).toHaveLength(4);
    expect(window.localStorage.getItem(VIEW_KEY)).toBe('cards');
    expect(within(cards).getByText('Groups 1 repository')).toBeInTheDocument();
    expect(within(cards).getAllByRole('link', { name: /Settings of/ })).toHaveLength(3);
  });

  it('shows only cards on a phone', async () => {
    setMedia(PHONE, true);
    setMedia(COMPACT, true);
    serve();
    page();
    expect(await screen.findByRole('list', { name: 'Repositories' })).toBeInTheDocument();
    expect(screen.queryByRole('table')).toBeNull();
    expect(screen.queryByRole('radio', { name: 'Table' })).toBeNull();
    await userEvent.click(screen.getByRole('button', { name: 'Filters' }));
    const dialog = await screen.findByRole('dialog', { name: 'Filters' });
    await userEvent.click(within(dialog).getByRole('checkbox', { name: /Proxy/ }));
    await userEvent.click(within(dialog).getByRole('button', { name: 'Show 1 repository' }));
    expect(screen.getByRole('button', { name: 'Filters: 1' })).toBeInTheDocument();
  });

  it('explains an empty list', async () => {
    serve([]);
    renderWithApp(null, {
      path: '/browse',
      state: ANONYMOUS_STATE,
      routes: [{ path: 'browse', element: <RepositoryListPage /> }],
    });
    expect(await screen.findByRole('heading', { name: 'No repositories to show' })).toBeInTheDocument();
  });
});
