import { screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { ANONYMOUS_STATE, fakeServer, jsonResponse, renderWithApp } from '../../../test/render';
import { setMedia } from '../../../test/setup';
import { COMPACT, PHONE } from '../../../ui';
import { FLAGS, repository } from '../test-fixtures';
import RepositoryListPage from './RepositoryListPage';

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

describe('the repository list', { timeout: 20_000 }, () => {
  it('filters by format and type and keeps the filters in the address', async () => {
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

  it('filters by status and name and shows the total beside the title', async () => {
    serve();
    page('/browse?status=offline');
    await screen.findByRole('table');
    expect(screen.getByText('total 4')).toBeInTheDocument();
    expect(names()).toEqual(['raw-old']);
    const filters = screen.getByRole('complementary', { name: 'Filters' });
    await userEvent.click(within(filters).getByRole('button', { name: 'Reset filters' }));
    await userEvent.type(screen.getByRole('searchbox', { name: 'Filter repositories by name' }), 'npm');
    expect(names()).toEqual(['npm-all']);
  });

  it('shows the URL, a row menu and the pager', async () => {
    serve();
    page('/browse?type=hosted,proxy&sort=type&direction=desc');
    await screen.findByRole('table');
    expect(names()).toEqual(['maven-central', 'maven-releases', 'raw-old']);
    expect(screen.getByRole('columnheader', { name: /Type/ })).toHaveAttribute('aria-sort', 'descending');
    expect(screen.getAllByRole('columnheader').map((cell) => cell.textContent)).toEqual([
      'Name',
      'Format',
      'Type',
      'Status',
      'URL',
      'Actions',
    ]);
    expect(screen.getByText('/repository/maven-central/')).toBeInTheDocument();
    await userEvent.click(screen.getByRole('button', { name: 'Actions for raw-old' }));
    const menu = await screen.findByRole('menu');
    expect(
      within(menu)
        .getAllByRole('menuitem')
        .map((item) => item.textContent),
    ).toEqual(['Open Repository', 'Copy URL']);
    await userEvent.keyboard('{Escape}');
    const pager = screen.getByRole('navigation', { name: 'Pages' });
    expect(within(pager).getByText('of 3')).toBeInTheDocument();
    expect(within(pager).getByRole('button', { name: 'Page 1' })).toHaveAttribute('aria-current', 'page');
  });

  it('shows compact cards on a phone', async () => {
    setMedia(PHONE, true);
    setMedia(COMPACT, true);
    serve();
    page();
    const cards = await screen.findByRole('list', { name: 'Repositories' });
    expect(within(cards).getAllByRole('listitem')).toHaveLength(4);
    expect(within(cards).getAllByRole('button', { name: /Copy the URL of/ })).toHaveLength(4);
    expect(screen.queryByRole('table')).toBeNull();
    await userEvent.click(screen.getByRole('button', { name: 'Filters' }));
    const dialog = await screen.findByRole('dialog', { name: 'Filters' });
    await userEvent.click(within(dialog).getByRole('checkbox', { name: /Proxy/ }));
    await userEvent.keyboard('{Escape}');
    expect(within(screen.getByRole('list', { name: 'Repositories' })).getAllByRole('listitem')).toHaveLength(
      1,
    );
  });

  it('explains an empty list', async () => {
    serve([]);
    renderWithApp(null, {
      path: '/browse',
      state: ANONYMOUS_STATE,
      routes: [{ path: 'browse', element: <RepositoryListPage /> }],
    });
    expect(
      await screen.findByRole('heading', { name: /There are no repositories you may open/ }),
    ).toBeInTheDocument();
  });
});
