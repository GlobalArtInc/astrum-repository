import React from 'react';
import {render, screen, fireEvent, waitFor, within, act} from '@testing-library/react';
import '@testing-library/jest-dom';
import Axios from 'axios';

import SearchRadix, {isApplePlatform, isSearchShortcut} from './SearchRadix';
import {
  RECENT_SEARCHES_KEY,
  addRecentSearch,
  classicBrowseParam,
  classicSearchKeyword,
  currentRepositoryOf,
  highlightParts,
  matchRepositories,
  readRecentSearches,
} from './searchPanel';

const mockGo = jest.fn().mockResolvedValue(undefined);
const mockSearch = jest.fn();
let mockState = {name: 'browse.welcome'};
let mockParams = {};

jest.mock('@uirouter/react', () => ({
  useRouter: () => ({
    stateService: {go: mockGo},
    stateRegistry: {
      get: () => ({data: {visibilityRequirements: []}}),
    },
  }),
  useCurrentStateAndParams: () => ({
    state: mockState,
    params: mockParams,
  }),
}));

let mockIsVisible = true;

jest.mock('@globalart/astrum-ui-plugin', () => ({
  ...jest.requireActual('@globalart/astrum-ui-plugin'),
  ExtJS: {
    search: (...args) => mockSearch(...args),
    state: () => ({getValue: () => false}),
    showSuccessMessage: jest.fn(),
  },
  useIsVisible: () => mockIsVisible,
  handleExtJsUnsavedChanges: (_ctrl, fn) => fn(),
}));

jest.mock('axios', () => ({
  get: jest.fn().mockResolvedValue({data: []}),
}));

jest.mock('../../routerConfig/routeNames/routeNames', () => ({
  ROUTE_NAMES: {
    BROWSE: {
      BROWSE: {
        ROOT: 'browse.browse',
      },
      SEARCH: {
        ROOT: 'browse.search',
        GENERIC: 'browse.search.generic',
      },
    },
  },
}));

const PLACEHOLDER = 'Search components or CVEs...';

const REPOSITORIES = [
  {name: 'maven-releases', format: 'maven2', type: 'hosted'},
  {name: 'maven-central', format: 'maven2', type: 'proxy'},
  {name: 'maven-public', format: 'maven2', type: 'group'},
  {name: 'npm-proxy', format: 'npm', type: 'proxy'},
  {name: 'docker-hosted', format: 'docker', type: 'hosted'},
  {name: 'old-maven', format: 'maven2', type: 'hosted'},
  {name: 'raw-maven-mirror', format: 'raw', type: 'proxy'},
];

function component(name, extra = {}) {
  return {
    id: `id-${name}`,
    repository: 'maven-releases',
    format: 'maven2',
    group: 'org.example',
    name,
    version: '1.0.0',
    assets: [{path: `/org/example/${name}/1.0.0/${name}-1.0.0.jar`}],
    ...extra,
  };
}

let mockComponents = [];

function mockApi({repositories = REPOSITORIES, components = []} = {}) {
  mockComponents = components;
  Axios.get.mockImplementation((url, config) => {
    if (url === '/service/rest/v1/repositories') {
      return Promise.resolve({data: repositories});
    }
    if (url === '/service/rest/v1/search') {
      const q = config.params.q.toLowerCase();
      return Promise.resolve({data: {items: mockComponents.filter((item) => item.name.includes(q))}});
    }
    return Promise.resolve({data: []});
  });
}

function field() {
  return screen.getByPlaceholderText(PLACEHOLDER);
}

function panel() {
  return screen.queryByTestId('search-panel');
}

function optionTexts() {
  return screen.queryAllByRole('option').map((option) => option.textContent);
}

function findOption(text) {
  return waitFor(() => {
    const option = screen.getAllByRole('option').find((candidate) => candidate.textContent.includes(text));
    expect(option).toBeDefined();
    return option;
  });
}

async function type(value) {
  fireEvent.focus(field());
  fireEvent.change(field(), {target: {value}});
}

