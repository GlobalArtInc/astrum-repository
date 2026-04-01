import { Database, FolderOpen, History, Package, Search } from 'lucide-react';
import { type ReactNode, useEffect, useMemo, useState } from 'react';
import { useLocation } from 'react-router';
import { useApi, withQuery } from '../../api';
import type { SearchResultsProps } from '../../app/area';
import { useT } from '../../i18n';
import { SearchGroup, SearchItem } from '../../shell/search';
import { FormatBadge, TypeBadge } from '../../ui';
import {
  browsePath,
  componentPath,
  REPOSITORIES,
  type RepositorySummary,
  SEARCH,
  type SearchComponent,
  type SearchPage,
} from './model';
import { MIN_QUERY, searchAddress } from './query';
import { readRecent, rememberSearch } from './recent';
import styles from './Search.module.css';

export const PANEL_REPOSITORIES = 5;
export const PANEL_COMPONENTS = 6;
const DEBOUNCE_MS = 250;

export function useDebounced<T>(value: T, delay: number): T {
  const [settled, setSettled] = useState(value);
  useEffect(() => {
    const timer = window.setTimeout(() => setSettled(value), delay);
    return () => window.clearTimeout(timer);
  }, [value, delay]);
  return settled;
}

export function currentRepository(pathname: string): string | null {
  const match = /^\/browse\/([^/]+)/.exec(pathname);
  if (!match?.[1]) {
    return null;
  }
  try {
    return decodeURIComponent(match[1]);
  } catch {
    return null;
  }
}

export function matchRepositories(repositories: readonly RepositorySummary[], text: string) {
  const wanted = text.trim().toLowerCase();
  if (wanted === '') {
    return [];
  }
  return repositories
    .filter((repository) => repository.name.toLowerCase().includes(wanted))
    .sort((left, right) => {
      const leftStarts = left.name.toLowerCase().startsWith(wanted) ? 0 : 1;
      const rightStarts = right.name.toLowerCase().startsWith(wanted) ? 0 : 1;
      return leftStarts - rightStarts || left.name.localeCompare(right.name);
    })
    .slice(0, PANEL_REPOSITORIES);
}

export function highlight(text: string, query: string): ReactNode {
  const wanted = query.trim().toLowerCase();
  const at = wanted === '' ? -1 : text.toLowerCase().indexOf(wanted);
  if (at < 0) {
    return text;
  }
  return (
    <>
      {text.slice(0, at)}
      <strong className={styles.match}>{text.slice(at, at + wanted.length)}</strong>
      {text.slice(at + wanted.length)}
    </>
  );
}

export function submitSearch(query: string): string {
  rememberSearch(query);
  return searchAddress(query);
}

function RecentSearches() {
  const t = useT();
  const recent = useMemo(readRecent, []);
  if (recent.length === 0) {
    return null;
  }
  return (
    <SearchGroup title={t('search.panel.recent')}>
      {recent.map((entry) => (
        <SearchItem key={entry} to={searchAddress(entry)} onSelect={() => rememberSearch(entry)}>
          <History size={16} aria-hidden="true" />
          <span className={styles.itemText}>{entry}</span>
        </SearchItem>
      ))}
    </SearchGroup>
  );
}

export function HeaderSearchResults({ query }: SearchResultsProps) {
  const t = useT();
  const location = useLocation();
  const here = currentRepository(location.pathname);
  const settled = useDebounced(query, DEBOUNCE_MS);
  const repositories = useApi<RepositorySummary[]>(query === '' ? null : REPOSITORIES);
  const wanted = settled.trim().length >= MIN_QUERY ? settled.trim() : null;
  const components = useApi<SearchPage<SearchComponent>>(
    wanted ? withQuery(SEARCH, { q: wanted, sort: 'name', direction: 'asc' }) : null,
  );
  const repositoryMatches = useMemo(
    () => matchRepositories(repositories.data ?? [], query),
    [repositories.data, query],
  );

  if (query === '') {
    return <RecentSearches />;
  }

  const componentMatches = (wanted === query.trim() ? components.data?.items : undefined)?.slice(
    0,
    PANEL_COMPONENTS,
  );
  const waiting = query.trim().length >= MIN_QUERY && (wanted !== query.trim() || components.isPending);
  const remember = () => rememberSearch(query);

  return (
    <>
      <SearchGroup title={t('search.panel.searchIn', { query })}>
        <SearchItem to={searchAddress(query)} onSelect={remember}>
          <Search size={16} aria-hidden="true" />
          <span className={styles.itemText}>{t('search.panel.everywhere')}</span>
        </SearchItem>
        {here && (
          <SearchItem to={searchAddress(query, here)} onSelect={remember}>
            <FolderOpen size={16} aria-hidden="true" />
            <span className={styles.itemText}>{t('search.panel.thisRepository')}</span>
          </SearchItem>
        )}
      </SearchGroup>
      {repositoryMatches.length > 0 && (
        <SearchGroup title={t('search.panel.repositories')}>
          {repositoryMatches.map((repository) => (
            <SearchItem key={repository.name} to={browsePath(repository.name)}>
              <Database size={16} aria-hidden="true" />
              <span className={styles.itemText}>{highlight(repository.name, query)}</span>
              <span className={styles.itemMeta}>
                <FormatBadge format={repository.format} variant="name" />
                <TypeBadge type={repository.type} variant="outline" />
              </span>
            </SearchItem>
          ))}
        </SearchGroup>
      )}
      {(componentMatches?.length ?? 0) > 0 ? (
        <SearchGroup title={t('search.panel.components')}>
          {componentMatches?.map((component) => (
            <SearchItem key={component.id} to={componentPath(component)} onSelect={remember}>
              <Package size={16} aria-hidden="true" />
              <span className={styles.itemText}>
                {highlight(component.name, query)}
                {component.version && <span className={styles.version}>{component.version}</span>}
                {component.group && <span className={styles.group}>{component.group}</span>}
              </span>
              <span className={styles.itemMeta}>
                <span className={styles.repositoryName}>{component.repository}</span>
                <FormatBadge format={component.format} variant="name" />
              </span>
            </SearchItem>
          ))}
        </SearchGroup>
      ) : waiting ? (
        <div className={styles.panelNote} role="status">
          {t('search.panel.searching')}
        </div>
      ) : wanted && repositoryMatches.length === 0 ? (
        <div className={styles.panelNote} role="status">
          {t('search.panel.nothing')}
        </div>
      ) : null}
    </>
  );
}
