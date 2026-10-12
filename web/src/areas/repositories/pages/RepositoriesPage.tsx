import { Button, Flex, Select, Text, TextField } from '@radix-ui/themes';
import { Database, FolderOpen, Plus, Search, Settings2 } from 'lucide-react';
import { useMemo } from 'react';
import { Link, useNavigate, useSearchParams } from 'react-router';
import type { RepositoryType } from '../../../api';
import { useCan } from '../../../auth/state';
import { type Translate, useT } from '../../../i18n';
import {
  type Column,
  DataTable,
  EmptyState,
  FileSize,
  FormatBadge,
  formatName,
  Page,
  TypeBadge,
} from '../../../ui';
import { browseAddress, CREATE_ADDRESS, editAddress, type UiRepository, useUiRepositories } from '../api';
import { Count, RepositoryUrl, Status } from '../components/bits';
import { TYPES } from '../model/spec';
import styles from '../repositories.module.css';

const ALL = 'all';

export interface Filters {
  q: string;
  format: string;
  type: string;
  status: string;
}

export function filterRepositories(rows: readonly UiRepository[], filters: Filters): UiRepository[] {
  const wanted = filters.q.trim().toLowerCase();
  return rows.filter((row) => {
    if (filters.format !== ALL && row.format !== filters.format) {
      return false;
    }
    if (filters.type !== ALL && row.type !== filters.type) {
      return false;
    }
    if (filters.status !== ALL && (filters.status === 'online') !== row.online) {
      return false;
    }
    if (wanted === '') {
      return true;
    }
    return [row.name, row.url, row.blobStoreName ?? '', row.remoteUrl ?? ''].some((text) =>
      text.toLowerCase().includes(wanted),
    );
  });
}

function columns(t: Translate): Column<UiRepository>[] {
  return [
    {
      id: 'name',
      header: t('repositories.list.name'),
      cell: (row) => <span className={styles.name}>{row.name}</span>,
      sortValue: (row) => row.name,
      primary: true,
    },
    {
      id: 'format',
      header: t('repositories.list.format'),
      cell: (row) => <FormatBadge format={row.format} />,
      sortValue: (row) => formatName(row.format),
    },
    {
      id: 'type',
      header: t('repositories.list.type'),
      cell: (row) => <TypeBadge type={row.type} />,
      sortValue: (row) => row.type,
    },
    {
      id: 'status',
      header: t('repositories.list.status'),
      cell: (row) => <Status online={row.online} />,
      sortValue: (row) => (row.online ? 0 : 1),
    },
    {
      id: 'blobStore',
      header: t('repositories.list.blobStore'),
      cell: (row) => row.blobStoreName ?? '—',
      sortValue: (row) => row.blobStoreName,
      hideOnPhone: true,
    },
    {
      id: 'url',
      header: t('repositories.list.url'),
      cell: (row) => <RepositoryUrl name={row.name} url={row.url} compact />,
    },
    {
      id: 'size',
      header: t('repositories.list.size'),
      cell: (row) => (row.totalSize === null ? '—' : <FileSize bytes={row.totalSize} />),
      sortValue: (row) => row.totalSize,
      align: 'right',
    },
    {
      id: 'components',
      header: t('repositories.list.components'),
      cell: (row) => <Count value={row.componentCount} />,
      sortValue: (row) => row.componentCount,
      align: 'right',
      hideOnPhone: true,
    },
    {
      id: 'assets',
      header: t('repositories.list.assets'),
      cell: (row) => <Count value={row.assetCount} />,
      sortValue: (row) => row.assetCount,
      align: 'right',
      hideOnPhone: true,
    },
  ];
}

function FilterSelect({
  label,
  value,
  onChange,
  options,
}: {
  label: string;
  value: string;
  onChange(value: string): void;
  options: Array<{ value: string; label: string }>;
}) {
  return (
    <Flex direction="column" gap="1" className={styles.filter}>
      <Text size="1" color="gray" aria-hidden="true">
        {label}
      </Text>
      <Select.Root value={value} onValueChange={onChange}>
        <Select.Trigger aria-label={label} />
        <Select.Content position="popper">
          {options.map((option) => (
            <Select.Item key={option.value} value={option.value}>
              {option.label}
            </Select.Item>
          ))}
        </Select.Content>
      </Select.Root>
    </Flex>
  );
}

