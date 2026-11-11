import { Button, Checkbox, Link as RadixLink, Skeleton, Text } from '@radix-ui/themes';
import { useQueries } from '@tanstack/react-query';
import { Package } from 'lucide-react';
import { Link } from 'react-router';
import { api, apiKey, withQuery } from '../../api';
import { useCan } from '../../auth/state';
import { useT } from '../../i18n';
import { ErrorState } from '../../ui';
import { SEARCH_READ } from './details/panels';
import { componentTitle } from './links';
import { PROMOTE_LIMIT, promotionItem } from './promote/model';
import {
  atLimit,
  canPick,
  isChosen,
  type PromoteEntry,
  selectPage,
  toggleSelection,
} from './promote/selection';
import { usePromotionTargets } from './promote/targets';
import styles from './RepositoryPage.module.css';
import { searchTarget } from './tree/model';
import type { RepositoryItem, SearchComponent, SearchPage } from './types';

export const SEARCH_PATH = '/service/rest/v1/search';

const LAYOUT_FORMATS = ['docker', 'npm', 'pypi', 'helm', 'nuget', 'rubygems'];

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

export function entryOf(item: SearchComponent): PromoteEntry {
  return {
    id: item.id,
    source: item.repository,
    format: item.format,
    item: promotionItem(item.format, item),
  };
}

export interface RepositorySearchProps {
  repository: RepositoryItem;
  query: string;
  selection: string;
  picked: PromoteEntry[];
  onPick(next: PromoteEntry[]): void;
  onPromote(): void;
  onOpen(path: string): void;
}

export function RepositorySearch({
  repository,
  query,
  selection,
  picked,
  onPick,
  onPromote,
  onOpen,
}: RepositorySearchProps) {
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
  const targets = usePromotionTargets(names);
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
  const promotable = (item: SearchComponent) => (targets.get(item.repository)?.length ?? 0) > 0;
  const offered = items.filter(promotable).map(entryOf);
  const mixed = picked.length > 0 && offered.some((entry) => !canPick(picked, entry));
  return (
    <div className={styles.searchResults}>
      <Text as="p" size="1" color="gray" role="status" className={styles.searchNote}>
        {t('browse.search.count', { count: items.length })}
      </Text>
      {offered.length > 0 && (
        <div className={styles.promoteBar}>
          <Text size="2" role="status">
            {t('browse.promote.selectedCount', { count: picked.length, limit: PROMOTE_LIMIT })}
          </Text>
          <Button
            size="1"
            variant="soft"
            disabled={atLimit(picked)}
            onClick={() => onPick(selectPage(picked, offered))}
          >
            {t('browse.promote.selectPage')}
          </Button>
          <Button size="1" disabled={picked.length === 0} onClick={onPromote}>
            {t('browse.promote.selected')}
          </Button>
        </div>
      )}
      {atLimit(picked) && (
        <Text as="p" size="1" color="amber" className={styles.searchNote}>
          {t('browse.promote.limit', { limit: PROMOTE_LIMIT })}
        </Text>
      )}
      {mixed && (
        <Text as="p" size="1" color="gray" className={styles.searchNote}>
          {t('browse.promote.oneSource')}
        </Text>
      )}
      <ul aria-label={t('browse.search.results')} className={styles.resultList}>
        {items.map((item) => {
          const target = LAYOUT_FORMATS.includes(item.format)
            ? (item.assets[0]?.path ?? '')
            : searchTarget(
                item.version,
                item.assets.map((asset) => asset.path),
              );
          const current = target !== '' && target === selection;
          const title = componentTitle(item.format, item.group, item.name);
          const entry = entryOf(item);
          const chosen = isChosen(picked, item.id);
          const eligible = canPick(picked, entry) && !atLimit(picked);
          return (
            <li key={item.id}>
              <div className={styles.resultRow}>
                {promotable(item) && (
                  <Checkbox
                    className={styles.resultPick}
                    checked={chosen}
                    disabled={!chosen && !eligible}
                    aria-label={t('browse.promote.pick', { name: title })}
                    onCheckedChange={() => onPick(toggleSelection(picked, entry))}
                  />
                )}
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
                    <span className={styles.resultName}>{title}</span>
                    <span className={styles.resultMeta}>
                      {[item.version, repository.type === 'group' ? item.repository : '']
                        .filter(Boolean)
                        .join(' · ')}
                      {item.version || repository.type === 'group' ? ' · ' : ''}
                      {t('browse.search.files', { count: item.assets.length })}
                    </span>
                  </span>
                </button>
              </div>
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
