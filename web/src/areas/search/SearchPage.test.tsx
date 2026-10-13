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
    const query = screen.getByRole('textbox', { name: 'Search components' });
    expect(query).toHaveFocus();
    expect(query).toHaveAttribute('placeholder', 'Name, group or version; * stands for any characters');
    await userEvent.type(query, 'tool');
    await waitFor(() => expect(router.state.location.search).toBe('?q=tool'));
    const row = await screen.findByRole('button', { name: 'Open tool-1 in its repository' });
    expect(within(row).getByText('/tools')).toBeInTheDocument();
    expect(screen.getByRole('status')).toHaveTextContent('Shown: 1');
    const searched = calls.filter((url) => url.pathname === '/service/rest/v1/search');
    expect(searched.at(-1)?.search).toBe('?q=tool&direction=desc');
    await userEvent.click(row);
    expect(router.state.location.pathname).toBe('/browse/raw-files/tools/tool-1');
  });

  it('opens the versions of a component and prints npm scopes', async () => {
    stub((url) =>
      url.pathname === '/service/rest/v1/search'
        ? jsonResponse({
            items: [component(2), component(5, { format: 'npm', group: 'team', name: 'kit' })],
            continuationToken: null,
          })
        : undefined,
    );
    const { router } = page('/search?q=kit');
    expect(
      await screen.findByRole('button', { name: 'Open @team/kit in its repository' }),
    ).toBeInTheDocument();
    await userEvent.click(screen.getByRole('button', { name: 'Versions of tool-2 in all repositories' }));
    expect(router.state.location.pathname).toBe('/search/component/maven2%3Aorg.example%3Atool-2');
    expect(router.state.location.search).toBe('?version=1.2.0');
  });

  it('fetches fifty components at a time and loads more', async () => {
    const calls = stub((url) => {
      if (url.pathname !== '/service/rest/v1/search') {
        return undefined;
      }
      const page = Number(url.searchParams.get('continuationToken') ?? '0');
      return jsonResponse({
        items: Array.from({ length: 10 }, (_, index) => component(page * 10 + index + 1)),
        continuationToken: page < 5 ? String(page + 1) : null,
      });
    });
    page('/search?q=tool&sort=name');
    expect(await screen.findByText('Shown: 50')).toBeInTheDocument();
    await userEvent.click(screen.getByRole('button', { name: 'Show More' }));
    expect(await screen.findByText('Shown: 60')).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'Show More' })).toBeNull();
    const last = calls.find((url) => url.searchParams.get('continuationToken') === '5');
    expect(last?.searchParams.get('sort')).toBe('name');
    expect(last?.searchParams.get('direction')).toBe('asc');
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
    const sorts = await screen.findByRole('listbox');
    expect(within(sorts).getAllByRole('option')).toHaveLength(6);
    await userEvent.click(within(sorts).getByRole('option', { name: 'Name, Z to A' }));
    await waitFor(() =>
      expect(router.state.location.search).toBe('?q=tool&format=maven2&sort=name&direction=desc'),
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
    expect(screen.getByRole('button', { name: 'Try Again' })).toBeInTheDocument();
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
    expect(screen.getByText('Build a query')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Remove condition' })).toBeDisabled();
    expect(screen.queryByRole('button', { name: 'Clear All' })).toBeNull();
    await userEvent.type(screen.getByRole('textbox', { name: 'Value' }), 'tool');
    await userEvent.click(screen.getByRole('button', { name: 'Add Condition' }));
    await userEvent.click(screen.getAllByRole('combobox', { name: 'Field' })[1] as HTMLElement);
    await userEvent.click(await screen.findByRole('option', { name: 'Group' }));
    await userEvent.click(screen.getAllByRole('combobox', { name: 'Condition' })[1] as HTMLElement);
    await userEvent.click(await screen.findByRole('option', { name: 'Starts with' }));
    expect(screen.getAllByRole('textbox', { name: 'Value' })[1]).toHaveAttribute(
      'placeholder',
      'for example, org.apache.commons',
    );
    await userEvent.type(screen.getAllByRole('textbox', { name: 'Value' })[1] as HTMLElement, 'org.ex');
    await userEvent.click(screen.getByRole('button', { name: 'Search' }));
    await waitFor(() => expect(router.state.location.search).toBe('?q=tool&group=org.ex*'));
    const row = await screen.findByRole('button', { name: 'Open tool-2 in the repository' });
    expect(within(row).getByText('1.2.0')).toBeInTheDocument();
    expect(screen.getByText('1 component found')).toBeInTheDocument();
    expect(screen.getAllByRole('combobox', { name: 'Field' })).toHaveLength(2);
    const searched = calls.filter((url) => url.pathname === '/service/rest/v1/search');
    expect(searched.at(-1)?.searchParams.get('group')).toBe('org.ex*');
    await userEvent.click(row);
    expect(router.state.location.pathname).toBe('/browse/maven-releases/tools/tool-2');
  });

  it('searches everything without conditions and clears them', async () => {
    const calls = stub((url) =>
      url.pathname === '/service/rest/v1/search'
        ? jsonResponse({ items: [], continuationToken: null })
        : undefined,
    );
    const { router } = page('/search/custom?name=tool*');
    expect(screen.getByRole('textbox', { name: 'Value' })).toHaveValue('tool');
    expect(screen.getByRole('combobox', { name: 'Condition' })).toHaveTextContent('Starts with');
    expect(await screen.findByText('No components found')).toBeInTheDocument();
    await userEvent.click(screen.getByRole('button', { name: 'Clear All' }));
    await waitFor(() => expect(router.state.location.search).toBe(''));
    expect(screen.getByRole('textbox', { name: 'Value' })).toHaveValue('');
    expect(screen.getByText('Build a query')).toBeInTheDocument();
    await userEvent.click(screen.getByRole('button', { name: 'Search' }));
    await waitFor(() =>
      expect(calls.some((url) => url.pathname === '/service/rest/v1/search' && url.search === '')).toBe(true),
    );
  });
});