const TYPE_LABELS: Record<RepositoryType, string> = {
  hosted: 'ui.repositoryType.hosted',
  proxy: 'ui.repositoryType.proxy',
  group: 'ui.repositoryType.group',
};

export default function RepositoriesPage() {
  const t = useT();
  const navigate = useNavigate();
  const [params, setParams] = useSearchParams();
  const canCreate = useCan({ some: 'astrum:repository-admin:*:*:add' });
  const query = useUiRepositories();
  const filters: Filters = {
    q: params.get('q') ?? '',
    format: params.get('format') ?? ALL,
    type: params.get('type') ?? ALL,
    status: params.get('status') ?? ALL,
  };
  const setFilter = (key: keyof Filters, value: string) => {
    setParams(
      (previous) => {
        const next = new URLSearchParams(previous);
        if (value === '' || value === ALL) {
          next.delete(key);
        } else {
          next.set(key, value);
        }
        return next;
      },
      { replace: true },
    );
  };
  const administered = useMemo(() => (query.data ?? []).filter((row) => row.permissions.admin), [query.data]);
  const rows = filterRepositories(administered, filters);
  const formats = [...new Set(administered.map((row) => row.format))].sort((left, right) =>
    formatName(left).localeCompare(formatName(right)),
  );
  const filtered =
    filters.q !== '' || filters.format !== ALL || filters.type !== ALL || filters.status !== ALL;

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
      <div className={styles.toolbar} role="search">
        <TextField.Root
          className={styles.search}
          type="search"
          value={filters.q}
          placeholder={t('repositories.list.search')}
          aria-label={t('repositories.list.search')}
          onChange={(event) => setFilter('q', event.target.value)}
        >
          <TextField.Slot>
            <Search size={16} aria-hidden="true" />
          </TextField.Slot>
        </TextField.Root>
        <FilterSelect
          label={t('repositories.list.format')}
          value={filters.format}
          onChange={(value) => setFilter('format', value)}
          options={[
            { value: ALL, label: t('repositories.list.allFormats') },
            ...formats.map((format) => ({ value: format, label: formatName(format) })),
          ]}
        />
        <FilterSelect
          label={t('repositories.list.type')}
          value={filters.type}
          onChange={(value) => setFilter('type', value)}
          options={[
            { value: ALL, label: t('repositories.list.allTypes') },
            ...TYPES.map((type) => ({ value: type, label: t(TYPE_LABELS[type]) })),
          ]}
        />
        <FilterSelect
          label={t('repositories.list.status')}
          value={filters.status}
          onChange={(value) => setFilter('status', value)}
          options={[
            { value: ALL, label: t('repositories.list.anyStatus') },
            { value: 'online', label: t('repositories.status.online') },
            { value: 'offline', label: t('repositories.status.offline') },
          ]}
        />
      </div>
      {query.data && (
        <Text size="2" color="gray" aria-live="polite">
          {t('repositories.list.count', { count: rows.length })}
        </Text>
      )}
      <DataTable
        label={t('repositories.list.label')}
        columns={columns(t)}
        rows={query.data ? rows : undefined}
        getRowId={(row) => row.name}
        loading={query.isPending}
        error={query.error}
        onRetry={() => void query.refetch()}
        rowHref={(row) => editAddress(row.name)}
        initialSort={{ column: 'name', direction: 'asc' }}
        pageSize={50}
        paging="more"
        rowActions={(row) => [
          {
            id: 'edit',
            label: t('repositories.list.edit'),
            icon: Settings2,
            onSelect: () => navigate(editAddress(row.name)),
          },
          ...(row.permissions.browse || row.permissions.read
            ? [
                {
                  id: 'browse',
                  label: t('repositories.list.browse'),
                  icon: FolderOpen,
                  onSelect: () => navigate(browseAddress(row.name)),
                },
              ]
            : []),
        ]}
        empty={
          filtered ? (
            <EmptyState
              icon={Search}
              title={t('repositories.list.noMatches')}
              action={
                <Button variant="soft" onClick={() => setParams(new URLSearchParams(), { replace: true })}>
                  {t('repositories.list.clear')}
                </Button>
              }
            />
          ) : (
            <EmptyState
              icon={Database}
              title={t('repositories.list.empty')}
              description={t('repositories.list.emptyText')}
              action={
                canCreate && (
                  <Button asChild>
                    <Link to={CREATE_ADDRESS}>{t('repositories.list.create')}</Link>
                  </Button>
                )
              }
            />
          )
        }
      />
    </Page>
  );
}
