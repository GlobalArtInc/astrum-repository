import React from 'react';
import { act, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { Theme } from '@radix-ui/themes';
import '@testing-library/jest-dom';
import Axios from 'axios';

import { UnifiedSearchPage, hasSearchCriteria } from '../UnifiedSearchPage';

const mockGo = jest.fn();
jest.mock('@uirouter/react', () => ({
  useRouter: () => ({ stateService: { go: mockGo } }),
}));

jest.mock('axios', () => {
  const get = jest.fn();
  return { __esModule: true, default: { get }, get };
});

const mockGet = Axios.get as jest.Mock;

const REPOSITORIES = [
  { name: 'docker-hosted', format: 'docker', type: 'hosted' },
  { name: 'maven-releases', format: 'maven2', type: 'hosted' },
];

const ITEMS = [
  {
    id: 'c1',
    repository: 'docker-hosted',
    format: 'docker',
    group: null,
    name: 'herald-api',
    version: '0.1.0',
    assets: [{ id: 'a1', path: '/v2/herald-api/manifests/0.1.0', downloadUrl: 'x', lastModified: '2026-10-09T22:28:00Z' }],
  },
  {
    id: 'c2',
    repository: 'maven-releases',
    format: 'maven2',
    group: 'dev.globalart.herald',
    name: 'herald-core',
    version: '1.3.0',
    assets: [{ id: 'a2', path: 'dev/globalart/herald/herald-core/1.3.0/herald-core-1.3.0.jar', downloadUrl: 'y' }],
  },
];

function respond(items: unknown[] | Error) {
  mockGet.mockImplementation((url: string) => {
    if (url === '/service/rest/v1/repositories') {
      return Promise.resolve({ data: REPOSITORIES });
    }
    if (items instanceof Error) {
      return Promise.reject(items);
    }
    return Promise.resolve({ data: { items, continuationToken: undefined } });
  });
}

function searchCalls(): string[] {
  return mockGet.mock.calls.map(([url]) => url as string).filter((url) => url.startsWith('/service/rest/v1/search'));
}

function renderPage() {
  return render(
    <Theme>
      <UnifiedSearchPage />
    </Theme>,
  );
}

describe('UnifiedSearchPage', () => {
  beforeEach(() => {
    mockGo.mockReset();
    mockGet.mockReset();
    window.location.hash = '#preview/browse/search';
    respond(ITEMS);
  });

  it('invites a search before anything is typed', async () => {
    renderPage();

    expect(screen.getByRole('heading', { level: 1, name: 'Search' })).toBeInTheDocument();
    expect(screen.getByText('Search all repositories')).toBeInTheDocument();
    await waitFor(() => expect(mockGet).toHaveBeenCalledWith('/service/rest/v1/repositories'));
    expect(searchCalls()).toEqual([]);
  });

  it('searches as the user types and lists the components with their format', async () => {
    renderPage();

    fireEvent.change(screen.getByRole('textbox', { name: 'Search components' }), { target: { value: 'herald' } });

    const table = await screen.findByRole('table');
    expect(searchCalls()[0]).toContain('q=herald');
    expect(window.location.hash).toContain('q=herald');
    const rows = within(table).getAllByRole('button', { name: /in its repository/ });
    expect(rows).toHaveLength(2);
    expect(within(rows[0]).getByText('herald-api')).toBeInTheDocument();
    expect(within(rows[0]).getByText('0.1.0')).toBeInTheDocument();
    expect(within(rows[0]).getByTestId('format-badge')).toHaveTextContent('docker');
    expect(within(rows[1]).getByText('dev.globalart.herald')).toBeInTheDocument();
    expect(screen.getByText('Shown: 2')).toBeInTheDocument();
  });

  it('does not search for a single character', async () => {
    renderPage();

    fireEvent.change(screen.getByRole('textbox', { name: 'Search components' }), { target: { value: 'h' } });
    await act(async () => {
      await new Promise((resolve) => setTimeout(resolve, 400));
    });

    expect(searchCalls()).toEqual([]);
    expect(screen.getByText('Search all repositories')).toBeInTheDocument();
  });

  it('says when nothing was found', async () => {
    respond([]);
    renderPage();

    fireEvent.change(screen.getByRole('textbox', { name: 'Search components' }), { target: { value: 'nothing' } });

    expect(await screen.findByText('Nothing found')).toBeInTheDocument();
  });

  it('shows a failed search with a way to try again', async () => {
    respond(new Error('Server error'));
    renderPage();

    fireEvent.change(screen.getByRole('textbox', { name: 'Search components' }), { target: { value: 'herald' } });

    expect(await screen.findByText('The search failed')).toBeInTheDocument();
    respond(ITEMS);
    fireEvent.click(screen.getByRole('button', { name: 'Try again' }));
    expect(await screen.findByRole('table')).toBeInTheDocument();
  });

  it('opens a result in its repository at the path of its file', async () => {
    renderPage();
    fireEvent.change(screen.getByRole('textbox', { name: 'Search components' }), { target: { value: 'herald' } });

    fireEvent.click(await screen.findByRole('button', { name: 'Open herald-api in its repository' }));

    expect(mockGo).toHaveBeenCalledWith('preview.browse.browse.repo.path', {
      repoName: 'docker-hosted',
      path: 'v2/herald-api/manifests/0.1.0',
      tab: 'summary',
    });
  });

  it('opens the versions of a result in all repositories', async () => {
    renderPage();
    fireEvent.change(screen.getByRole('textbox', { name: 'Search components' }), { target: { value: 'herald' } });

    fireEvent.click(await screen.findByRole('button', { name: 'Versions of herald-core in all repositories' }));

    expect(mockGo).toHaveBeenCalledWith(
      'preview.browse.search.component',
      expect.objectContaining({ gaId: 'maven2:dev.globalart.herald:herald-core', version: '1.3.0' }),
    );
  });

  it('restores the search of the URL', async () => {
    window.location.hash = '#preview/browse/search?q=herald';
    renderPage();

    expect(screen.getByRole('textbox', { name: 'Search components' })).toHaveValue('herald');
    expect(await screen.findByRole('table')).toBeInTheDocument();
    expect(searchCalls()[0]).toContain('q=herald');
  });

  it('clears the search', async () => {
    renderPage();
    const field = screen.getByRole('textbox', { name: 'Search components' });
    fireEvent.change(field, { target: { value: 'herald' } });
    await screen.findByRole('table');

    fireEvent.click(screen.getByRole('button', { name: 'Clear the search' }));

    expect(field).toHaveValue('');
    expect(window.location.hash).not.toContain('q=');
  });
});

describe('hasSearchCriteria', () => {
  it('needs two characters, a format or a repository', () => {
    expect(hasSearchCriteria('', 'all', undefined)).toBe(false);
    expect(hasSearchCriteria(' h ', 'all', undefined)).toBe(false);
    expect(hasSearchCriteria('he', 'all', undefined)).toBe(true);
    expect(hasSearchCriteria('', 'docker', undefined)).toBe(true);
    expect(hasSearchCriteria('', 'all', 'npm-hosted')).toBe(true);
  });
});
