import { Badge, Heading, Switch, Table, Text, TextField } from '@radix-ui/themes';
import { Search } from 'lucide-react';
import { useMemo, useState } from 'react';
import { Link } from 'react-router';
import { formatNumber, useI18n, useT } from '../../../i18n';
import { RelativeTime } from '../../../ui';
import { SortHeader, usePaging } from '../component/tables';
import { Pager, pageCount, pageSlice } from '../Pager';
import { browseUrl, isPreRelease } from '../tree/model';
import type { RepositoryItem, TreeNode } from '../types';
import styles from './VersionsTable.module.css';
import { newestStable, sortVersions, type VersionColumn, type VersionDirection } from './versions';

export interface VersionsTableProps {
  repository: RepositoryItem;
  nodes: TreeNode[];
  truncated: boolean;
}

function firstDirection(column: VersionColumn): VersionDirection {
  return column === 'name' ? 'asc' : 'desc';
}

export function VersionsTable({ repository, nodes, truncated }: VersionsTableProps) {
  const t = useT();
  const { language } = useI18n();
  const docker = repository.format === 'docker';
  const [column, setColumn] = useState<VersionColumn>('modified');
  const [direction, setDirection] = useState<VersionDirection>('desc');
  const [filter, setFilter] = useState('');
  const [hidePre, setHidePre] = useState(false);
  const rows = useMemo(() => {
    const wanted = filter.trim().toLowerCase();
    const matching = nodes.filter(
      (node) =>
        (!hidePre || !isPreRelease(node, repository.format)) &&
        (wanted === '' || node.name.toLowerCase().includes(wanted)),
    );
    return sortVersions(matching, column, direction);
  }, [nodes, filter, hidePre, column, direction, repository.format]);
  const newest = newestStable(nodes, repository.format);
  const paging = usePaging(rows.length);
  const shown = pageSlice(rows, paging.page, paging.size);
  const title = docker ? t('browse.versions.title.docker') : t('browse.versions.title.default');
  const change = (next: VersionColumn, nextDirection: VersionDirection) => {
    setDirection(next === column ? nextDirection : firstDirection(next));
    setColumn(next);
  };
  return (
    <section className={styles.card} aria-label={title}>
      <Heading as="h3" size="3" className={styles.title}>
        {title}
      </Heading>
      <div className={styles.toolbar}>
        <TextField.Root
          className={styles.filter}
          type="search"
          value={filter}
          placeholder={t('browse.versions.filterPlaceholder')}
          aria-label={t('browse.versions.filterLabel')}
          onChange={(event) => setFilter(event.target.value)}
        >
          <TextField.Slot>
            <Search size={14} aria-hidden="true" />
          </TextField.Slot>
        </TextField.Root>
        <Text as="label" size="2" className={styles.switch}>
          <Switch checked={hidePre} onCheckedChange={setHidePre} />
          {t('browse.versions.hidePre')}
        </Text>
      </div>
      {rows.length === 0 ? (
        <Text as="p" size="2" color="gray">
          {t('browse.versions.noMatches')}
        </Text>
      ) : (
        <>
          <div className={styles.scroll}>
            <Table.Root variant="surface" size="2" className={styles.table}>
              <Table.Header>
                <Table.Row>
                  <SortHeader
                    column="name"
                    label={docker ? t('browse.versions.column.tag') : t('browse.versions.column.version')}
                    sort={column}
                    direction={direction}
                    onChange={change}
                  />
                  <SortHeader
                    column="modified"
                    label={docker ? t('browse.versions.column.pushed') : t('browse.versions.column.uploaded')}
                    sort={column}
                    direction={direction}
                    onChange={change}
                  />
                  {docker && (
                    <SortHeader
                      column="downloads"
                      label={t('browse.versions.column.pulls')}
                      sort={column}
                      direction={direction}
                      onChange={change}
                    />
                  )}
                </Table.Row>
              </Table.Header>
              <Table.Body>
                {shown.map((node) => (
                  <Table.Row key={node.path}>
                    <Table.RowHeaderCell>
                      <div className={styles.name}>
                        <Link to={browseUrl(repository.name, node.path)} className={styles.link}>
                          {node.name}
                        </Link>
                        {isPreRelease(node, repository.format) && (
                          <span className={styles.pre}>{t('browse.version.prerelease')}</span>
                        )}
                        {node.path === newest && (
                          <Badge variant="soft" size="1">
                            {t('browse.versions.newest')}
                          </Badge>
                        )}
                      </div>
                    </Table.RowHeaderCell>
                    <Table.Cell>
                      {node.lastModified ? <RelativeTime value={node.lastModified} /> : '—'}
                    </Table.Cell>
                    {docker && <Table.Cell>{formatNumber(language, node.downloadCount ?? 0)}</Table.Cell>}
                  </Table.Row>
                ))}
              </Table.Body>
            </Table.Root>
          </div>
          {pageCount(rows.length, paging.size) > 1 && (
            <Pager
              total={rows.length}
              page={paging.page}
              size={paging.size}
              onPage={paging.setPage}
              onSize={paging.setSize}
            />
          )}
        </>
      )}
      {truncated && (
        <Text as="p" size="2" color="gray">
          {t('browse.versions.truncated')}
        </Text>
      )}
    </section>
  );
}
