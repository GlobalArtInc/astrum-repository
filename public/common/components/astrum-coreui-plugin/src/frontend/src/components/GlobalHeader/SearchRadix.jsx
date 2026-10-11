import React, {useState, useEffect, useRef, useCallback, useId, useMemo} from 'react';
import {useCurrentStateAndParams, useRouter} from '@uirouter/react';
import {Badge, IconButton, TextField} from '@radix-ui/themes';
import {Database, FolderSearch, History, Package, Search as SearchIcon, X} from 'lucide-react';
import {ExtJS, FORMAT_LABELS, TYPE_LABELS, useIsVisible, handleExtJsUnsavedChanges} from '@globalart/astrum-ui-plugin';
import {ROUTE_NAMES} from '../../routerConfig/routeNames/routeNames';
import Axios from 'axios';

import './SearchRadix.scss';
import UIStrings from '../../constants/UIStrings';
import {
  MAX_COMPONENTS,
  addRecentSearch,
  classicBrowseParam,
  classicSearchKeyword,
  currentRepositoryOf,
  highlightParts,
  matchRepositories,
  readRecentSearches,
  removeRecentSearch,
  toComponentResult,
} from './searchPanel';

const DEBOUNCE_MS = 200;
const MIN_QUERY_LENGTH = 2;
const UNIFIED_SEARCH = 'preview.browse.search.unified';
const {HEADER} = UIStrings.SHELL;
const PANEL = HEADER.SEARCH_PANEL;

/**
 * Whether the browser runs on an Apple platform, where shortcuts use the Command key instead of Control.
 */
export function isApplePlatform() {
  if (typeof navigator === 'undefined') {
    return false;
  }
  const platform = navigator.userAgentData?.platform || navigator.platform || '';
  return /mac|iphone|ipad|ipod/i.test(platform);
}

/**
 * Whether a keydown is the shortcut that focuses the search: Ctrl+K, or Command+K on Apple platforms.
 */
export function isSearchShortcut(event, apple = isApplePlatform()) {
  const modifier = apple ? event.metaKey && !event.ctrlKey : event.ctrlKey && !event.metaKey;
  return Boolean(modifier && !event.altKey && !event.shiftKey && event.key?.toLowerCase() === 'k');
}

function isCancelled(error) {
  return error?.name === 'CanceledError' || error?.name === 'AbortError' || error?.code === 'ERR_CANCELED';
}

function Highlighted({text, query}) {
  return highlightParts(text, query).map((part, index) => (part.match
    ? <strong key={index} className="search-radix-match">{part.text}</strong>
    : <React.Fragment key={index}>{part.text}</React.Fragment>));
}

/**
 * Global search of the header, in the manner of GitLab: the field opens a panel right under it with the recent
 * searches, or, while the user types, where to search, the repositories whose name matches and the first
 * components found.
 *
 * Routing behavior:
 * - Astrum One: the search page `#preview/browse/search?q=<query>`, repositories and components in its browse page
 * - Classic UI: the search route `#browse/search/generic=keyword=<query>`, repositories and components in its
 *   browse route
 */
