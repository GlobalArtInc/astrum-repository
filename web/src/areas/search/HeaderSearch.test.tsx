import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { SearchPanel } from '../../shell/search';
import { ADMIN_STATE, ANONYMOUS_STATE, jsonResponse, renderWithApp } from '../../test/render';
import { searchArea } from '.';
import { RECENT_KEY } from './recent';

vi.setConfig({ testTimeout: 20_000 });

function stub() {
  const calls: URL[] = [];
  vi.stubGlobal('fetch', async (input: RequestInfo | URL) => {
    const url = new URL(String(input), 'http://localhost');
    calls.push(url);
    if (url.pathname === '/service/rest/v1/repositories') {
      return jsonResponse(
        ['maven-central', 'maven-releases', 'raw-files', 'npm-maven-mirror'].map((name) => ({
          name,
          format: name.startsWith('raw') ? 'raw' : 'maven2',
          type: 'hosted',
          url: '',
        })),
      );
    }
    if (url.pathname === '/service/rest/v1/search') {
      return jsonResponse({
        items: Array.from({ length: 8 }, (_, index) => ({
          id: `c${index}`,
          repository: 'maven-releases',
          format: 'maven2',
          group: 'org.maven',
          name: `maven-plugin-${index}`,
          version: `3.${index}`,
          assets: [],
        })),
        continuationToken: 'more',
      });
    }
    return jsonResponse({ id: '*', message: 'missing' }, 404);
  });
  return calls;
}

function panel(path: string, state = ADMIN_STATE) {
  return renderWithApp(null, {
    state,
    path,
    areas: [searchArea],
    routes: [{ path: '*', element: <SearchPanel open onOpenChange={() => {}} /> }],
  });
}

describe('the header search', () => {
  it('lists recent searches before anything is typed', async () => {
    stub();
    window.localStorage.setItem(RECENT_KEY, JSON.stringify(['log4j', 'junit']));
    panel('/');
    const recent = await screen.findByRole('group', { name: 'Recent searches' });
    expect(recent).toHaveTextContent('log4j');
    expect(screen.getByRole('option', { name: 'junit' })).toHaveAttribute('href', '/search?q=junit');
  });

  it('offers scopes, five repositories and six components', async () => {
    const calls = stub();
    panel('/browse/raw-files/tools');
    await userEvent.type(screen.getByRole('combobox', { name: 'Search' }), 'maven');
    expect(screen.getByRole('option', { name: 'all repositories' })).toHaveAttribute(
      'href',
      '/search?q=maven',
    );
    expect(screen.getByRole('option', { name: 'this repository, raw-files' })).toHaveAttribute(
      'href',
      '/search?q=maven&repository=raw-files',
    );
    const repositories = await screen.findByRole('group', { name: 'Repositories' });
    expect(repositories.querySelectorAll('[data-search-item]')).toHaveLength(3);
    const components = await screen.findByRole('group', { name: 'Components' });
    expect(components.querySelectorAll('[data-search-item]')).toHaveLength(6);
    expect(screen.getByRole('option', { name: /maven-plugin-0/ })).toHaveAttribute(
      'href',
      '/search/component/maven2%3Aorg.maven%3Amaven-plugin-0?version=3.0',
    );
    const searches = calls.filter((url) => url.pathname === '/service/rest/v1/search');
    expect(searches.map((url) => url.searchParams.get('q'))).toEqual(['maven']);
  });

  it('opens the chosen entry with the keyboard and remembers the search', async () => {
    stub();
    const { router } = panel('/');
    const input = screen.getByRole('combobox', { name: 'Search' });
    await userEvent.type(input, 'maven');
    await screen.findByRole('group', { name: 'Components' });
    await userEvent.keyboard('{ArrowDown}{ArrowDown}');
    expect(screen.getByRole('option', { name: /maven-central/ })).toHaveAttribute('aria-selected', 'true');
    await userEvent.keyboard('{Enter}');
    await waitFor(() => expect(router.state.location.pathname).toBe('/browse/maven-central'));
  });

  it('searches everywhere on Enter without a choice', async () => {
    stub();
    const { router } = panel('/');
    await userEvent.type(screen.getByRole('combobox', { name: 'Search' }), 'junit{Enter}');
    await waitFor(() => expect(router.state.location.search).toBe('?q=junit'));
    expect(JSON.parse(window.localStorage.getItem(RECENT_KEY) ?? '[]')).toEqual(['junit']);
  });

  it('stays away from callers who may not search', () => {
    stub();
    panel('/', { ...ANONYMOUS_STATE, permissions: [] });
    expect(screen.queryByRole('group', { name: 'Recent searches' })).toBeNull();
  });
});
