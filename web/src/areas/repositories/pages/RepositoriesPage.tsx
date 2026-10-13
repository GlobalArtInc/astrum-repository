import { Button, DropdownMenu, IconButton, Select, Text, TextField } from '@radix-ui/themes';
import { ArrowUpDown, Copy, Ellipsis, Plus, Search } from 'lucide-react';
import { useMemo, useState } from 'react';
import { Link, useNavigate, useSearchParams } from 'react-router';
import type { RepositoryType } from '../../../api';
import { useCan } from '../../../auth/state';
import { type Translate, useT } from '../../../i18n';
import { FileSize, formatName, Page, useToast } from '../../../ui';
import { CREATE_ADDRESS, editAddress, type UiRepository, useUiRepositories } from '../api';
import { type AdminColumn, AdminTable, compareText } from '../components/admin/AdminTable';
import styles from '../components/admin/admin.module.css';
import {
  CheckGroup,
  countBy,
  type Direction,
  FilterPanel,
  ResetFilters,
  SortGroup,
  toggled,
} from '../components/admin/FilterPanel';
import { HelpCard } from '../components/admin/HelpCard';
import { Pager } from '../components/admin/Pager';
import { canInvalidate, canRebuild, useRepositoryActions } from '../components/useRepositoryActions';

export const HEALTH_CHECK_FORMATS = new Set([
  'maven2',
  'npm',
  'nuget',
  'pypi',
  'rubygems',
  'cocoapods',
  'conan',
  'conda',
  'go',
  'r',
  'apt',
]);

const TYPE_LABELS: Record<RepositoryType, string> = {
  hosted: 'ui.repositoryType.hosted',
  proxy: 'ui.repositoryType.proxy',
  group: 'ui.repositoryType.group',
};

export type SortField = 'name' | 'type' | 'format' | 'status';

export interface Filters {
  q: string;
  formats: ReadonlySet<string>;
  types: ReadonlySet<string>;
  statuses: ReadonlySet<string>;
}

function statusOf(row: UiRepository): string {
  return row.online ? 'online' : 'offline';
}

export function filterRepositories(rows: readonly UiRepository[], filters: Filters): UiRepository[] {
  const wanted = filters.q.trim().toLowerCase();
  return rows.filter(
    (row) =>
      (filters.formats.size === 0 || filters.formats.has(row.format)) &&
      (filters.types.size === 0 || filters.types.has(row.type)) &&
      (filters.statuses.size === 0 || filters.statuses.has(statusOf(row))) &&
      (wanted === '' || row.name.toLowerCase().includes(wanted)),
  );
}

export function sortRepositories(
  rows: readonly UiRepository[],
  field: SortField,
  direction: Direction,
  t: Translate,
): UiRepository[] {
  const value = (row: UiRepository): string => {
    switch (field) {
      case 'type':
        return t(TYPE_LABELS[row.type]);
      case 'format':
        return formatName(row.format);
      case 'status':
        return row.online ? t('repositories.status.online') : t('repositories.status.offline');
      default:
        return row.name;
    }
  };
  const sign = direction === 'asc' ? 1 : -1;
  return [...rows].sort(
    (left, right) => sign * compareText(value(left), value(right)) || compareText(left.name, right.name),
  );
}

function listParam(params: URLSearchParams, key: string): Set<string> {
  return new Set((params.get(key) ?? '').split(',').filter((value) => value !== ''));
}

