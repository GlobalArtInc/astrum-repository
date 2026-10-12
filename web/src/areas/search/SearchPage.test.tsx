import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { ADMIN_STATE, jsonResponse, renderWithApp } from '../../test/render';
import CriteriaPage from './CriteriaPage';
import SearchPage from './SearchPage';

vi.setConfig({ testTimeout: 20_000 });

const STATE = {
  ...ADMIN_STATE,
  formats: [
    { id: 'raw', apiName: 'raw', types: ['hosted' as const] },
    { id: 'maven2', apiName: 'maven', types: ['hosted' as const] },
  ],
};

function component(index: number, extra: object = {}) {
  return {
    id: `c${index}`,
    repository: index % 2 ? 'raw-files' : 'maven-releases',
    format: index % 2 ? 'raw' : 'maven2',
    group: index % 2 ? '/tools' : 'org.example',
    name: `tool-${index}`,
    version: index % 2 ? '' : `1.${index}.0`,
    assets: [
      {
        id: `a${index}`,
        path: `/tools/tool-${index}`,
        downloadUrl: `http://localhost/repository/raw-files/tools/tool-${index}`,
        repository: 'raw-files',
        format: 'raw',
        contentType: 'text/plain',
        lastModified: '2026-10-12T10:00:00.000+00:00',
        fileSize: 12,
        checksum: { sha1: 'x' },
      },
    ],
    ...extra,
  };
}

function stub(handler: (url: URL) => Response | undefined) {
  const calls: URL[] = [];
  vi.stubGlobal('fetch', async (input: RequestInfo | URL) => {
    const url = new URL(String(input), 'http://localhost');
    calls.push(url);
    if (url.pathname === '/service/rest/v1/repositories') {
      return jsonResponse([
        { name: 'raw-files', format: 'raw', type: 'hosted', url: '' },
        { name: 'maven-releases', format: 'maven2', type: 'hosted', url: '' },
      ]);
    }
    return handler(url) ?? jsonResponse({ id: '*', message: `no handler for ${url}` }, 404);
  });
  return calls;
}

function page(path: string, element = <SearchPage />) {
  return renderWithApp(null, {
    state: STATE,
    path,
    routes: [
      { path: '/search', element },
      { path: '/search/custom', element: <CriteriaPage /> },
      { path: '*', element: <p>elsewhere</p> },
    ],
  });
}

describe('the search page', () => {
  it('starts with a hint and searches once there is a query', async () => {
    const calls = stub((url) =>
      url.pathname === '/service/rest/v1/search'
        ? jsonResponse({ items: [component(1)], continuationToken: null })
        : undefined,
    );
    const { router } = page('/search');
    expect(screen.getByText('Search all repositories')).toBeInTheDocument();
    await userEvent.type(screen.getByLabelText('Name, group or version'), 'tool');
    await waitFor(() => expect(router.state.location.search).toBe('?q=tool'));
    expect(await screen.findByRole('link', { name: /tool-1/ })).toHaveAttribute(
      'href',
      '/search/component/raw%3A%2Ftools%3Atool-1',
    );
    expect(screen.getByRole('status')).toHaveTextContent('1 component found');
    const searched = calls.filter((url) => url.pathname === '/service/rest/v1/search');
    expect(searched.at(-1)?.search).toBe('?q=tool&direction=desc');
  });

  it('loads more with the continuation token', async () => {
    const calls = stub((url) => {
      if (url.pathname !== '/service/rest/v1/search') {
        return undefined;
      }
      return url.searchParams.get('continuationToken') === 'page-2'
        ? jsonResponse({ items: [component(11), component(12)], continuationToken: null })
        : jsonResponse({
            items: Array.from({ length: 10 }, (_, index) => component(index + 1)),
            continuationToken: 'page-2',
          });
    });
    page('/search?q=tool&sort=name');
    expect(await screen.findByText('10 shown, more can be loaded')).toBeInTheDocument();
    await userEvent.click(screen.getByRole('button', { name: 'Show more' }));
    expect(await screen.findByText('12 components found')).toBeInTheDocument();
    expect(screen.getByRole('link', { name: /tool-12/ })).toBeInTheDocument();
    const second = calls.find((url) => url.searchParams.get('continuationToken') === 'page-2');
    expect(second?.searchParams.get('sort')).toBe('name');
    expect(second?.searchParams.get('direction')).toBe('asc');
  });

  it('keeps filters and sort in the address and clears a repository of another format', async () => {
    stub((url) =>
      url.pathname === '/service/rest/v1/search'
        ? jsonResponse({ items: [], continuationToken: null })
        : undefined,
    );
    const { router } = page('/search?q=tool&repository=raw-files');
    expect(await screen.findByText('Nothing found')).toBeInTheDocument();
    await userEvent.click(screen.getByRole('combobox', { name: 'Format' }));
    await userEvent.click(await screen.findByRole('option', { name: 'Maven' }));
    await waitFor(() => expect(router.state.location.search).toBe('?q=tool&format=maven2'));
    await userEvent.click(screen.getByRole('combobox', { name: 'Sort' }));
    await userEvent.click(await screen.findByRole('option', { name: 'Version, descending' }));
    await waitFor(() =>
      expect(router.state.location.search).toBe('?q=tool&format=maven2&sort=version&direction=desc'),
    );
    await userEvent.click(screen.getByRole('combobox', { name: 'Repository' }));
    const listbox = await screen.findByRole('listbox');
    expect(within(listbox).queryByRole('option', { name: 'raw-files' })).toBeNull();
    await userEvent.click(within(listbox).getByRole('option', { name: 'maven-releases' }));
    await waitFor(() => expect(router.state.location.search).toContain('repository=maven-releases'));
  });

  it('shows a failed search with a retry', async () => {
    stub((url) =>
      url.pathname === '/service/rest/v1/search'
        ? jsonResponse({ id: '*', message: 'broken' }, 400)
        : undefined,
    );
    page('/search?q=tool');
    expect(await screen.findByRole('alert')).toHaveTextContent('broken');
    expect(screen.getByRole('button', { name: 'Try again' })).toBeInTheDocument();
  });
});