describe('SearchRadix', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    mockIsVisible = true;
    mockState = {name: 'browse.welcome'};
    mockParams = {};
    window.localStorage.clear();
    mockApi();
  });

  describe('rendering', () => {
    it('renders the search input', () => {
      render(<SearchRadix />);
      expect(field()).toBeInTheDocument();
      expect(panel()).not.toBeInTheDocument();
    });

    it('renders nothing when visibility check fails', () => {
      mockIsVisible = false;
      const {container} = render(<SearchRadix />);
      expect(container.firstChild).toBeNull();
    });
  });

  describe('Default UI navigation (bug rhv)', () => {
    it('navigates via router on Enter with query (not hardcoded hash)', () => {
      render(<SearchRadix isPreviewUI={false} />);

      fireEvent.change(field(), {target: {value: 'logback'}});
      fireEvent.keyDown(field(), {key: 'Enter'});

      expect(mockGo).toHaveBeenCalledWith(
        'browse.search.generic',
        {keyword: '=keyword=logback'}
      );
    });

    it('navigates via router on Enter with empty query (keyword null)', () => {
      render(<SearchRadix isPreviewUI={false} />);

      fireEvent.keyDown(field(), {key: 'Enter'});

      expect(mockGo).toHaveBeenCalledWith(
        'browse.search.generic',
        {keyword: null}
      );
    });

    it('searches in place when the search page is already open', () => {
      mockState = {name: 'browse.search.generic'};
      render(<SearchRadix isPreviewUI={false} />);

      fireEvent.change(field(), {target: {value: 'logback'}});
      fireEvent.keyDown(field(), {key: 'Enter'});

      expect(mockGo).not.toHaveBeenCalled();
      expect(mockSearch).toHaveBeenCalledWith('logback');
    });

    it('does not set window.location.hash directly', () => {
      const originalHash = window.location.hash;
      render(<SearchRadix isPreviewUI={false} />);

      fireEvent.keyDown(field(), {key: 'Enter'});

      expect(window.location.hash).toBe(originalHash);
    });
  });

  describe('Preview UI navigation', () => {
    it('navigates to unified search on Enter with query', () => {
      render(<SearchRadix isPreviewUI={true} />);

      fireEvent.change(field(), {target: {value: 'react'}});
      fireEvent.keyDown(field(), {key: 'Enter'});

      expect(mockGo).toHaveBeenCalledWith(
        'preview.browse.search.unified',
        {q: 'react'},
        {inherit: false, reload: 'preview.browse.search.unified'},
      );
    });

    it('navigates to unified search on Enter without query', () => {
      render(<SearchRadix isPreviewUI={true} />);

      fireEvent.keyDown(field(), {key: 'Enter'});

      expect(mockGo).toHaveBeenCalledWith(
        'preview.browse.search.unified',
        {},
        {inherit: false, reload: 'preview.browse.search.unified'},
      );
    });

    it('forces the transition when the term matches the router cache (AT-017)', () => {
      // inherit: false alone is not enough. The search page writes its URL with
      // raw pushState, so the router keeps a stale `q`; re-submitting that same
      // term produces a same-state/same-params transition, which UI-Router
      // rejects as 'SameAsCurrent' and the page never re-syncs. The reload
      // option names the state rather than passing `true`, which would resolve
      // to the registry root and remount the whole preview shell.
      render(<SearchRadix isPreviewUI={true} />);

      fireEvent.change(field(), {target: {value: 'spring'}});
      fireEvent.keyDown(field(), {key: 'Enter'});

      const [, , options] = mockGo.mock.calls.at(-1);
      expect(options.inherit).toBe(false);
      expect(options.reload).toBe('preview.browse.search.unified');
      expect(options.reload).not.toBe(true);
    });
  });

  describe('opening the panel', () => {
    it('opens under the field on focus, with a hint when there is no history', () => {
      render(<SearchRadix isPreviewUI={true} />);

      fireEvent.focus(field());

      expect(panel()).toHaveTextContent('Find components by name, group or version, or open a repository by its name.');
      expect(screen.queryByRole('listbox')).not.toBeInTheDocument();
    });

    it('opens on a click in the field after it was closed', () => {
      render(<SearchRadix isPreviewUI={true} />);
      fireEvent.focus(field());
      fireEvent.keyDown(field(), {key: 'Escape'});
      expect(panel()).not.toBeInTheDocument();

      fireEvent.mouseDown(field());

      expect(panel()).toBeInTheDocument();
    });

    it('opens on Ctrl+K', () => {
      Object.defineProperty(window.navigator, 'platform', {value: 'Win32', configurable: true});
      render(<SearchRadix isPreviewUI={true} />);

      fireEvent.keyDown(document.body, {key: 'k', ctrlKey: true});

      expect(field()).toHaveFocus();
      expect(panel()).toBeInTheDocument();
    });

    it('closes on Escape and on a click outside', () => {
      render(
        <div>
          <SearchRadix isPreviewUI={true} />
          <button type="button">Elsewhere</button>
        </div>
      );

      fireEvent.focus(field());
      fireEvent.keyDown(field(), {key: 'Escape'});
      expect(panel()).not.toBeInTheDocument();

      fireEvent.mouseDown(field());
      expect(panel()).toBeInTheDocument();
      fireEvent.mouseDown(screen.getByText('Elsewhere'));
      expect(panel()).not.toBeInTheDocument();
    });

    it('shows a clear button once the field has text', () => {
      render(<SearchRadix isPreviewUI={true} />);
      expect(screen.queryByRole('button', {name: 'Clear search'})).not.toBeInTheDocument();
      expect(screen.getByTestId('search-shortcut-hint')).toBeInTheDocument();

      fireEvent.change(field(), {target: {value: 'mav'}});
      expect(screen.queryByTestId('search-shortcut-hint')).not.toBeInTheDocument();
      fireEvent.click(screen.getByRole('button', {name: 'Clear search'}));

      expect(field()).toHaveValue('');
      expect(field()).toHaveFocus();
      expect(panel()).toBeInTheDocument();
    });

    it('sets the combobox roles', async () => {
      render(<SearchRadix isPreviewUI={true} />);
      const input = screen.getByRole('combobox', {name: 'Search components'});
      expect(input).toHaveAttribute('aria-expanded', 'false');

      await type('maven');

      const listbox = await screen.findByRole('listbox', {name: 'Search suggestions'});
      expect(input).toHaveAttribute('aria-expanded', 'true');
      expect(input).toHaveAttribute('aria-controls', listbox.id);
      expect(within(listbox).getAllByRole('group').length).toBeGreaterThan(0);
    });
  });

  describe('sections', () => {
    it('offers to search all repositories, outside a repository', async () => {
      render(<SearchRadix isPreviewUI={true} />);

      await type('zzz');

      const searchIn = screen.getByTestId('search-section-search-in');
      expect(searchIn).toHaveTextContent('Search for “zzz” in…');
      expect(within(searchIn).getAllByRole('option').map((option) => option.textContent)).toEqual(['all repositories']);
    });

    it('offers this repository first while browsing one in Astrum One', async () => {
      mockState = {name: 'preview.browse.browse.repo.path'};
      mockParams = {repoName: 'maven-releases', path: 'org/example'};
      render(<SearchRadix isPreviewUI={true} />);

      await type('zzz');

      const options = within(screen.getByTestId('search-section-search-in')).getAllByRole('option');
      expect(options.map((option) => option.textContent)).toEqual(['this repositorymaven-releases', 'all repositories']);

      fireEvent.click(options[0]);
      expect(mockGo).toHaveBeenCalledWith(
        'preview.browse.search.unified',
        {q: 'zzz', repository: 'maven-releases'},
        {inherit: false, reload: 'preview.browse.search.unified'},
      );
    });

    it('offers this repository while browsing one in the Classic UI', async () => {
      mockState = {name: 'browse.browse'};
      mockParams = {repo: ':maven-releases:org%2Fexample'};
      render(<SearchRadix isPreviewUI={false} />);

      await type('zzz');
      fireEvent.click(screen.getByRole('option', {name: /this repository/}));

      expect(mockGo).toHaveBeenCalledWith('browse.search.generic', {
        keyword: '=keyword=zzz%20AND%20repository_name=maven-releases',
      });
    });

    it('lists up to 5 repositories whose name contains the query, with the match in bold', async () => {
      render(<SearchRadix isPreviewUI={true} />);

      await type('MAVEN');

      const section = await screen.findByTestId('search-section-repositories');
      const options = within(section).getAllByRole('option');
      expect(section).toHaveTextContent('Repositories');
      expect(options).toHaveLength(5);
      expect(options[0]).toHaveTextContent('maven-central');
      expect(options[0]).toHaveTextContent('Maven');
      expect(options[0]).toHaveTextContent('Proxy');
      expect(options[0].querySelector('strong')).toHaveTextContent('maven');
      expect(section).not.toHaveTextContent('npm-proxy');
    });

    it('lists up to 6 components found by the search API, after a pause in typing', async () => {
      mockApi({components: ['spring-a', 'spring-b', 'spring-c', 'spring-d', 'spring-e', 'spring-f', 'spring-g'].map((name) => component(name))});
      render(<SearchRadix isPreviewUI={true} />);

      await type('spring');

      const section = await screen.findByTestId('search-section-components');
      const options = within(section).getAllByRole('option');
      expect(section).toHaveTextContent('Components');
      expect(options).toHaveLength(6);
      expect(options[0]).toHaveTextContent('spring-a');
      expect(options[0]).toHaveTextContent('1.0.0');
      expect(options[0]).toHaveTextContent('org.example');
      expect(options[0]).toHaveTextContent('maven-releases');
      expect(options[0]).toHaveTextContent('Maven');
      expect(options[0].querySelector('strong')).toHaveTextContent('spring');

      const searches = Axios.get.mock.calls.filter(([url]) => url === '/service/rest/v1/search');
      expect(searches).toHaveLength(1);
      expect(searches[0][1].params).toEqual({q: 'spring'});
    });

    it('debounces the component search and cancels a request still in flight', async () => {
      jest.useFakeTimers();
      try {
        render(<SearchRadix isPreviewUI={true} />);
        fireEvent.focus(field());

        fireEvent.change(field(), {target: {value: 'sp'}});
        fireEvent.change(field(), {target: {value: 'spr'}});
        act(() => {
          jest.advanceTimersByTime(150);
        });
        expect(Axios.get.mock.calls.filter(([url]) => url === '/service/rest/v1/search')).toHaveLength(0);

        fireEvent.change(field(), {target: {value: 'spri'}});
        act(() => {
          jest.advanceTimersByTime(250);
        });
        let searches = Axios.get.mock.calls.filter(([url]) => url === '/service/rest/v1/search');
        expect(searches).toHaveLength(1);
        expect(searches[0][1].params).toEqual({q: 'spri'});
        const firstSignal = searches[0][1].signal;

        fireEvent.change(field(), {target: {value: 'sprin'}});
        expect(firstSignal.aborted).toBe(true);
        act(() => {
          jest.advanceTimersByTime(250);
        });
        searches = Axios.get.mock.calls.filter(([url]) => url === '/service/rest/v1/search');
        expect(searches).toHaveLength(2);
      } finally {
        jest.useRealTimers();
      }
    });

    it('says when nothing matches', async () => {
      render(<SearchRadix isPreviewUI={true} />);

      await type('nothing-like-this');

      expect(await screen.findByText('No results')).toBeInTheDocument();
      expect(screen.queryByTestId('search-section-repositories')).not.toBeInTheDocument();
      expect(screen.queryByTestId('search-section-components')).not.toBeInTheDocument();
    });

    it('keeps the sections in order', async () => {
      mockApi({components: [component('maven-plugin')]});
      render(<SearchRadix isPreviewUI={true} />);

      await type('maven');

      await screen.findByTestId('search-section-components');
      expect(screen.queryByText('No results')).not.toBeInTheDocument();
      const sections = [...panel().querySelectorAll('[data-testid^="search-section-"]')].map((section) => section.dataset.testid);
      expect(sections).toEqual(['search-section-search-in', 'search-section-repositories', 'search-section-components']);
    });
  });

  describe('opening a row', () => {
    it('opens a repository in the Astrum One browse page', async () => {
      render(<SearchRadix isPreviewUI={true} />);

      await type('docker');
      fireEvent.click(await findOption('docker-hosted'));

      await waitFor(() => expect(mockGo).toHaveBeenCalledWith('preview.browse.browse.repo', {repoName: 'docker-hosted'}));
      expect(field()).toHaveValue('');
      expect(panel()).not.toBeInTheDocument();
    });

    it('opens a component at its path in the Astrum One browse page', async () => {
      mockApi({components: [component('spring-core')]});
      render(<SearchRadix isPreviewUI={true} />);

      await type('spring');
      fireEvent.click(await findOption('spring-core'));

      await waitFor(() => expect(mockGo).toHaveBeenCalledWith('preview.browse.browse.repo.path', {
        repoName: 'maven-releases',
        path: 'org/example/spring-core/1.0.0/spring-core-1.0.0.jar',
        tab: 'summary',
      }));
    });

    it('opens repositories and components in the Classic browse route', async () => {
      mockApi({components: [component('spring-core')]});
      render(<SearchRadix isPreviewUI={false} />);

      await type('docker');
      fireEvent.click(await findOption('docker-hosted'));
      expect(mockGo).toHaveBeenCalledWith('browse.browse', {repo: ':docker-hosted'});

      await type('spring');
      fireEvent.click(await findOption('spring-core'));
      expect(mockGo).toHaveBeenCalledWith('browse.browse', {
        repo: ':maven-releases:org%2Fexample%2Fspring-core%2F1.0.0%2Fspring-core-1.0.0.jar',
      });
    });
  });

  describe('keyboard', () => {
    it('moves through every row across sections, wrapping, and opens the active one on Enter', async () => {
      mockApi({components: [component('maven-plugin')]});
      render(<SearchRadix isPreviewUI={true} />);

      await type('maven-p');
      await screen.findByTestId('search-section-components');
      expect(optionTexts()).toHaveLength(3);
      const input = field();
      expect(input).not.toHaveAttribute('aria-activedescendant');

      fireEvent.keyDown(input, {key: 'ArrowDown'});
      expect(input).toHaveAttribute('aria-activedescendant', screen.getAllByRole('option')[0].id);
      expect(screen.getAllByRole('option')[0]).toHaveAttribute('aria-selected', 'true');
      expect(screen.getAllByRole('option')[0]).toHaveTextContent('↵');

      fireEvent.keyDown(input, {key: 'ArrowDown'});
      fireEvent.keyDown(input, {key: 'ArrowDown'});
      expect(input).toHaveAttribute('aria-activedescendant', screen.getAllByRole('option')[2].id);

      fireEvent.keyDown(input, {key: 'ArrowDown'});
      expect(input).toHaveAttribute('aria-activedescendant', screen.getAllByRole('option')[0].id);

      fireEvent.keyDown(input, {key: 'ArrowUp'});
      expect(input).toHaveAttribute('aria-activedescendant', screen.getAllByRole('option')[2].id);

      fireEvent.keyDown(input, {key: 'ArrowUp'});
      fireEvent.keyDown(input, {key: 'Enter'});

      await waitFor(() => expect(mockGo).toHaveBeenCalledWith('preview.browse.browse.repo', {repoName: 'maven-public'}));
    });

    it('starts from the last row on ArrowUp', async () => {
      render(<SearchRadix isPreviewUI={true} />);

      await type('npm');
      await screen.findByTestId('search-section-repositories');
      fireEvent.keyDown(field(), {key: 'ArrowUp'});

      const options = screen.getAllByRole('option');
      expect(field()).toHaveAttribute('aria-activedescendant', options[options.length - 1].id);
    });

    it('searches all repositories on Enter when no row is active', async () => {
      render(<SearchRadix isPreviewUI={true} />);

      await type('maven');
      fireEvent.keyDown(field(), {key: 'Enter'});

      expect(mockGo).toHaveBeenCalledWith(
        'preview.browse.search.unified',
        {q: 'maven'},
        {inherit: false, reload: 'preview.browse.search.unified'},
      );
      expect(panel()).not.toBeInTheDocument();
    });
  });

  describe('recent searches', () => {
    it('keeps the queries the user runs, newest first', async () => {
      render(<SearchRadix isPreviewUI={true} />);

      await type('first');
      fireEvent.keyDown(field(), {key: 'Enter'});
      fireEvent.change(field(), {target: {value: 'second'}});
      fireEvent.click(screen.getByRole('option', {name: 'all repositories'}));
      fireEvent.change(field(), {target: {value: ''}});

      const section = screen.getByTestId('search-section-recent');
      expect(section).toHaveTextContent('Recent searches');
      expect(within(section).getAllByRole('option').map((option) => option.textContent)).toEqual(['second', 'first']);
      expect(JSON.parse(window.localStorage.getItem(RECENT_SEARCHES_KEY))).toEqual(['second', 'first']);
    });

    it('does not keep repositories or components the user opens', async () => {
      render(<SearchRadix isPreviewUI={true} />);

      await type('docker');
      fireEvent.click(await findOption('docker-hosted'));

      expect(readRecentSearches()).toEqual([]);
    });

    it('shows the last 5 on an empty query and runs one when opened', () => {
      ['one', 'two', 'three', 'four', 'five', 'six'].forEach(addRecentSearch);
      render(<SearchRadix isPreviewUI={true} />);

      fireEvent.focus(field());

      expect(optionTexts()).toEqual(['six', 'five', 'four', 'three', 'two']);
      fireEvent.click(screen.getAllByRole('option')[2]);
      expect(mockGo).toHaveBeenCalledWith(
        'preview.browse.search.unified',
        {q: 'four'},
        {inherit: false, reload: 'preview.browse.search.unified'},
      );
      expect(readRecentSearches()[0]).toBe('four');
    });

    it('removes one with its × button', () => {
      ['one', 'two'].forEach(addRecentSearch);
      render(<SearchRadix isPreviewUI={true} />);

      fireEvent.focus(field());
      fireEvent.click(screen.getByRole('button', {name: 'Remove “two” from recent searches'}));

      expect(optionTexts()).toEqual(['one']);
      expect(readRecentSearches()).toEqual(['one']);
      expect(mockGo).not.toHaveBeenCalled();
    });

    it('reads unusable storage as no history', () => {
      window.localStorage.setItem(RECENT_SEARCHES_KEY, '{not json');
      expect(readRecentSearches()).toEqual([]);
      window.localStorage.setItem(RECENT_SEARCHES_KEY, JSON.stringify({a: 1}));
      expect(readRecentSearches()).toEqual([]);
    });

    it('keeps one entry per query, whatever its case', () => {
      addRecentSearch('Maven');
      addRecentSearch('npm');
      addRecentSearch('maven');
      expect(readRecentSearches()).toEqual(['maven', 'npm']);
    });
  });

  describe('helpers', () => {
    it('splits a text around the matches of the query', () => {
      expect(highlightParts('Maven-maven', 'MAV')).toEqual([
        {text: 'Mav', match: true},
        {text: 'en-', match: false},
        {text: 'mav', match: true},
        {text: 'en', match: false},
      ]);
      expect(highlightParts('a.b(c)', '(c')).toEqual([{text: 'a.b', match: false}, {text: '(c', match: true}, {text: ')', match: false}]);
      expect(highlightParts('text', ' ')).toEqual([{text: 'text', match: false}]);
    });

    it('puts the repositories that start with the query first', () => {
      expect(matchRepositories(REPOSITORIES, 'maven').map((repository) => repository.name))
        .toEqual(['maven-central', 'maven-public', 'maven-releases', 'old-maven', 'raw-maven-mirror']);
      expect(matchRepositories(REPOSITORIES, '')).toEqual([]);
    });

    it('finds the repository the page browses', () => {
      expect(currentRepositoryOf('preview.browse.browse.repo', {repoName: 'npm-proxy'})).toBe('npm-proxy');
      expect(currentRepositoryOf('preview.browse.browse.repo.path.summary', {repoName: 'npm-proxy'})).toBe('npm-proxy');
      expect(currentRepositoryOf('preview.browse.browse', {})).toBeNull();
      expect(currentRepositoryOf('browse.browse', {repo: ':my%20repo:a%2Fb'})).toBe('my repo');
      expect(currentRepositoryOf('browse.browse', {repo: null})).toBeNull();
      expect(currentRepositoryOf('browse.welcome', {})).toBeNull();
    });

    it('builds the Classic search and browse parameters', () => {
      expect(classicSearchKeyword('', null)).toBeNull();
      expect(classicSearchKeyword('a b', null)).toBe('=keyword=a%20b');
      expect(classicSearchKeyword('a', 'maven-releases')).toBe('=keyword=a%20AND%20repository_name=maven-releases');
      expect(classicBrowseParam('maven-releases')).toBe(':maven-releases');
      expect(classicBrowseParam('maven-releases', '/a/b.jar')).toBe(':maven-releases:a%2Fb.jar');
    });
  });

  describe('keyboard shortcut', () => {
    const originalPlatform = Object.getOwnPropertyDescriptor(window.navigator, 'platform');

    function setPlatform(platform) {
      Object.defineProperty(window.navigator, 'platform', {value: platform, configurable: true});
    }

    afterEach(() => {
      if (originalPlatform) {
        Object.defineProperty(window.navigator, 'platform', originalPlatform);
      } else {
        delete window.navigator.platform;
      }
    });

    it('labels the field and announces the shortcut', () => {
      setPlatform('Linux x86_64');
      render(<SearchRadix />);

      const input = screen.getByRole('combobox', {name: 'Search components'});
      expect(input).toHaveAttribute('aria-keyshortcuts', 'Control+K');
      expect(screen.getByTestId('search-shortcut-hint')).toHaveTextContent('Ctrl K');
      expect(screen.getByRole('search')).toContainElement(input);
    });

    it('focuses the search field on Ctrl+K', () => {
      setPlatform('Win32');
      render(<SearchRadix />);
      const input = field();
      expect(input).not.toHaveFocus();

      const notPrevented = fireEvent.keyDown(document.body, {key: 'k', ctrlKey: true});

      expect(input).toHaveFocus();
      expect(notPrevented).toBe(false);
    });

    it('selects what the field already holds so typing replaces it', () => {
      setPlatform('Win32');
      render(<SearchRadix />);
      const input = field();
      fireEvent.change(input, {target: {value: 'maven'}});
      input.blur();

      fireEvent.keyDown(document.body, {key: 'K', ctrlKey: true});

      expect(input).toHaveFocus();
      expect(input.selectionStart).toBe(0);
      expect(input.selectionEnd).toBe('maven'.length);
    });

    it('uses Command+K and shows the Command key on Apple platforms', () => {
      setPlatform('MacIntel');
      render(<SearchRadix />);
      const input = field();

      expect(screen.getByTestId('search-shortcut-hint')).toHaveTextContent('⌘ K');
      expect(input).toHaveAttribute('aria-keyshortcuts', 'Meta+K');

      fireEvent.keyDown(document.body, {key: 'k', ctrlKey: true});
      expect(input).not.toHaveFocus();

      fireEvent.keyDown(document.body, {key: 'k', metaKey: true});
      expect(input).toHaveFocus();
    });

    it('leaves other keys and modifier combinations alone', () => {
      setPlatform('Linux x86_64');
      render(<SearchRadix />);
      const input = field();

      expect(fireEvent.keyDown(document.body, {key: 'k'})).toBe(true);
      expect(fireEvent.keyDown(document.body, {key: 'j', ctrlKey: true})).toBe(true);
      expect(fireEvent.keyDown(document.body, {key: 'k', ctrlKey: true, shiftKey: true})).toBe(true);
      expect(fireEvent.keyDown(document.body, {key: 'k', ctrlKey: true, altKey: true})).toBe(true);
      expect(input).not.toHaveFocus();
    });

    it('stops listening once the header is gone', () => {
      setPlatform('Linux x86_64');
      const {unmount} = render(<SearchRadix />);
      unmount();

      expect(fireEvent.keyDown(document.body, {key: 'k', ctrlKey: true})).toBe(true);
    });

    it('recognises the platform and the shortcut', () => {
      setPlatform('MacIntel');
      expect(isApplePlatform()).toBe(true);
      setPlatform('Linux x86_64');
      expect(isApplePlatform()).toBe(false);

      expect(isSearchShortcut({key: 'k', ctrlKey: true}, false)).toBe(true);
      expect(isSearchShortcut({key: 'k', metaKey: true}, false)).toBe(false);
      expect(isSearchShortcut({key: 'k', metaKey: true}, true)).toBe(true);
      expect(isSearchShortcut({key: 'k', ctrlKey: true}, true)).toBe(false);
    });
  });
});
