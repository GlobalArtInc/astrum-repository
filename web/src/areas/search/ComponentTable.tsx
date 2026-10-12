import { Flex, Text } from '@radix-ui/themes';
import type { ReactNode } from 'react';
import { Link } from 'react-router';
import { useT } from '../../i18n';
import { type Column, DataTable, DateTime, FileSize, FormatBadge } from '../../ui';
import {
  browsePath,
  componentPath,
  folderOf,
  lastModified,
  type SearchAsset,
  type SearchComponent,
} from './model';
import styles from './Search.module.css';
import type { PagedSearch } from './usePagedSearch';

export function ComponentTable({
  label,
  results,
  empty,
}: {
  label: string;
  results: PagedSearch<SearchComponent>;
  empty: ReactNode;
}) {
  const t = useT();
  const columns: Column<SearchComponent>[] = [
    {
      id: 'name',
      header: t('search.results.component'),
      primary: true,
      cell: (component) => (
        <Flex direction="column" gap="1" className={styles.nameCell}>
          <span>{component.name}</span>
          {component.group && (
            <Text size="1" color="gray" className={styles.mono}>
              {component.group}
            </Text>
          )}
        </Flex>
      ),
    },
    {
      id: 'version',
      header: t('search.results.version'),
      cell: (component) => <span className={styles.mono}>{component.version || '—'}</span>,
    },
    {
      id: 'format',
      header: t('search.results.format'),
      cell: (component) => <FormatBadge format={component.format} />,
    },
    {
      id: 'repository',
      header: t('search.results.repository'),
      cell: (component) => (
        <Link className={styles.link} to={browsePath(component.repository)}>
          {component.repository}
        </Link>
      ),
    },
    {
      id: 'updated',
      header: t('search.results.updated'),
      hideOnPhone: true,
      cell: (component) => {
        const when = lastModified(component);
        return when ? <DateTime value={when} /> : '—';
      },
    },
  ];
  return (
    <DataTable
      label={label}
      columns={columns}
      rows={results.items}
      getRowId={(component) => component.id}
      loading={results.loading}
      error={results.error}
      onRetry={results.retry}
      rowHref={componentPath}
      empty={empty}
      paging="more"
      hasMore={results.hasMore}
      onLoadMore={results.loadMore}
      loadingMore={results.loadingMore}
    />
  );
}

export function AssetTable({
  label,
  results,
  empty,
}: {
  label: string;
  results: PagedSearch<SearchAsset>;
  empty: ReactNode;
}) {
  const t = useT();
  const columns: Column<SearchAsset>[] = [
    {
      id: 'path',
      header: t('search.results.path'),
      primary: true,
      cell: (asset) => (
        <a className={`${styles.link} ${styles.mono}`} href={asset.downloadUrl} download>
          {asset.path}
        </a>
      ),
    },
    {
      id: 'repository',
      header: t('search.results.repository'),
      cell: (asset) => (
        <Link className={styles.link} to={browsePath(asset.repository, folderOf(asset.path))}>
          {asset.repository}
        </Link>
      ),
    },
    {
      id: 'format',
      header: t('search.results.format'),
      cell: (asset) => <FormatBadge format={asset.format} />,
    },
    {
      id: 'size',
      header: t('search.results.size'),
      align: 'right',
      cell: (asset) => <FileSize bytes={asset.fileSize} />,
    },
    {
      id: 'contentType',
      header: t('search.results.contentType'),
      hideOnPhone: true,
      cell: (asset) => <span className={styles.mono}>{asset.contentType ?? '—'}</span>,
    },
    {
      id: 'updated',
      header: t('search.results.updated'),
      hideOnPhone: true,
      cell: (asset) => (asset.lastModified ? <DateTime value={asset.lastModified} /> : '—'),
    },
  ];
  return (
    <DataTable
      label={label}
      columns={columns}
      rows={results.items}
      getRowId={(asset) => asset.id}
      loading={results.loading}
      error={results.error}
      onRetry={results.retry}
      empty={empty}
      paging="more"
      hasMore={results.hasMore}
      onLoadMore={results.loadMore}
      loadingMore={results.loadingMore}
    />
  );
}