describe('the criteria search', () => {
  it('builds the address from the rows and lists components', async () => {
    const calls = stub((url) =>
      url.pathname === '/service/rest/v1/search'
        ? jsonResponse({ items: [component(2)], continuationToken: null })
        : undefined,
    );
    const { router } = page('/search/custom');
    expect(screen.getByText('Build your search')).toBeInTheDocument();
    await userEvent.type(screen.getByLabelText('Value of Keyword'), 'tool');
    await userEvent.click(screen.getByRole('button', { name: 'Add a criterion' }));
    await userEvent.click(screen.getAllByRole('combobox', { name: 'Criterion' })[1] as HTMLElement);
    await userEvent.click(await screen.findByRole('option', { name: 'Maven group ID' }));
    await userEvent.click(screen.getByRole('combobox', { name: 'Condition' }));
    await userEvent.click(await screen.findByRole('option', { name: 'starts with' }));
    await userEvent.type(screen.getByLabelText('Value of Maven group ID'), 'org.ex');
    await userEvent.click(screen.getByRole('button', { name: 'Search' }));
    await waitFor(() => expect(router.state.location.search).toBe('?q=tool&maven.groupId=org.ex*'));
    expect(await screen.findByRole('link', { name: /tool-2/ })).toBeInTheDocument();
    const searched = calls.filter((url) => url.pathname === '/service/rest/v1/search');
    expect(searched.at(-1)?.searchParams.get('maven.groupId')).toBe('org.ex*');
  });

  it('refuses empty searches and bad checksums', async () => {
    stub(() => undefined);
    const { router } = page('/search/custom');
    await userEvent.click(screen.getByRole('button', { name: 'Search' }));
    expect(await screen.findByText('Give at least one criterion a value.')).toBeInTheDocument();
    await userEvent.click(screen.getByRole('combobox', { name: 'Criterion' }));
    await userEvent.click(await screen.findByRole('option', { name: 'SHA-1 checksum' }));
    await userEvent.type(screen.getByLabelText('Value of SHA-1 checksum'), 'abc');
    await userEvent.click(screen.getByRole('button', { name: 'Search' }));
    expect(
      await screen.findByText('A checksum of this kind has 40 hexadecimal characters.'),
    ).toBeInTheDocument();
    expect(router.state.location.search).toBe('');
  });

  it('lists files with download links in the assets mode', async () => {
    stub((url) =>
      url.pathname === '/service/rest/v1/search/assets'
        ? jsonResponse({ items: component(3).assets, continuationToken: null })
        : undefined,
    );
    page('/search/custom?name=tool*&mode=assets');
    const link = await screen.findByRole('link', { name: '/tools/tool-3' });
    expect(link).toHaveAttribute('href', 'http://localhost/repository/raw-files/tools/tool-3');
    expect(link).toHaveAttribute('download');
    expect(screen.getByRole('link', { name: 'raw-files' })).toHaveAttribute(
      'href',
      '/browse/raw-files/tools',
    );
    expect(screen.getByLabelText('Value of Name')).toHaveValue('tool');
  });
});
