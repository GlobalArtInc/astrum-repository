import {
  Badge,
  Button,
  Checkbox,
  DropdownMenu,
  IconButton,
  Select,
  Table,
  Text,
  TextField,
} from '@radix-ui/themes';
import { ArrowDown, ArrowUp, ArrowUpDown, Download, Ellipsis, RotateCcw, Search } from 'lucide-react';
import { type ReactNode, useMemo, useState } from 'react';
import { Link, useNavigate } from 'react-router';
import { formatNumber, useI18n, useT } from '../../../i18n';
import { DateTime, EmptyState, FileSize } from '../../../ui';
import { Pager, pageCount, pageSlice } from '../Pager';
import { browseUrl } from '../tree/model';
import type { RepositoryItem, SearchAsset, VersionEntry } from '../types';
import styles from './ComponentPage.module.css';

type Direction = 'asc' | 'desc';

export function csv(rows: string[][]): string {
  const cell = (value: string) => (/[",\n;]/.test(value) ? `"${value.replace(/"/g, '""')}"` : value);
  return `${rows.map((row) => row.map(cell).join(',')).join('\n')}\n`;
}

function saveCsv(name: string, rows: string[][]) {
  const url = URL.createObjectURL(new Blob([csv(rows)], { type: 'text/csv;charset=utf-8' }));
  const link = document.createElement('a');
  link.href = url;
  link.download = name;
  link.click();
  URL.revokeObjectURL(url);
}

function FilterColumn<K extends string>({
  sorts,
  sort,
  direction,
  onSort,
  onDirection,
  onReset,
  children,
}: {
  sorts: Array<[K, string]>;
  sort: K;
  direction: Direction;
  onSort(value: K): void;
  onDirection(value: Direction): void;
  onReset(): void;
  children?: ReactNode;
}) {
  const t = useT();
  return (
    <aside className={styles.filters} aria-label={t('browse.list.filters')}>
      <Button variant="outline" color="gray" className={styles.reset} onClick={onReset}>
        <RotateCcw size={14} aria-hidden="true" />
        {t('browse.list.reset')}
      </Button>
      <fieldset className={`${styles.facet} ${styles.sortFacet}`}>
        <legend>{t('browse.list.sortBy')}</legend>
        <Select.Root value={sort} onValueChange={(value) => onSort(value as K)}>
          <Select.Trigger aria-label={t('browse.list.sortBy')} />
          <Select.Content>
            {sorts.map(([value, label]) => (
              <Select.Item key={value} value={value}>
                {label}
              </Select.Item>
            ))}
          </Select.Content>
        </Select.Root>
        <Select.Root value={direction} onValueChange={(value) => onDirection(value as Direction)}>
          <Select.Trigger aria-label={t('browse.list.direction')} />
          <Select.Content>
            <Select.Item value="asc">{t('browse.list.ascending')}</Select.Item>
            <Select.Item value="desc">{t('browse.list.descending')}</Select.Item>
          </Select.Content>
        </Select.Root>
      </fieldset>
      {children}
    </aside>
  );
}

function Facet({
  title,
  options,
  chosen,
  onToggle,
}: {
  title: string;
  options: Array<[string, string]>;
  chosen: string[];
  onToggle(value: string): void;
}) {
  if (options.length === 0) {
    return null;
  }
  return (
    <fieldset className={styles.facet}>
      <legend>{title}</legend>
      {options.map(([value, label]) => (
        <Text as="label" size="2" key={value} className={styles.facetOption}>
          <Checkbox checked={chosen.includes(value)} onCheckedChange={() => onToggle(value)} />
          {label}
        </Text>
      ))}
    </fieldset>
  );
}

function toggled(values: string[], value: string): string[] {
  return values.includes(value) ? values.filter((known) => known !== value) : [...values, value];
}

function SortHeader<K extends string>({
  column,
  label,
  sort,
  direction,
  onChange,
}: {
  column: K;
  label: string;
  sort: K;
  direction: Direction;
  onChange(column: K, direction: Direction): void;
}) {
  const active = sort === column;
  const Arrow = !active ? ArrowUpDown : direction === 'asc' ? ArrowUp : ArrowDown;
  return (
    <Table.ColumnHeaderCell
      aria-sort={active ? (direction === 'asc' ? 'ascending' : 'descending') : undefined}
    >
      <button
        type="button"
        className={styles.sort}
        onClick={() => onChange(column, active && direction === 'asc' ? 'desc' : 'asc')}
      >
        {label}
        <Arrow size={14} aria-hidden="true" opacity={active ? 1 : 0.6} />
      </button>
    </Table.ColumnHeaderCell>
  );
}

function Toolbar({
  filter,
  placeholder,
  onFilter,
  action,
}: {
  filter: string;
  placeholder: string;
  onFilter(value: string): void;
  action: ReactNode;
}) {
  return (
    <div className={styles.toolbar}>
      <TextField.Root
        className={styles.filterText}
        type="search"
        value={filter}
        placeholder={placeholder}
        aria-label={placeholder}
        onChange={(event) => onFilter(event.target.value)}
      >
        <TextField.Slot>
          <Search size={14} aria-hidden="true" />
        </TextField.Slot>
      </TextField.Root>
      {action}
    </div>
  );
}

function ExportButton({ label, onClick, title }: { label: string; onClick(): void; title?: string }) {
  return (
    <Button variant="outline" color="gray" onClick={onClick} aria-label={title}>
      <Download size={14} aria-hidden="true" />
      {label}
    </Button>
  );
}

function usePaging(total: number, initial = 20) {
  const [page, setPage] = useState(0);
  const [size, setSize] = useState(initial);
  const current = Math.min(page, pageCount(total, size) - 1);
  return {
    page: current,
    size,
    setPage,
    setSize: (value: number) => {
      setPage(0);
      setSize(value);
    },
  };
}

type VersionSort = 'version' | 'modified';

export function VersionsTab({
  versions,
  current,
  hrefOf,
  compare,
}: {
  versions: VersionEntry[];
  current: string;
  hrefOf(version: string): string;
  compare(left: string, right: string): number;
}) {
  const t = useT();
  const { language } = useI18n();
  const navigate = useNavigate();
  const [sort, setSort] = useState<VersionSort>('version');
  const [direction, setDirection] = useState<Direction>('desc');
  const [filter, setFilter] = useState('');
  const rows = useMemo(() => {
    const wanted = filter.trim().toLowerCase();
    const sign = direction === 'asc' ? 1 : -1;
    return versions
      .filter((entry) => wanted === '' || entry.version.toLowerCase().includes(wanted))
      .sort((a, b) =>
        sort === 'version'
          ? sign * compare(a.version, b.version)
          : sign * ((a.lastModified ?? '') < (b.lastModified ?? '') ? -1 : 1),
      );
  }, [versions, filter, sort, direction, compare]);
  const paging = usePaging(rows.length);
  const shown = pageSlice(rows, paging.page, paging.size);
  const pages = pageCount(rows.length, paging.size);
  const exportPage = () =>
    saveCsv('versions.csv', [
      [
        t('browse.details.version'),
        t('browse.component.repositories'),
        t('browse.component.versionModified'),
      ],
      ...shown.map((entry) => [entry.version, entry.repositories.join(' '), entry.lastModified ?? '']),
    ]);
  return (
    <div className={styles.tab}>
      <FilterColumn<VersionSort>
        sorts={[
          ['version', t('browse.details.version')],
          ['modified', t('browse.component.versionModified')],
        ]}
        sort={sort}
        direction={direction}
        onSort={setSort}
        onDirection={setDirection}
        onReset={() => {
          setSort('version');
          setDirection('desc');
          setFilter('');
        }}
      />
      <div className={styles.tabMain}>
        <Toolbar
          filter={filter}
          placeholder={t('browse.component.versionFilter')}
          onFilter={setFilter}
          action={
            <ExportButton
              label={t('browse.component.exportPage')}
              title={t('browse.component.exportPageLabel', {
                page: formatNumber(language, paging.page + 1),
                pages: formatNumber(language, pages),
              })}
              onClick={exportPage}
            />
          }
        />
        {rows.length === 0 ? (
          <EmptyState title={t('browse.component.noVersions')} />
        ) : (
          <Table.Root variant="surface" size="2">
            <caption className="visually-hidden">{t('browse.component.versionsTab')}</caption>
            <Table.Header>
              <Table.Row>
                <SortHeader
                  column="version"
                  label={t('browse.details.version')}
                  sort={sort}
                  direction={direction}
                  onChange={(column, next) => {
                    setSort(column);
                    setDirection(next);
                  }}
                />
                <Table.ColumnHeaderCell>{t('browse.component.repositories')}</Table.ColumnHeaderCell>
                <SortHeader
                  column="modified"
                  label={t('browse.component.versionModified')}
                  sort={sort}
                  direction={direction}
                  onChange={(column, next) => {
                    setSort(column);
                    setDirection(next);
                  }}
                />
              </Table.Row>
            </Table.Header>
            <Table.Body>
              {shown.map((entry) => (
                <Table.Row
                  key={entry.version}
                  className={styles.row}
                  data-current={entry.version === current}
                  onClick={(event) => {
                    if (!(event.target as HTMLElement).closest('a')) {
                      navigate(hrefOf(entry.version));
                    }
                  }}
                >
                  <Table.RowHeaderCell>
                    <Link to={hrefOf(entry.version)} className={styles.plainLink}>
                      {entry.version}
                    </Link>
                  </Table.RowHeaderCell>
                  <Table.Cell>
                    <Badge color="gray" variant="soft" radius="full">
                      {t('browse.component.repositoryCount', { count: entry.repositories.length })}
                    </Badge>
                  </Table.Cell>
                  <Table.Cell>
                    {entry.lastModified ? <DateTime value={entry.lastModified} dateOnly /> : '—'}
                  </Table.Cell>
                </Table.Row>
              ))}
            </Table.Body>
          </Table.Root>
        )}
        <Pager
          total={rows.length}
          page={paging.page}
          size={paging.size}
          onPage={paging.setPage}
          onSize={paging.setSize}
        />
      </div>
    </div>
  );
}

type RepositorySort = 'repository' | 'type' | 'versions';

const TYPE_LABELS = {
  hosted: 'browse.list.type.hosted',
  proxy: 'browse.list.type.proxy',
  group: 'browse.list.type.group',
} as const;

interface RepositoryRow {
  name: string;
  type: RepositoryItem['type'];
  versions: number;
}

export function RepositoriesTab({
  versions,
  repositories,
}: {
  versions: VersionEntry[];
  repositories: RepositoryItem[];
}) {
  const t = useT();
  const [sort, setSort] = useState<RepositorySort>('repository');
  const [direction, setDirection] = useState<Direction>('asc');
  const [types, setTypes] = useState<string[]>([]);
  const [filter, setFilter] = useState('');
  const all = useMemo(() => {
    const counts = new Map<string, number>();
    for (const entry of versions) {
      for (const name of entry.repositories) {
        counts.set(name, (counts.get(name) ?? 0) + 1);
      }
    }
    return [...counts.entries()].map(
      ([name, count]): RepositoryRow => ({
        name,
        type: repositories.find((item) => item.name === name)?.type ?? 'hosted',
        versions: count,
      }),
    );
  }, [versions, repositories]);
  const rows = useMemo(() => {
    const wanted = filter.trim().toLowerCase();
    const sign = direction === 'asc' ? 1 : -1;
    return all
      .filter((row) => types.length === 0 || types.includes(row.type))
      .filter((row) => wanted === '' || row.name.toLowerCase().includes(wanted))
      .sort((a, b) => {
        if (sort === 'versions') {
          return sign * (a.versions - b.versions);
        }
        const left = sort === 'type' ? a.type : a.name;
        const right = sort === 'type' ? b.type : b.name;
        return sign * left.localeCompare(right);
      });
  }, [all, types, filter, sort, direction]);
  const paging = usePaging(rows.length);
  const shown = pageSlice(rows, paging.page, paging.size);
  const change = (column: RepositorySort, next: Direction) => {
    setSort(column);
    setDirection(next);
  };
  return (
    <div className={styles.tab}>
      <FilterColumn<RepositorySort>
        sorts={[
          ['repository', t('browse.details.repository')],
          ['type', t('browse.list.type')],
          ['versions', t('browse.component.versionsInRepository')],
        ]}
        sort={sort}
        direction={direction}
        onSort={setSort}
        onDirection={setDirection}
        onReset={() => {
          setSort('repository');
          setDirection('asc');
          setTypes([]);
          setFilter('');
        }}
      >
        <Facet
          title={t('browse.list.type')}
          options={[
            ['hosted', t(TYPE_LABELS.hosted)],
            ['proxy', t(TYPE_LABELS.proxy)],
            ['group', t(TYPE_LABELS.group)],
          ]}
          chosen={types}
          onToggle={(value) => setTypes(toggled(types, value))}
        />
      </FilterColumn>
      <div className={styles.tabMain}>
        <Toolbar
          filter={filter}
          placeholder={t('browse.component.filter')}
          onFilter={setFilter}
          action={
            <ExportButton
              label={t('browse.component.exportCsv')}
              title={t('browse.component.exportCsvLabel')}
              onClick={() =>
                saveCsv('repositories.csv', [
                  [
                    t('browse.details.repository'),
                    t('browse.list.type'),
                    t('browse.component.versionsInRepository'),
                  ],
                  ...rows.map((row) => [row.name, t(TYPE_LABELS[row.type]), String(row.versions)]),
                ])
              }
            />
          }
        />
        {rows.length === 0 ? (
          <EmptyState title={t('browse.component.noRepositories')} />
        ) : (
          <Table.Root variant="surface" size="2">
            <caption className="visually-hidden">{t('browse.component.repositoriesTab')}</caption>
            <Table.Header>
              <Table.Row>
                <SortHeader
                  column="repository"
                  label={t('browse.details.repository')}
                  sort={sort}
                  direction={direction}
                  onChange={change}
                />
                <SortHeader
                  column="type"
                  label={t('browse.list.type')}
                  sort={sort}
                  direction={direction}
                  onChange={change}
                />
                <SortHeader
                  column="versions"
                  label={t('browse.component.versionsInRepository')}
                  sort={sort}
                  direction={direction}
                  onChange={change}
                />
              </Table.Row>
            </Table.Header>
            <Table.Body>
              {shown.map((row) => (
                <Table.Row key={row.name}>
                  <Table.RowHeaderCell>
                    <Link to={browseUrl(row.name)} className={styles.plainLink}>
                      {row.name}
                    </Link>
                  </Table.RowHeaderCell>
                  <Table.Cell>{t(TYPE_LABELS[row.type])}</Table.Cell>
                  <Table.Cell>
                    <Badge color="gray" variant="soft" radius="full">
                      {t('browse.component.versionCount', { count: row.versions })}
                    </Badge>
                  </Table.Cell>
                </Table.Row>
              ))}
            </Table.Body>
          </Table.Root>
        )}
        <Pager
          total={rows.length}
          page={paging.page}
          size={paging.size}
          onPage={paging.setPage}
          onSize={paging.setSize}
        />
      </div>
    </div>
  );
}

export interface FileRow extends SearchAsset {
  key: string;
}

type FileSort = 'file' | 'type' | 'size' | 'modified';

export function fileName(path: string): string {
  return path.split('/').pop() ?? path;
}

export function fileType(path: string): string {
  const name = fileName(path);
  const dot = name.lastIndexOf('.');
  return dot > 0 ? name.slice(dot + 1) : '';
}

function FileMenu({ row, treePath }: { row: FileRow; treePath: string }) {
  const t = useT();
  const navigate = useNavigate();
  return (
    <DropdownMenu.Root>
      <DropdownMenu.Trigger>
        <IconButton
          variant="ghost"
          color="gray"
          size="1"
          aria-label={t('browse.component.rowActions', { name: fileName(row.path) })}
        >
          <Ellipsis size={16} aria-hidden="true" />
        </IconButton>
      </DropdownMenu.Trigger>
      <DropdownMenu.Content align="end">
        <DropdownMenu.Item onSelect={() => navigate(browseUrl(row.repository, treePath))}>
          {t('browse.component.openInRepository')}
        </DropdownMenu.Item>
        <DropdownMenu.Item asChild>
          <a href={row.downloadUrl} download>
            {t('browse.details.download')}
          </a>
        </DropdownMenu.Item>
      </DropdownMenu.Content>
    </DropdownMenu.Root>
  );
}

export function FilesTab({ rows: all }: { rows: FileRow[] }) {
  const t = useT();
  const [sort, setSort] = useState<FileSort>('file');
  const [direction, setDirection] = useState<Direction>('desc');
  const [types, setTypes] = useState<string[]>([]);
  const [filter, setFilter] = useState('');
  const options = useMemo(
    () =>
      [...new Set(all.map((row) => fileType(row.path)).filter(Boolean))]
        .sort()
        .map((value): [string, string] => [value, value]),
    [all],
  );
  const rows = useMemo(() => {
    const wanted = filter.trim().toLowerCase();
    const sign = direction === 'asc' ? 1 : -1;
    const value = (row: FileRow): string | number =>
      sort === 'size'
        ? row.fileSize
        : sort === 'modified'
          ? row.lastModified
          : sort === 'type'
            ? fileType(row.path)
            : fileName(row.path);
    return all
      .filter((row) => types.length === 0 || types.includes(fileType(row.path)))
      .filter((row) => wanted === '' || fileName(row.path).toLowerCase().includes(wanted))
      .sort((a, b) => {
        const left = value(a);
        const right = value(b);
        return (
          sign *
          (typeof left === 'number' && typeof right === 'number'
            ? left - right
            : String(left).localeCompare(String(right)))
        );
      });
  }, [all, types, filter, sort, direction]);
  const paging = usePaging(rows.length);
  const shown = pageSlice(rows, paging.page, paging.size);
  const change = (column: FileSort, next: Direction) => {
    setSort(column);
    setDirection(next);
  };
  return (
    <div className={styles.tab}>
      <FilterColumn<FileSort>
        sorts={[
          ['file', t('browse.component.file')],
          ['type', t('browse.list.type')],
          ['size', t('browse.details.size')],
          ['modified', t('browse.details.updated')],
        ]}
        sort={sort}
        direction={direction}
        onSort={setSort}
        onDirection={setDirection}
        onReset={() => {
          setSort('file');
          setDirection('desc');
          setTypes([]);
          setFilter('');
        }}
      >
        <Facet
          title={t('browse.list.type')}
          options={options}
          chosen={types}
          onToggle={(value) => setTypes(toggled(types, value))}
        />
      </FilterColumn>
      <div className={styles.tabMain}>
        <Toolbar
          filter={filter}
          placeholder={t('browse.component.filter')}
          onFilter={setFilter}
          action={
            <ExportButton
              label={t('browse.component.exportCsv')}
              title={t('browse.component.exportCsvLabel')}
              onClick={() =>
                saveCsv('files.csv', [
                  [
                    t('browse.component.file'),
                    t('browse.details.repository'),
                    t('browse.list.type'),
                    t('browse.details.size'),
                    t('browse.details.updated'),
                  ],
                  ...rows.map((row) => [
                    fileName(row.path),
                    row.repository,
                    fileType(row.path),
                    String(row.fileSize),
                    row.lastModified,
                  ]),
                ])
              }
            />
          }
        />
        {rows.length === 0 ? (
          <EmptyState title={t('browse.component.noFiles')} />
        ) : (
          <Table.Root variant="surface" size="2">
            <caption className="visually-hidden">{t('browse.component.files')}</caption>
            <Table.Header>
              <Table.Row>
                <SortHeader
                  column="file"
                  label={t('browse.component.file')}
                  sort={sort}
                  direction={direction}
                  onChange={change}
                />
                <SortHeader
                  column="type"
                  label={t('browse.list.type')}
                  sort={sort}
                  direction={direction}
                  onChange={change}
                />
                <SortHeader
                  column="size"
                  label={t('browse.details.size')}
                  sort={sort}
                  direction={direction}
                  onChange={change}
                />
                <SortHeader
                  column="modified"
                  label={t('browse.details.updated')}
                  sort={sort}
                  direction={direction}
                  onChange={change}
                />
                <Table.ColumnHeaderCell width="56px">
                  <span className="visually-hidden">{t('browse.list.actions')}</span>
                </Table.ColumnHeaderCell>
              </Table.Row>
            </Table.Header>
            <Table.Body>
              {shown.map((row) => (
                <Table.Row key={row.key}>
                  <Table.RowHeaderCell>
                    <span className={styles.fileCell}>
                      <span className={styles.fileName}>{fileName(row.path)}</span>
                      <Badge color="gray" variant="solid" radius="full" size="1" className={styles.repoBadge}>
                        {row.repository}
                      </Badge>
                    </span>
                  </Table.RowHeaderCell>
                  <Table.Cell>
                    <Text color="gray">{fileType(row.path) || '—'}</Text>
                  </Table.Cell>
                  <Table.Cell>
                    <FileSize bytes={row.fileSize} />
                  </Table.Cell>
                  <Table.Cell>
                    <DateTime value={row.lastModified} dateOnly />
                  </Table.Cell>
                  <Table.Cell align="right">
                    <FileMenu row={row} treePath={row.path} />
                  </Table.Cell>
                </Table.Row>
              ))}
            </Table.Body>
          </Table.Root>
        )}
        <Pager
          total={rows.length}
          page={paging.page}
          size={paging.size}
          onPage={paging.setPage}
          onSize={paging.setSize}
        />
      </div>
    </div>
  );
}
