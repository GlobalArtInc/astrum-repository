import React, { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import {
  Box,
  Button,
  Callout,
  Flex,
  Heading,
  IconButton,
  Select,
  Spinner,
  Table,
  Text,
  TextField,
  Tooltip,
} from '@radix-ui/themes';
import { AlertCircle, History, PackageSearch, Search, SearchX, X } from 'lucide-react';
import { useRouter } from '@uirouter/react';

import UIStrings from '../../../constants/UIStrings';
import { FormatBadge } from '../../browse/shared';
import { DockerArtifactBadge } from '../../../shared/DockerArtifact';
import { getFormatOptions } from './searchFilters';
import { useRepositories } from './useRepositories';
import { useSearchNavigation } from './useSearchNavigation';
import { useSearchUrlState } from './useSearchUrlState';
import { useUnifiedSearch } from './useUnifiedSearch';
import { SORT_OPTIONS } from './sortOptions';
import { formatDateTime } from '../../browse/detail/detail.utils';
import { useIsPhone } from '../../../../../hooks/useMediaQuery';
import type { SearchFormat, SearchResult, SortDirection, SortField } from './unified.types';

import './UnifiedSearchPage.scss';

/** Keystrokes are collected this long before a search starts. */
export const SEARCH_DEBOUNCE_MS = 300;
/** A keyword search needs at least this many characters. */
export const MIN_QUERY_LENGTH = 2;

const ALL_REPOSITORIES = '__all__';

const SORT_LABEL_KEYS: Record<string, keyof typeof UIStrings.SEARCH_PAGE.SORT> = {
  'lastUpdated:desc': 'LAST_UPDATED_DESC',
  'lastUpdated:asc': 'LAST_UPDATED_ASC',
  'name:asc': 'NAME_ASC',
  'name:desc': 'NAME_DESC',
  'repository:asc': 'REPOSITORY_ASC',
  'repository:desc': 'REPOSITORY_DESC',
};

/**
 * Whether there is anything to search for: a keyword long enough, or a format or repository to list.
 */
export function hasSearchCriteria(query: string, format: SearchFormat, repository: string | undefined): boolean {
  return query.trim().length >= MIN_QUERY_LENGTH || format !== 'all' || Boolean(repository);
}

function displayName(result: SearchResult): string {
  if (result.format === 'npm' && result.group) {
    return `@${result.group.replace(/^@/, '')}/${result.name}`;
  }
  return result.name;
}

/**
 * Path of a result's first file in the browse tree of its repository.
 */
function browsePath(result: SearchResult): string | null {
  return result.path ? result.path.replace(/^\/+/, '') : null;
}

/**
 * Search: a keyword field with format, repository and sort, and the components found in every
 * repository the user can browse. The criteria live in the URL, so a search can be shared and survives
 * a reload; the global search in the header lands here with its keyword.
 */
export function UnifiedSearchPage(): JSX.Element {
  const strings = UIStrings.SEARCH_PAGE;
  const isPhone = useIsPhone();
  const router = useRouter();
  const { state, setFormat, setQuery, setFilter, setSort, search, loadMore, hasMore } = useUnifiedSearch();
  const { readFromUrl, syncToUrl } = useSearchUrlState();
  const { navigateToDetail } = useSearchNavigation();

  const initial = useMemo(() => readFromUrl(), [readFromUrl]);
  const [input, setInput] = useState(initial.query);
  const [searched, setSearched] = useState(false);
  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  const format = state.format;
  const repository = state.filters.repository || undefined;
  const { repositories, availableFormats } = useRepositories(
    format === 'all' ? undefined : getFormatOptions().find((option) => option.id === format)?.apiFormat,
  );

  const formatOptions = useMemo(
    () => getFormatOptions().filter((option) => option.id !== 'all' && availableFormats.has(option.apiFormat)),
    [availableFormats],
  );

  const runSearch = useCallback(
    (query: string, nextFormat: SearchFormat, nextRepository: string | undefined, replace: boolean) => {
      syncToUrl(
        {
          format: nextFormat,
          query,
          filters: nextRepository ? { repository: nextRepository } : {},
          sortField: state.sortField,
          sortDirection: state.sortDirection,
        },
        replace,
      );
      if (hasSearchCriteria(query, nextFormat, nextRepository)) {
        setSearched(true);
        search();
      } else {
        setSearched(false);
      }
    },
    [syncToUrl, search, state.sortField, state.sortDirection],
  );

  // Restore the search of the URL once.
  useEffect(() => {
    if (initial.format !== 'all') {
      setFormat(initial.format);
    }
    if (initial.filters.repository) {
      setFilter('repository', initial.filters.repository);
    }
    if (initial.sortField || initial.sortDirection) {
      setSort(initial.sortField ?? 'lastUpdated', initial.sortDirection ?? 'desc');
    }
    setQuery(initial.query);
    if (hasSearchCriteria(initial.query, initial.format, initial.filters.repository)) {
      setSearched(true);
      search();
    }
    inputRef.current?.focus();
  }, []);

  useEffect(() => () => {
    if (debounceRef.current) {
      clearTimeout(debounceRef.current);
    }
  }, []);

  const handleInput = useCallback(
    (value: string) => {
      setInput(value);
      setQuery(value);
      if (debounceRef.current) {
        clearTimeout(debounceRef.current);
      }
      debounceRef.current = setTimeout(() => runSearch(value, format, repository, true), SEARCH_DEBOUNCE_MS);
    },
    [setQuery, runSearch, format, repository],
  );

  const handleSubmit = useCallback(
    (event: React.FormEvent) => {
      event.preventDefault();
      if (debounceRef.current) {
        clearTimeout(debounceRef.current);
      }
      runSearch(input, format, repository, false);
    },
    [runSearch, input, format, repository],
  );

  const handleClear = useCallback(() => {
    if (debounceRef.current) {
      clearTimeout(debounceRef.current);
    }
    setInput('');
    setQuery('');
    runSearch('', format, repository, false);
    inputRef.current?.focus();
  }, [setQuery, runSearch, format, repository]);

  // A select inside a form reports an empty value while its options are still loading and miss the current one.
  // That is no choice of the user, and taking it would drop the format or the repository of the URL.
  const handleFormatChange = useCallback(
    (value: string) => {
      // Radix Select reports an empty value through its hidden native select while the format of the
      // URL is not among the options yet, which load with the repositories. That is not a choice.
      if (!getFormatOptions().some((option) => option.id === value)) {
        return;
      }
      const nextFormat = value as SearchFormat;
      setFormat(nextFormat);
      setQuery(input);
      runSearch(input, nextFormat, undefined, false);
    },
    [setFormat, setQuery, runSearch, input],
  );

  const handleRepositoryChange = useCallback(
    (value: string) => {
      if (!value) return;
      const nextRepository = value === ALL_REPOSITORIES ? undefined : value;
      setFilter('repository', nextRepository ?? '');
      runSearch(input, format, nextRepository, false);
    },
    [setFilter, runSearch, input, format],
  );

  const handleSortChange = useCallback(
    (value: string) => {
      const option = SORT_OPTIONS.find((candidate) => candidate.value === value);
      if (!option) return;
      setSort(option.field as SortField, option.direction as SortDirection);
      syncToUrl(
        {
          format,
          query: input,
          filters: repository ? { repository } : {},
          sortField: option.field,
          sortDirection: option.direction,
        },
        false,
      );
      if (hasSearchCriteria(input, format, repository)) {
        setSearched(true);
        search();
      }
    },
    [setSort, syncToUrl, search, format, input, repository],
  );

  const showInRepository = useCallback(
    (result: SearchResult) => {
      const path = browsePath(result);
      if (path) {
        router.stateService.go('preview.browse.browse.repo.path', { repoName: result.repository, path, tab: 'summary' });
      } else {
        router.stateService.go('preview.browse.browse.repo', { repoName: result.repository });
      }
    },
    [router],
  );

  const sortValue = `${state.sortField}:${state.sortDirection}`;
  const results = state.results;
  const firstLoad = state.loading && results.length === 0;

  let body: React.ReactNode;
  if (!searched && results.length === 0 && !state.loading) {
    body = (
      <SearchMessage icon={<PackageSearch size={28} aria-hidden />} title={strings.START_TITLE} description={strings.START_DESCRIPTION} />
    );
  } else if (firstLoad) {
    body = (
      <Flex align="center" justify="center" gap="3" py="9" className="unified-search__loading" role="status">
        <Spinner size="3" />
        <Text color="gray">{strings.LOADING}</Text>
      </Flex>
    );
  } else if (state.error && results.length === 0) {
    body = (
      <Callout.Root color="red" size="2">
        <Callout.Icon>
          <AlertCircle size={16} />
        </Callout.Icon>
        <Callout.Text>
          <Text weight="medium">{strings.ERROR_TITLE}</Text>
          {' '}
          {state.error}
        </Callout.Text>
        <Box>
          <Button size="1" variant="soft" color="red" onClick={() => runSearch(input, format, repository, true)}>
            {strings.RETRY}
          </Button>
        </Box>
      </Callout.Root>
    );
  } else if (results.length === 0) {
    body = (
      <SearchMessage icon={<SearchX size={28} aria-hidden />} title={strings.EMPTY_TITLE} description={strings.EMPTY_DESCRIPTION} />
    );
  } else {
    body = (
      <Flex direction="column" gap="3">
        <Flex align="center" justify="between" gap="2">
          <Text size="2" color="gray" aria-live="polite">{strings.SHOWN(results.length)}</Text>
          {state.loading && <Spinner size="1" />}
        </Flex>
        {isPhone ? (
          <ul className="unified-search__list" data-testid="unified-search-list">
            {results.map((result) => (
              <li key={`${result.repository}:${result.id}`} className="unified-search__item">
                <button
                  type="button"
                  className="unified-search__open"
                  aria-label={strings.OPEN(displayName(result))}
                  onClick={() => showInRepository(result)}
                >
                  <Text size="2" weight="medium" className="unified-search__name">{displayName(result)}</Text>
                  <Text size="1" color="gray" className="unified-search__mono unified-search__ellipsis">
                    {result.group && result.format !== 'npm' ? `${result.group} · ` : ''}
                    {result.version || '—'}
                  </Text>
                  <Flex align="center" gap="2" wrap="wrap">
                    <FormatBadge format={result.format} size="1" radius="full" />
                    {result.artifactKind && <DockerArtifactBadge kind={result.artifactKind} />}
                    <Text size="1" color="gray">{result.repository}</Text>
                  </Flex>
                </button>
                <IconButton
                  variant="ghost"
                  color="gray"
                  size="3"
                  className="unified-search__versions"
                  aria-label={strings.VERSIONS_OF(displayName(result))}
                  onClick={() => navigateToDetail(result)}
                >
                  <History size={18} />
                </IconButton>
              </li>
            ))}
          </ul>
        ) : (
        <Table.Root variant="surface" size="2" className="unified-search__table">
          <Table.Header>
            <Table.Row>
              <Table.ColumnHeaderCell>{strings.COLUMNS.NAME}</Table.ColumnHeaderCell>
              <Table.ColumnHeaderCell>{strings.COLUMNS.VERSION}</Table.ColumnHeaderCell>
              <Table.ColumnHeaderCell>{strings.COLUMNS.FORMAT}</Table.ColumnHeaderCell>
              <Table.ColumnHeaderCell>{strings.COLUMNS.REPOSITORY}</Table.ColumnHeaderCell>
              <Table.ColumnHeaderCell className="unified-search__updated">{strings.COLUMNS.UPDATED}</Table.ColumnHeaderCell>
              <Table.ColumnHeaderCell width="48px" aria-label={strings.VERSIONS} />
            </Table.Row>
          </Table.Header>
          <Table.Body>
            {results.map((result) => (
              <Table.Row
                key={`${result.repository}:${result.id}`}
                className="unified-search__row"
                tabIndex={0}
                role="button"
                aria-label={strings.OPEN(displayName(result))}
                onClick={() => showInRepository(result)}
                onKeyDown={(event) => {
                  if (event.target === event.currentTarget && (event.key === 'Enter' || event.key === ' ')) {
                    event.preventDefault();
                    showInRepository(result);
                  }
                }}
              >
                <Table.Cell>
                  <Flex direction="column" gap="1" minWidth="0">
                    <Text size="2" weight="medium" className="unified-search__name">{displayName(result)}</Text>
                    {result.group && result.format !== 'npm' && (
                      <Text size="1" color="gray" className="unified-search__mono">{result.group}</Text>
                    )}
                  </Flex>
                </Table.Cell>
                <Table.Cell>
                  <Text size="2" className="unified-search__mono">{result.version || '—'}</Text>
                </Table.Cell>
                <Table.Cell>
                  <Flex align="center" gap="2" wrap="wrap">
                    <FormatBadge format={result.format} size="1" radius="full" />
                    {result.artifactKind && <DockerArtifactBadge kind={result.artifactKind} />}
                  </Flex>
                </Table.Cell>
                <Table.Cell>
                  <Text size="2" color="gray">{result.repository}</Text>
                </Table.Cell>
                <Table.Cell className="unified-search__updated">
                  <Text size="2" color="gray">{formatDateTime(result.lastUpdated)}</Text>
                </Table.Cell>
                <Table.Cell justify="end">
                  <Tooltip content={strings.VERSIONS}>
                    <IconButton
                      variant="ghost"
                      color="gray"
                      size="1"
                      aria-label={strings.VERSIONS_OF(displayName(result))}
                      onClick={(event) => {
                        event.stopPropagation();
                        navigateToDetail(result);
                      }}
                    >
                      <History size={16} />
                    </IconButton>
                  </Tooltip>
                </Table.Cell>
              </Table.Row>
            ))}
          </Table.Body>
        </Table.Root>
        )}
        {hasMore && (
          <Flex justify="center">
            <Button variant="surface" color="gray" highContrast disabled={state.loading} onClick={() => loadMore()}>
              {state.loading ? <Spinner size="1" /> : null}
              {strings.LOAD_MORE}
            </Button>
          </Flex>
        )}
      </Flex>
    );
  }

  return (
    <Box className="unified-search" px={{ initial: '4', md: '6' }} py={{ initial: '4', md: '5' }} data-testid="unified-search-page">
      <Flex direction="column" gap="5" className="unified-search__inner">
        <Flex direction="column" gap="1">
          <Heading as="h1" size="7" weight="bold" className="unified-search__title">{strings.TITLE}</Heading>
          <Text size="2" color="gray">{strings.SUBTITLE}</Text>
        </Flex>

        <form role="search" onSubmit={handleSubmit} className="unified-search__bar">
          <Flex gap="3" wrap="wrap" align="end">
            <Box className="unified-search__field">
              <TextField.Root
                ref={inputRef}
                size="3"
                value={input}
                onChange={(event) => handleInput(event.target.value)}
                placeholder={strings.PLACEHOLDER}
                aria-label={strings.SEARCH_LABEL}
              >
                <TextField.Slot>
                  <Search size={16} aria-hidden />
                </TextField.Slot>
                {input && (
                  <TextField.Slot side="right">
                    <IconButton type="button" variant="ghost" color="gray" size="1" onClick={handleClear} aria-label={strings.CLEAR}>
                      <X size={14} />
                    </IconButton>
                  </TextField.Slot>
                )}
              </TextField.Root>
            </Box>
            <Flex direction="column" gap="1" className="unified-search__select">
              <Text as="label" size="1" color="gray" id="unified-search-format">{strings.FORMAT_LABEL}</Text>
              <Select.Root value={format} onValueChange={handleFormatChange} size="3">
                <Select.Trigger aria-labelledby="unified-search-format" />
                <Select.Content position="popper">
                  <Select.Item value="all">{strings.ALL_FORMATS}</Select.Item>
                  {formatOptions.map((option) => (
                    <Select.Item key={option.id} value={option.id}>{option.label}</Select.Item>
                  ))}
                </Select.Content>
              </Select.Root>
            </Flex>
            <Flex direction="column" gap="1" className="unified-search__select">
              <Text as="label" size="1" color="gray" id="unified-search-repository">{strings.REPOSITORY_LABEL}</Text>
              <Select.Root value={repository ?? ALL_REPOSITORIES} onValueChange={handleRepositoryChange} size="3">
                <Select.Trigger aria-labelledby="unified-search-repository" />
                <Select.Content position="popper">
                  <Select.Item value={ALL_REPOSITORIES}>{strings.ALL_REPOSITORIES}</Select.Item>
                  {repositories.map((name) => (
                    <Select.Item key={name} value={name}>{name}</Select.Item>
                  ))}
                </Select.Content>
              </Select.Root>
            </Flex>
            <Flex direction="column" gap="1" className="unified-search__select">
              <Text as="label" size="1" color="gray" id="unified-search-sort">{strings.SORT_LABEL}</Text>
              <Select.Root value={sortValue} onValueChange={handleSortChange} size="3">
                <Select.Trigger aria-labelledby="unified-search-sort" />
                <Select.Content position="popper">
                  {SORT_OPTIONS.map((option) => (
                    <Select.Item key={option.value} value={option.value}>
                      {SORT_LABEL_KEYS[option.value] ? strings.SORT[SORT_LABEL_KEYS[option.value]] : option.label}
                    </Select.Item>
                  ))}
                </Select.Content>
              </Select.Root>
            </Flex>
          </Flex>
        </form>

        <section aria-label={strings.RESULTS_LABEL} className="unified-search__results">
          {body}
        </section>
      </Flex>
    </Box>
  );
}

function SearchMessage({ icon, title, description }: { icon: React.ReactNode; title: string; description: string }): JSX.Element {
  return (
    <Flex direction="column" align="center" justify="center" gap="2" className="unified-search__message">
      <Box className="unified-search__message-icon">{icon}</Box>
      <Text size="3" weight="bold">{title}</Text>
      <Text size="2" color="gray" align="center" className="unified-search__message-text">{description}</Text>
    </Flex>
  );
}

export default UnifiedSearchPage;