export default function RepositoriesPage() {
  const t = useT();
  const toast = useToast();
  const navigate = useNavigate();
  const [params, setParams] = useSearchParams();
  const canCreate = useCan({ some: 'astrum:repository-admin:*:*:add' });
  const query = useUiRepositories();
  const actions = useRepositoryActions();
  const [sortField, setSortField] = useState<SortField>('name');
  const [direction, setDirection] = useState<Direction>('asc');
  const [pageSize, setPageSize] = useState(40);
  const [page, setPage] = useState(0);

  const filters: Filters = {
    q: params.get('q') ?? '',
    formats: listParam(params, 'format'),
    types: listParam(params, 'type'),
    statuses: listParam(params, 'status'),
  };
  const update = (key: string, value: string) => {
    setPage(0);
    setParams(
      (previous) => {
        const next = new URLSearchParams(previous);
        if (value === '') {
          next.delete(key);
        } else {
          next.set(key, value);
        }
        return next;
      },
      { replace: true },
    );
  };
  const toggle = (key: 'format' | 'type' | 'status', current: ReadonlySet<string>, value: string) =>
    update(key, [...toggled(current, value)].join(','));

  const administered = useMemo(() => (query.data ?? []).filter((row) => row.permissions.admin), [query.data]);
  const matching = sortRepositories(filterRepositories(administered, filters), sortField, direction, t);
  const lastPage = Math.max(0, Math.ceil(matching.length / pageSize) - 1);
  const current = Math.min(page, lastPage);
  const visible = matching.slice(current * pageSize, (current + 1) * pageSize);

  const sortOptions = [
    { value: 'name', label: t('repositories.list.name') },
    { value: 'type', label: t('repositories.list.type') },
    { value: 'format', label: t('repositories.list.format') },
    { value: 'status', label: t('repositories.list.status') },
  ];
  const sortBy = (column: string) => {
    if (column === sortField) {
      setDirection(direction === 'asc' ? 'desc' : 'asc');
    } else {
      setSortField(column as SortField);
      setDirection('asc');
    }
  };

  const columns: AdminColumn<UiRepository>[] = [
    {
      id: 'name',
      header: t('repositories.list.name'),
      sortable: true,
      cell: (row) => <span className={styles.name}>{row.name}</span>,
    },
    { id: 'size', header: t('repositories.list.size'), cell: (row) => <SizeCell row={row} /> },
    {
      id: 'type',
      header: t('repositories.list.type'),
      sortable: true,
      cell: (row) => t(TYPE_LABELS[row.type]),
    },
    {
      id: 'format',
      header: t('repositories.list.ecosystem'),
      sortable: true,
      cell: (row) => formatName(row.format),
    },
    {
      id: 'blobStore',
      header: t('repositories.list.blobStore'),
      cell: (row) => <span className={styles.muted}>{row.blobStoreName ?? '—'}</span>,
    },
    {
      id: 'status',
      header: t('repositories.list.status'),
      sortable: true,
      cell: (row) => (row.online ? t('repositories.status.online') : t('repositories.status.offline')),
    },
    {
      id: 'url',
      header: t('repositories.list.url'),
      cell: (row) => (
        <IconButton
          variant="ghost"
          color="gray"
          size="1"
          aria-label={t('repositories.list.copyUrl')}
          onClick={() => void actions.copyUrl(row)}
        >
          <Copy size={16} aria-hidden="true" />
        </IconButton>
      ),
    },
    {
      id: 'health',
      header: t('repositories.list.healthCheck'),
      cell: (row) =>
        row.type === 'proxy' && HEALTH_CHECK_FORMATS.has(row.format) ? (
          <Button
            size="1"
            variant="soft"
            onClick={() => toast.error(t('repositories.list.healthCheckUnavailable'))}
          >
            {t('repositories.list.analyze')}
          </Button>
        ) : (
          <span className={styles.small}>{t('repositories.list.notSupported')}</span>
        ),
    },
    {
      id: 'actions',
      header: '',
      label: t('repositories.list.rowActions'),
      width: '40px',
      cell: (row) => (
        <DropdownMenu.Root>
          <DropdownMenu.Trigger>
            <IconButton
              variant="ghost"
              color="gray"
              size="1"
              aria-label={t('repositories.list.actionsFor', { name: row.name })}
            >
              <Ellipsis size={16} aria-hidden="true" />
            </IconButton>
          </DropdownMenu.Trigger>
          <DropdownMenu.Content align="end">
            <DropdownMenu.Item onSelect={() => navigate(editAddress(row.name))}>
              {t('repositories.actions.edit')}
            </DropdownMenu.Item>
            {canRebuild(row) && (
              <DropdownMenu.Item onSelect={() => void actions.maintain(row, 'rebuild-index')}>
                {t('repositories.actions.rebuild')}
              </DropdownMenu.Item>
            )}
            {canInvalidate(row) && (
              <DropdownMenu.Item onSelect={() => void actions.maintain(row, 'invalidate-cache')}>
                {t('repositories.actions.invalidate')}
              </DropdownMenu.Item>
            )}
            {row.permissions.adminEdit && (
              <DropdownMenu.Item onSelect={() => void actions.toggleOnline(row)}>
                {row.online ? t('repositories.actions.takeOffline') : t('repositories.actions.bringOnline')}
              </DropdownMenu.Item>
            )}
            <DropdownMenu.Item onSelect={() => void actions.copyUrl(row)}>
              {t('repositories.actions.copyUrl')}
            </DropdownMenu.Item>
            {row.permissions.adminDelete && (
              <DropdownMenu.Item color="red" onSelect={() => actions.remove(row)}>
                {t('repositories.actions.delete')}
              </DropdownMenu.Item>
            )}
          </DropdownMenu.Content>
        </DropdownMenu.Root>
      ),
    },
  ];

  const typeOptions = countBy(
    administered,
    (row) => row.type,
    (type) => t(TYPE_LABELS[type as RepositoryType]),
    'value',
  );
  const statusOptions = countBy(administered, statusOf, (status) =>
    status === 'online' ? t('repositories.status.online') : t('repositories.status.offline'),
  );

  return (
    <Page
      title={t('repositories.list.title')}
      description={t('repositories.list.description')}
      breadcrumbs={[
        { label: t('repositories.crumbs.settings'), to: '/settings' },
        { label: t('repositories.crumbs.repositories') },
      ]}
      actions={
        canCreate && (
          <Button asChild>
            <Link to={CREATE_ADDRESS}>
              <Plus size={16} aria-hidden="true" />
              {t('repositories.list.create')}
            </Link>
          </Button>
        )
      }
    >
      <div className={styles.layout}>
        <FilterPanel>
          <ResetFilters
            onReset={() => {
              setParams(new URLSearchParams(), { replace: true });
              setSortField('name');
              setDirection('asc');
            }}
          />
          <SortGroup
            options={sortOptions}
            value={sortField}
            direction={direction}
            onChange={(field, next) => {
              setSortField(field as SortField);
              setDirection(next);
            }}
          />
          <CheckGroup
            title={t('repositories.list.format')}
            options={countBy(administered, (row) => row.format, formatName)}
            selected={filters.formats}
            onToggle={(value) => toggle('format', filters.formats, value)}
            limit={7}
          />
          <CheckGroup
            title={t('repositories.list.type')}
            options={typeOptions}
            selected={filters.types}
            onToggle={(value) => toggle('type', filters.types, value)}
          />
          <CheckGroup
            title={t('repositories.list.state')}
            options={statusOptions}
            selected={filters.statuses}
            onToggle={(value) => toggle('status', filters.statuses, value)}
          />
        </FilterPanel>
        <div className={styles.main}>
          <div className={styles.toolbar} role="search">
            <TextField.Root
              className={styles.search}
              type="search"
              value={filters.q}
              placeholder={t('repositories.list.search')}
              aria-label={t('repositories.list.search')}
              onChange={(event) => update('q', event.target.value)}
            >
              <TextField.Slot>
                <Search size={14} aria-hidden="true" />
              </TextField.Slot>
            </TextField.Root>
            <Select.Root value={sortField} onValueChange={(value) => setSortField(value as SortField)}>
              <Select.Trigger className={styles.sortButton} aria-label={t('repositories.admin.sortField')}>
                <span className={styles.sortLabel}>
                  <ArrowUpDown size={14} aria-hidden="true" />
                  <span>{t('repositories.admin.sortBy')}</span>
                  <span>{sortOptions.find((option) => option.value === sortField)?.label}</span>
                </span>
              </Select.Trigger>
              <Select.Content position="popper" align="end">
                {sortOptions.map((option) => (
                  <Select.Item key={option.value} value={option.value}>
                    {option.label}
                  </Select.Item>
                ))}
              </Select.Content>
            </Select.Root>
          </div>
          {query.data && (
            <p className={styles.shown} aria-live="polite">
              {t('repositories.list.shown', { shown: visible.length, total: matching.length })}
            </p>
          )}
          {query.isPending ? (
            <Text size="2" color="gray">
              {t('repositories.list.loading')}
            </Text>
          ) : query.error ? (
            <Text size="2" color="red">
              {t('repositories.list.loadFailed')}
            </Text>
          ) : (
            <AdminTable
              label={t('repositories.list.label')}
              columns={columns}
              rows={visible}
              getRowId={(row) => row.name}
              sort={{ column: sortField, direction }}
              onSort={sortBy}
              onOpen={(row) => navigate(editAddress(row.name))}
              openLabel={(row) => t('repositories.list.open', { name: row.name })}
              empty={
                administered.length === 0 ? t('repositories.list.empty') : t('repositories.list.noMatches')
              }
              footer={
                <Pager
                  total={matching.length}
                  page={current}
                  pageSize={pageSize}
                  onPage={setPage}
                  onPageSize={(size) => {
                    setPageSize(size);
                    setPage(0);
                  }}
                />
              }
            />
          )}
          <HelpCard title={t('repositories.list.about')}>{t('repositories.list.aboutText')}</HelpCard>
        </div>
      </div>
      {actions.dialog}
    </Page>
  );
}

function SizeCell({ row }: { row: UiRepository }) {
  return (
    <span className={styles.muted}>{row.totalSize === null ? '—' : <FileSize bytes={row.totalSize} />}</span>
  );
}
