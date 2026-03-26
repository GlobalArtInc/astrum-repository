import { Button, Text, TextField } from '@radix-ui/themes';
import { ChevronRight, Circle, Plus, Search, Server } from 'lucide-react';
import { useState } from 'react';
import { Link, useNavigate } from 'react-router';
import { useCan } from '../../../auth/state';
import { formatNumber, useI18n } from '../../../i18n';
import { FileSize, Page } from '../../../ui';
import { type AdminColumn, AdminTable, compareText } from '../../repositories/components/admin/AdminTable';
import type { Direction } from '../../repositories/components/admin/FilterPanel';
import { HelpCard } from '../../repositories/components/admin/HelpCard';
import { storeTypeName } from '../../repositories/components/EditTabs';
import { CREATE_ADDRESS, storeAddress, storeType, type UiBlobStore, useUiBlobStores } from '../api';
import styles from '../blobstores.module.css';

type SortKey = 'name' | 'path' | 'type' | 'state' | 'blobs' | 'size' | 'available';

export function filterStores(rows: readonly UiBlobStore[], query: string): UiBlobStore[] {
  const wanted = query.trim().toLowerCase();
  return rows.filter((row) => wanted === '' || row.name.toLowerCase().includes(wanted));
}

export function location(store: UiBlobStore): string {
  return store.path ?? store.bucket ?? '—';
}

function sortStores(rows: UiBlobStore[], key: SortKey, direction: Direction): UiBlobStore[] {
  const sign = direction === 'asc' ? 1 : -1;
  const number = (row: UiBlobStore): number =>
    key === 'blobs'
      ? row.blobCount
      : key === 'size'
        ? row.totalSize
        : (row.availableSpace ?? Number.MAX_SAFE_INTEGER);
  return [...rows].sort((left, right) => {
    if (key === 'blobs' || key === 'size' || key === 'available') {
      return sign * (number(left) - number(right));
    }
    const text = (row: UiBlobStore) =>
      key === 'path'
        ? location(row)
        : key === 'type'
          ? row.type
          : key === 'state'
            ? String(row.unavailable)
            : row.name;
    return sign * compareText(text(left), text(right));
  });
}

export default function BlobStoresPage() {
  const { t, language } = useI18n();
  const navigate = useNavigate();
  const canCreate = useCan('astrum:blobstores:create');
  const query = useUiBlobStores();
  const [filter, setFilter] = useState('');
  const [sort, setSort] = useState<{ column: SortKey; direction: Direction }>({
    column: 'name',
    direction: 'asc',
  });
  const rows = sortStores(filterStores(query.data ?? [], filter), sort.column, sort.direction);
  const open = (row: UiBlobStore) => {
    const kind = storeType(row.type);
    if (kind) {
      navigate(storeAddress(kind, row.name));
    }
  };

  const columns: AdminColumn<UiBlobStore>[] = [
    {
      id: 'name',
      header: t('blobstores.list.name'),
      sortable: true,
      cell: (row) => (
        <span className={styles.nameCell}>
          <Server size={16} aria-hidden="true" />
          {row.name}
        </span>
      ),
    },
    { id: 'path', header: t('blobstores.list.path'), cell: (row) => location(row) },
    { id: 'type', header: t('blobstores.list.type'), cell: (row) => storeTypeName(row.type) },
    {
      id: 'state',
      header: t('blobstores.list.state'),
      cell: (row) => (
        <span className={styles.state}>
          <Circle
            size={6}
            fill="currentColor"
            aria-hidden="true"
            style={{ color: row.unavailable ? 'var(--red-9)' : 'var(--green-9)' }}
          />
          {row.unavailable ? t('blobstores.list.stateUnavailable') : t('blobstores.list.stateAvailable')}
        </span>
      ),
    },
    {
      id: 'blobs',
      header: <span className={styles.right}>{t('blobstores.list.blobCount')}</span>,
      cell: (row) => <div className={styles.right}>{formatNumber(language, row.blobCount)}</div>,
    },
    {
      id: 'size',
      header: t('blobstores.list.size'),
      cell: (row) => (
        <div className={styles.right}>
          <FileSize bytes={row.totalSize} />
        </div>
      ),
    },
    {
      id: 'available',
      header: t('blobstores.list.available'),
      cell: (row) => (
        <div className={styles.right}>
          {row.availableSpace === null ? (
            t('blobstores.list.unlimited')
          ) : (
            <FileSize bytes={row.availableSpace} />
          )}
        </div>
      ),
    },
    {
      id: 'open',
      header: '',
      label: t('blobstores.list.open'),
      width: '40px',
      cell: () => <ChevronRight size={16} aria-hidden="true" color="var(--gray-9)" />,
    },
  ];

  return (
    <Page
      title={t('blobstores.list.title')}
      description={t('blobstores.list.description')}
      breadcrumbs={[
        { label: t('blobstores.crumbs.settings'), to: '/settings' },
        { label: t('blobstores.crumbs.list') },
      ]}
      actions={
        canCreate && (
          <Button asChild>
            <Link to={CREATE_ADDRESS}>
              <Plus size={16} aria-hidden="true" />
              {t('blobstores.list.create')}
            </Link>
          </Button>
        )
      }
    >
      <TextField.Root
        className={styles.filter}
        type="search"
        value={filter}
        placeholder={t('blobstores.list.filter')}
        aria-label={t('blobstores.list.filter')}
        onChange={(event) => setFilter(event.target.value)}
      >
        <TextField.Slot>
          <Search size={14} aria-hidden="true" />
        </TextField.Slot>
      </TextField.Root>
      <div className={styles.listLayout}>
        {query.isPending ? (
          <Text size="2" color="gray">
            {t('blobstores.list.loading')}
          </Text>
        ) : query.error ? (
          <Text size="2" color="red">
            {t('blobstores.list.loadFailed')}
          </Text>
        ) : (
          <AdminTable
            label={t('blobstores.list.label')}
            columns={columns}
            rows={rows}
            getRowId={(row) => row.name}
            sort={sort}
            onSort={(column) =>
              setSort((current) => ({
                column: column as SortKey,
                direction: current.column === column && current.direction === 'asc' ? 'desc' : 'asc',
              }))
            }
            onOpen={open}
            openLabel={(row) => t('blobstores.list.openStore', { name: row.name })}
            empty={t('blobstores.list.empty')}
          />
        )}
        <HelpCard title={t('blobstores.list.about')}>{t('blobstores.list.aboutText')}</HelpCard>
      </div>
    </Page>
  );
}
