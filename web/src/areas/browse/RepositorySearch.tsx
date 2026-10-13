import { Link as RadixLink, Skeleton, Text } from '@radix-ui/themes';
import { useQueries } from '@tanstack/react-query';
import { Package } from 'lucide-react';
import { Link } from 'react-router';
import { api, apiKey, withQuery } from '../../api';
import { useCan } from '../../auth/state';
import { useT } from '../../i18n';
import { ErrorState } from '../../ui';
import { SEARCH_READ } from './details/panels';
import { componentTitle } from './links';
import styles from './RepositoryPage.module.css';
import { searchTarget } from './tree/model';
import type { RepositoryItem, SearchComponent, SearchPage } from './types';

export const SEARCH_PATH = '/service/rest/v1/search';

const LAYOUT_FORMATS = ['docker', 'npm', 'pypi', 'helm', 'nuget'];

export function searchedRepositories(repository: RepositoryItem): string[] {
  return repository.type === 'group' ? (repository.members ?? []) : [repository.name];
}

export function mergeResults(pages: Array<SearchPage | undefined>): SearchComponent[] {
  const seen = new Set<string>();
  const merged: SearchComponent[] = [];
  for (const page of pages) {
    for (const item of page?.items ?? []) {
      const key = `${item.format}:${item.group}:${item.name}:${item.version}`;
      if (!seen.has(key)) {
        seen.add(key);
        merged.push(item);
      }
    }
  }
  return merged;
}

export function RepositorySearch({
  repository,
  query,
  selection,
  onOpen,
}: {
  repository: RepositoryItem;
  query: string;
  selection: string;
  onOpen(path: string): void;
}) {
  const t = useT();
  const canSearch = useCan(SEARCH_READ);
  const names = searchedRepositories(repository);
  const results = useQueries({
    queries: names.map((name) => {
      const key = withQuery(SEARCH_PATH, { repository: name, q: query });
      return {
        queryKey: apiKey(key),
        queryFn: ({ signal }: { signal: AbortSignal }) => api.get<SearchPage>(key, { signal }),
      };
    }),
  });
  if (names.length === 0) {
    return (
      <Text as="p" size="2" color="gray" className={styles.searchNote}>
        {t('browse.search.noMembers')}
      </Text>
    );
  }
  if (results.some((result) => result.isPending)) {
    return (
      <div className={styles.searchLoading} aria-busy="true">
        <Skeleton height="40px" />
        <Skeleton height="40px" />
      </div>
    );
  }
  const failed = results.find((result) => result.isError);
  if (failed && results.every((result) => !result.data)) {
    return <ErrorState error={failed.error} onRetry={() => failed.refetch()} />;
  }
  const items = mergeResults(results.map((result) => result.data));
  const more = results.some((result) => result.data?.continuationToken);
  if (items.length === 0) {
    return (
      <Text as="p" size="2" color="gray" className={styles.searchNote} role="status">
        {t('browse.search.empty', { query })}
      </Text>
    );
  }
  return (
    <div className={styles.searchResults}>
      <Text as="p" size="1" color="gray" role="status" className={styles.searchNote}>
        {t('browse.search.count', { count: items.length })}
      </Text>
      <ul aria-label={t('browse.search.results')} className={styles.resultList}>
        {items.map((item) => {
          const target = LAYOUT_FORMATS.includes(item.format)
            ? (item.assets[0]?.path ?? '')
            : searchTarget(
                item.version,
                item.assets.map((asset) => asset.path),
              );
          const current = target !== '' && target === selection;
          return (
            <li key={item.id}>
              <button
                type="button"
                className={styles.result}
                data-current={current}
                aria-current={current ? 'true' : undefined}
                disabled={target === ''}
                onClick={() => onOpen(target)}
              >
                <Package size={15} aria-hidden="true" className={styles.resultIcon} />
                <span className={styles.resultText}>
                  <span className={styles.resultName}>
                    {componentTitle(item.format, item.group, item.name)}
                  </span>
                  <span className={styles.resultMeta}>
                    {[item.version, repository.type === 'group' ? item.repository : '']
                      .filter(Boolean)
                      .join(' · ')}
                    {item.version || repository.type === 'group' ? ' · ' : ''}
                    {t('browse.search.files', { count: item.assets.length })}
                  </span>
                </span>
              </button>
            </li>
          );
        })}
      </ul>
      {more && canSearch && (
        <Text as="p" size="2" className={styles.searchNote}>
          <RadixLink asChild>
            <Link to={withQuery('/search', { q: query, repository: repository.name })}>
              {t('browse.search.more')}
            </Link>
          </RadixLink>
        </Text>
      )}
    </div>
  );
}