export default function SearchRadix({ isPreviewUI = false }) {
  const [searchValue, setSearchValue] = useState('');
  const [open, setOpen] = useState(false);
  const [activeIndex, setActiveIndex] = useState(-1);
  const [recentSearches, setRecentSearches] = useState(readRecentSearches);
  const [repositories, setRepositories] = useState([]);
  const [components, setComponents] = useState({query: '', items: [], loading: false});
  const [panelTop, setPanelTop] = useState(null);

  const containerRef = useRef(null);
  const fieldRef = useRef(null);
  const [isApple] = useState(isApplePlatform);
  const listboxId = useId();

  const router = useRouter();
  const {state, params} = useCurrentStateAndParams();
  const currentRepository = currentRepositoryOf(state?.name, params || {});

  // Check visibility for Default UI search route
  const searchRoute = router.stateRegistry.get(ROUTE_NAMES.BROWSE.SEARCH.GENERIC);
  const defaultUIVisible = useIsVisible(searchRoute?.data?.visibilityRequirements);

  const query = searchValue.trim();

  // The repositories the user can see, loaded again each time the panel opens so a new one shows up
  useEffect(() => {
    if (!open) {
      return undefined;
    }
    const controller = new AbortController();
    Axios.get('/service/rest/v1/repositories', {signal: controller.signal})
      .then((response) => setRepositories(Array.isArray(response?.data) ? response.data : []))
      .catch((error) => {
        if (!isCancelled(error)) {
          setRepositories([]);
        }
      });
    return () => controller.abort();
  }, [open]);

  // The first components found, debounced; a newer query cancels the request still in flight
  useEffect(() => {
    if (!open || query.length < MIN_QUERY_LENGTH) {
      setComponents({query: '', items: [], loading: false});
      return undefined;
    }

    setComponents((previous) => ({...previous, loading: true}));
    const controller = new AbortController();
    const timer = setTimeout(() => {
      Axios.get('/service/rest/v1/search', {params: {q: query}, signal: controller.signal})
        .then((response) => {
          const items = response?.data?.items || [];
          setComponents({query, items: items.slice(0, MAX_COMPONENTS).map(toComponentResult), loading: false});
        })
        .catch((error) => {
          if (!isCancelled(error)) {
            console.error('[SearchRadix] Failed to search components:', error);
            setComponents({query, items: [], loading: false});
          }
        });
    }, DEBOUNCE_MS);

    return () => {
      clearTimeout(timer);
      controller.abort();
    };
  }, [open, query]);

  useEffect(() => {
    setActiveIndex(-1);
  }, [query]);

  // Close the panel when clicking outside the field and the panel
  useEffect(() => {
    const handleClickOutside = (event) => {
      if (containerRef.current && !containerRef.current.contains(event.target)) {
        setOpen(false);
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  const openPanel = useCallback(() => {
    const header = containerRef.current?.closest('.as-global-header-radix__sticky-bar');
    const bottom = (header || containerRef.current)?.getBoundingClientRect().bottom;
    setPanelTop(Number.isFinite(bottom) ? bottom : null);
    setRecentSearches(readRecentSearches());
    setOpen(true);
  }, []);

  const closePanel = useCallback(() => {
    setOpen(false);
    setActiveIndex(-1);
  }, []);

  // Ctrl+K (Command+K on Apple platforms) focuses the search field from anywhere in the page
  useEffect(() => {
    const handleShortcut = (event) => {
      if (!fieldRef.current || !isSearchShortcut(event, isApple)) {
        return;
      }
      event.preventDefault();
      fieldRef.current.focus();
      fieldRef.current.select();
      openPanel();
    };

    document.addEventListener('keydown', handleShortcut);
    return () => document.removeEventListener('keydown', handleShortcut);
  }, [isApple, openPanel]);

  const withClassicNavigation = useCallback((navigate) => {
    const menuCtrl =
      window.Ext && Ext.getApplication && Ext.getApplication().getController
        ? Ext.getApplication().getController('Menu')
        : null;
    handleExtJsUnsavedChanges(menuCtrl, navigate);
  }, []);

  const remember = useCallback((text) => {
    if (text) {
      setRecentSearches(addRecentSearch(text));
    }
  }, []);

  // Run a search: in every repository, or only in the one the page browses
  const navigateToSearch = useCallback((text, repository = null) => {
    closePanel();
    remember(text);

    if (isPreviewUI) {
      // inherit: false — `go` defaults to inheriting the router's cached params,
      // and those go stale because the search page writes its URL with raw
      // pushState, which never informs the router. Without this, a fresh search
      // silently resurrects a format or sort the user had moved on from.
      // Unspecified params fall back to their declared defaults, all null.
      //
      // reload — inherit: false alone still loses the case this fix targets.
      // Re-submitting a term the router already has cached produces a
      // same-state/same-params transition, which UI-Router rejects as
      // 'SameAsCurrent' (see Transition._ignoredReason), so nothing navigates
      // and the page keeps showing the drilled-down nameOrVersion results.
      // Naming the state (rather than passing `true`) scopes the forced
      // re-entry: options.reload === true resolves to the registry root
      // (stateService.transitionTo), remounting the whole preview shell —
      // including this header — on every search.
      const searchParams = text ? {q: text} : {};
      if (text && repository) {
        searchParams.repository = repository;
      }
      router.stateService.go(UNIFIED_SEARCH, searchParams, {inherit: false, reload: UNIFIED_SEARCH});
    } else {
      withClassicNavigation(async () => {
        if (state.name !== ROUTE_NAMES.BROWSE.SEARCH.GENERIC) {
          await router.stateService.go(ROUTE_NAMES.BROWSE.SEARCH.GENERIC, {
            keyword: classicSearchKeyword(text, repository)
          });
        } else if (text) {
          ExtJS.search(text);
        }
      });
    }
  }, [isPreviewUI, state.name, router, closePanel, remember, withClassicNavigation]);

  // Open a repository, or a component at the path of its first asset, in the browse page
  const navigateToBrowse = useCallback((repository, path = null) => {
    closePanel();
    setSearchValue('');

    if (isPreviewUI) {
      // Deferred so focus and blur handlers of the panel cannot cancel the transition
      queueMicrotask(() => {
        if (path) {
          router.stateService.go('preview.browse.browse.repo.path', {repoName: repository, path, tab: 'summary'});
        } else {
          router.stateService.go('preview.browse.browse.repo', {repoName: repository});
        }
      });
    } else {
      withClassicNavigation(() => router.stateService.go(ROUTE_NAMES.BROWSE.BROWSE.ROOT, {
        repo: classicBrowseParam(repository, path)
      }));
    }
  }, [isPreviewUI, router, closePanel, withClassicNavigation]);

  const forgetRecentSearch = useCallback((text) => {
    setRecentSearches(removeRecentSearch(text));
    setActiveIndex(-1);
    fieldRef.current?.focus();
  }, []);

  const matchedRepositories = useMemo(() => matchRepositories(repositories, query), [repositories, query]);

  // The sections of the panel, each row with what it does when opened
  const sections = useMemo(() => {
    if (!query) {
      return recentSearches.length === 0 ? [] : [{
        id: 'recent',
        title: PANEL.RECENT,
        rows: recentSearches.map((recent) => ({
          key: `recent-${recent}`,
          kind: 'recent',
          icon: History,
          label: recent,
          value: recent,
          select: () => {
            setSearchValue(recent);
            navigateToSearch(recent);
          },
        })),
      }];
    }

    const searchIn = {
      id: 'search-in',
      title: PANEL.SEARCH_IN(query),
      rows: [
        ...(currentRepository ? [{
          key: 'search-repository',
          kind: 'search',
          icon: FolderSearch,
          label: PANEL.THIS_REPOSITORY,
          meta: currentRepository,
          select: () => navigateToSearch(query, currentRepository),
        }] : []),
        {
          key: 'search-all',
          kind: 'search',
          icon: SearchIcon,
          label: PANEL.ALL_REPOSITORIES,
          select: () => navigateToSearch(query),
        },
      ],
    };
    const repositoriesSection = matchedRepositories.length === 0 ? null : {
      id: 'repositories',
      title: PANEL.REPOSITORIES,
      rows: matchedRepositories.map((repository) => ({
        key: `repository-${repository.name}`,
        kind: 'repository',
        icon: Database,
        repository,
        select: () => navigateToBrowse(repository.name),
      })),
    };
    const shownComponents = components.query === query ? components.items : [];
    const componentsSection = shownComponents.length === 0 ? null : {
      id: 'components',
      title: PANEL.COMPONENTS,
      rows: shownComponents.map((component) => ({
        key: `component-${component.repository}-${component.id}`,
        kind: 'component',
        icon: Package,
        component,
        select: () => navigateToBrowse(component.repository, component.path),
      })),
    };
    return [searchIn, repositoriesSection, componentsSection].filter(Boolean);
  }, [query, recentSearches, currentRepository, matchedRepositories, components, navigateToSearch, navigateToBrowse]);

  const rows = useMemo(() => sections.flatMap((section) => section.rows), [sections]);
  const optionId = useCallback((index) => `${listboxId}-option-${index}`, [listboxId]);
  const searching = query.length >= MIN_QUERY_LENGTH && (components.loading || components.query !== query);
  const noResults = Boolean(query) && matchedRepositories.length === 0 && rows.length > 0 &&
    rows.every((row) => row.kind === 'search') && !searching;
  const showHint = !query && recentSearches.length === 0;
  const expanded = open && (rows.length > 0 || showHint);

  // Keep the active row in view when the keyboard moves past the visible part of the panel
  useEffect(() => {
    if (activeIndex >= 0) {
      document.getElementById(optionId(activeIndex))?.scrollIntoView?.({block: 'nearest'});
    }
  }, [activeIndex, optionId]);

  // Both UIs should behave identically - use the same visibility check (permission-based)
  // In dev, anonymous has astrum:search:read so search shows
  // In prod, anonymous typically doesn't have this permission so search is hidden
  // NOTE: This check must come AFTER all hooks to avoid React hook violations
  if (!defaultUIVisible) {
    return null;
  }

  const handleKeyDown = (event) => {
    switch (event.key) {
      case 'ArrowDown':
      case 'ArrowUp': {
        event.preventDefault();
        if (!open) {
          openPanel();
          return;
        }
        if (rows.length === 0) {
          return;
        }
        const step = event.key === 'ArrowDown' ? 1 : -1;
        setActiveIndex((previous) => {
          if (previous < 0) {
            return step > 0 ? 0 : rows.length - 1;
          }
          return (previous + step + rows.length) % rows.length;
        });
        break;
      }
      case 'Enter':
        event.preventDefault();
        if (open && activeIndex >= 0 && rows[activeIndex]) {
          rows[activeIndex].select();
        } else {
          navigateToSearch(query);
        }
        break;
      case 'Escape':
        if (open) {
          event.preventDefault();
          closePanel();
        }
        break;
      case 'Tab':
        closePanel();
        break;
      default:
        break;
    }
  };

  const clearSearch = () => {
    setSearchValue('');
    fieldRef.current?.focus();
    openPanel();
  };

  const renderRowContent = (row) => {
    if (row.kind === 'repository') {
      const {repository} = row;
      return (
        <>
          <span className="search-radix-row__label"><Highlighted text={repository.name} query={query} /></span>
          <span className="search-radix-row__chips">
            <Badge size="1" variant="soft" color="gray" radius="full">
              {FORMAT_LABELS[repository.format] || repository.format}
            </Badge>
            {repository.type && (
              <Badge size="1" variant="outline" color="gray" radius="full">
                {TYPE_LABELS[repository.type] || repository.type}
              </Badge>
            )}
          </span>
        </>
      );
    }
    if (row.kind === 'component') {
      const {component} = row;
      return (
        <>
          <span className="search-radix-row__label">
            <Highlighted text={component.name} query={query} />
            {component.version && <span className="search-radix-row__version">{component.version}</span>}
            {component.group && (
              <span className="search-radix-row__meta"><Highlighted text={component.group} query={query} /></span>
            )}
          </span>
          <span className="search-radix-row__chips">
            <span className="search-radix-row__repository">{component.repository}</span>
            <Badge size="1" variant="soft" color="gray" radius="full">
              {FORMAT_LABELS[component.format] || component.format}
            </Badge>
          </span>
        </>
      );
    }
    return (
      <span className="search-radix-row__label">
        {row.label}
        {row.meta && <span className="search-radix-row__meta">{row.meta}</span>}
      </span>
    );
  };

  let rowIndex = -1;

  return (
    <div
      className="search-radix-container"
      role="search"
      ref={containerRef}
      onBlur={(event) => {
        if (!containerRef.current?.contains(event.relatedTarget)) {
          closePanel();
        }
      }}
    >
      <TextField.Root
        ref={fieldRef}
        role="combobox"
        placeholder={HEADER.SEARCH_PLACEHOLDER}
        aria-label={HEADER.SEARCH_LABEL}
        aria-keyshortcuts={isApple ? 'Meta+K' : 'Control+K'}
        aria-autocomplete="list"
        aria-expanded={expanded && rows.length > 0}
        aria-controls={expanded && rows.length > 0 ? listboxId : undefined}
        aria-activedescendant={expanded && rows[activeIndex] ? optionId(activeIndex) : undefined}
        autoComplete="off"
        value={searchValue}
        onChange={(e) => {
          setSearchValue(e.target.value);
          if (!open) {
            openPanel();
          }
        }}
        onKeyDown={handleKeyDown}
        onFocus={openPanel}
        onMouseDown={() => {
          if (!open) {
            openPanel();
          }
        }}
        size="3"
        className="search-radix-input"
        data-analytics-id="as-header-search"
      >
        <TextField.Slot>
          <SearchIcon size={16} aria-hidden />
        </TextField.Slot>
        <TextField.Slot side="right">
          {searchValue ? (
            <IconButton
              type="button"
              size="1"
              variant="ghost"
              color="gray"
              className="search-radix-clear"
              aria-label={HEADER.SEARCH_CLEAR}
              onMouseDown={(event) => event.preventDefault()}
              onClick={clearSearch}
            >
              <X size={14} aria-hidden />
            </IconButton>
          ) : (
            <kbd className="search-radix-shortcut" aria-hidden="true" data-testid="search-shortcut-hint">
              {isApple ? HEADER.SEARCH_SHORTCUT_MAC : HEADER.SEARCH_SHORTCUT}
            </kbd>
          )}
        </TextField.Slot>
      </TextField.Root>

      {expanded && (
        <div
          className="search-radix-panel"
          data-testid="search-panel"
          style={panelTop == null ? undefined : {'--search-panel-top': `${panelTop}px`}}
          onMouseDown={(event) => event.preventDefault()}
        >
          {rows.length > 0 && (
            <div role="listbox" id={listboxId} aria-label={PANEL.LABEL} className="search-radix-panel__list">
              {sections.map((section) => (
                <div
                  key={section.id}
                  role="group"
                  aria-labelledby={`${listboxId}-${section.id}`}
                  className="search-radix-panel__section"
                  data-testid={`search-section-${section.id}`}
                >
                  <div id={`${listboxId}-${section.id}`} className="search-radix-panel__title">{section.title}</div>
                  {section.rows.map((row) => {
                    rowIndex += 1;
                    const index = rowIndex;
                    const active = index === activeIndex;
                    const Icon = row.icon;
                    return (
                      <div
                        key={row.key}
                        id={optionId(index)}
                        role="option"
                        aria-selected={active}
                        className={`search-radix-row${active ? ' search-radix-row--active' : ''}`}
                        data-testid={`search-option-${index}`}
                        onMouseMove={() => {
                          if (!active) {
                            setActiveIndex(index);
                          }
                        }}
                        onClick={() => row.select()}
                      >
                        <Icon size={16} aria-hidden className="search-radix-row__icon" />
                        {renderRowContent(row)}
                        {active && <kbd className="search-radix-row__enter" aria-hidden="true">{PANEL.OPEN_HINT}</kbd>}
                        {row.kind === 'recent' && (
                          <IconButton
                            type="button"
                            size="1"
                            variant="ghost"
                            color="gray"
                            tabIndex={-1}
                            className="search-radix-row__remove"
                            aria-label={PANEL.REMOVE_RECENT(row.value)}
                            onClick={(event) => {
                              event.stopPropagation();
                              forgetRecentSearch(row.value);
                            }}
                          >
                            <X size={14} aria-hidden />
                          </IconButton>
                        )}
                      </div>
                    );
                  })}
                </div>
              ))}
            </div>
          )}
          {noResults && <div className="search-radix-panel__status" role="status">{PANEL.NO_RESULTS}</div>}
          {searching && matchedRepositories.length === 0 && (
            <div className="search-radix-panel__status" role="status">{PANEL.SEARCHING}</div>
          )}
          {showHint && <div className="search-radix-panel__hint">{PANEL.EMPTY_HINT}</div>}
        </div>
      )}
    </div>
  );
}
