

import React from 'react';
import {render, screen, fireEvent, waitFor} from '@testing-library/react';
import '@testing-library/jest-dom';
import Axios from 'axios';

import SearchRadix, {isApplePlatform, isSearchShortcut} from './SearchRadix';

const mockGo = jest.fn().mockResolvedValue(undefined);
const mockSearch = jest.fn();

jest.mock('@uirouter/react', () => ({
  useRouter: () => ({
    stateService: {go: mockGo},
    stateRegistry: {
      get: () => ({data: {visibilityRequirements: []}}),
    },
  }),
  useCurrentStateAndParams: () => ({
    state: {name: 'browse.welcome'},
    params: {},
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
      SEARCH: {
        ROOT: 'browse.search',
        GENERIC: 'browse.search.generic',
      },
    },
  },
}));

describe('SearchRadix', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    mockIsVisible = true;
  });

  describe('rendering', () => {
    it('renders the search input', () => {
      render(<SearchRadix />);
      expect(screen.getByPlaceholderText('Search components or CVEs...')).toBeInTheDocument();
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

      const input = screen.getByPlaceholderText('Search components or CVEs...');
      fireEvent.change(input, {target: {value: 'logback'}});
      fireEvent.keyDown(input, {key: 'Enter'});

      expect(mockGo).toHaveBeenCalledWith(
        'browse.search.generic',
        {keyword: '=keyword=logback'}
      );
    });

    it('navigates via router on Enter with empty query (keyword null)', () => {
      render(<SearchRadix isPreviewUI={false} />);

      const input = screen.getByPlaceholderText('Search components or CVEs...');
      fireEvent.keyDown(input, {key: 'Enter'});

      expect(mockGo).toHaveBeenCalledWith(
        'browse.search.generic',
        {keyword: null}
      );
    });

    it('does not set window.location.hash directly', () => {
      const originalHash = window.location.hash;
      render(<SearchRadix isPreviewUI={false} />);

      const input = screen.getByPlaceholderText('Search components or CVEs...');
      fireEvent.keyDown(input, {key: 'Enter'});

      expect(window.location.hash).toBe(originalHash);
    });
  });

  describe('Preview UI navigation', () => {
    it('navigates to unified search on Enter with query', () => {
      render(<SearchRadix isPreviewUI={true} />);

      const input = screen.getByPlaceholderText('Search components or CVEs...');
      fireEvent.change(input, {target: {value: 'react'}});
      fireEvent.keyDown(input, {key: 'Enter'});

      expect(mockGo).toHaveBeenCalledWith(
        'preview.browse.search.unified',
        {q: 'react'},
        {inherit: false, reload: 'preview.browse.search.unified'},
      );
    });

    it('navigates to unified search on Enter without query', () => {
      render(<SearchRadix isPreviewUI={true} />);

      const input = screen.getByPlaceholderText('Search components or CVEs...');
      fireEvent.keyDown(input, {key: 'Enter'});

      expect(mockGo).toHaveBeenCalledWith(
        'preview.browse.search.unified',
        {},
        {inherit: false, reload: 'preview.browse.search.unified'},
      );
    });

    it('navigates without inheriting stale router params (AT-017)', () => {
      render(<SearchRadix isPreviewUI={true} />);

      const input = screen.getByPlaceholderText('Search components or CVEs...');
      fireEvent.change(input, {target: {value: 'shared02'}});
      fireEvent.keyDown(input, {key: 'Enter'});

      expect(mockGo).toHaveBeenCalledWith(
        'preview.browse.search.unified',
        {q: 'shared02'},
        {inherit: false, reload: 'preview.browse.search.unified'},
      );
    });

    it('resets params on an empty search (AT-017)', () => {
      render(<SearchRadix isPreviewUI={true} />);

      const input = screen.getByPlaceholderText('Search components or CVEs...');
      fireEvent.keyDown(input, {key: 'Enter'});

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

      const input = screen.getByPlaceholderText('Search components or CVEs...');
      fireEvent.change(input, {target: {value: 'spring'}});
      fireEvent.keyDown(input, {key: 'Enter'});

      const [, , options] = mockGo.mock.calls.at(-1);
      expect(options.reload).toBe('preview.browse.search.unified');
      expect(options.reload).not.toBe(true);
    });
  });

  describe('input behavior', () => {
    it('updates input value on typing', () => {
      render(<SearchRadix />);

      const input = screen.getByPlaceholderText('Search components or CVEs...');
      fireEvent.change(input, {target: {value: 'test-input'}});

      expect(input).toHaveValue('test-input');
    });
  });

  describe('autocomplete behavior', () => {
    it('navigates to component detail when a suggestion is clicked in Preview UI', async () => {
      const mockSuggestions = [
        { name: 'test-comp', format: 'maven2', group: 'org.test', version: '1.0', repository: 'maven-releases' }
      ];
      Axios.get.mockResolvedValue({ data: mockSuggestions });

      render(<SearchRadix isPreviewUI={true} />);

      const input = screen.getByPlaceholderText('Search components or CVEs...');
      fireEvent.change(input, { target: { value: 'test' } });

      // Wait for suggestions to appear
      await waitFor(() => {
        expect(screen.getByTestId('search-suggestion-0')).toBeInTheDocument();
      });

      // Simulate mousedown (triggers selection - navigation is deferred via queueMicrotask)
      fireEvent.mouseDown(screen.getByTestId('search-suggestion-0'));

      await waitFor(() => {
        expect(mockGo).toHaveBeenCalledWith('preview.browse.search.component', {
          gaId: 'maven2:org.test:test-comp',
          keyword: 'test-comp',
          version: '1.0'
        });
      });
    });

    it('navigates to component detail when Enter is pressed on a selected suggestion', async () => {
      const mockSuggestions = [
        { name: 'test-comp', format: 'maven2', group: 'org.test', version: '1.0', repository: 'maven-releases' }
      ];
      Axios.get.mockResolvedValue({ data: mockSuggestions });

      render(<SearchRadix isPreviewUI={true} />);

      const input = screen.getByPlaceholderText('Search components or CVEs...');
      fireEvent.change(input, { target: { value: 'test' } });

      await waitFor(() => {
        expect(screen.getByTestId('search-suggestion-0')).toBeInTheDocument();
      });

      // Press ArrowDown to select the first suggestion
      fireEvent.keyDown(input, { key: 'ArrowDown' });
      
      // Press Enter to navigate (navigation is deferred via queueMicrotask)
      fireEvent.keyDown(input, { key: 'Enter' });

      await waitFor(() => {
        expect(mockGo).toHaveBeenCalledWith('preview.browse.search.component', {
          gaId: 'maven2:org.test:test-comp',
          keyword: 'test-comp',
          version: '1.0'
        });
      });
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

      const input = screen.getByRole('textbox', {name: 'Search components'});
      expect(input).toHaveAttribute('aria-keyshortcuts', 'Control+K');
      expect(screen.getByTestId('search-shortcut-hint')).toHaveTextContent('Ctrl K');
      expect(screen.getByRole('search')).toContainElement(input);
    });

    it('focuses the search field on Ctrl+K', () => {
      setPlatform('Win32');
      render(<SearchRadix />);
      const input = screen.getByPlaceholderText('Search components or CVEs...');
      expect(input).not.toHaveFocus();

      const notPrevented = fireEvent.keyDown(document.body, {key: 'k', ctrlKey: true});

      expect(input).toHaveFocus();
      expect(notPrevented).toBe(false);
    });

    it('selects what the field already holds so typing replaces it', () => {
      setPlatform('Win32');
      render(<SearchRadix />);
      const input = screen.getByPlaceholderText('Search components or CVEs...');
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
      const input = screen.getByPlaceholderText('Search components or CVEs...');

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
      const input = screen.getByPlaceholderText('Search components or CVEs...');

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
