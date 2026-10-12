import { Button } from '@radix-ui/themes';
import { HardDrive, Plus, Settings2, Trash2 } from 'lucide-react';
import { Link, useNavigate } from 'react-router';
import { api, useInvalidateApi } from '../../../api';
import { useCan } from '../../../auth/state';
import { formatNumber, type Language, type Translate, useI18n } from '../../../i18n';
import {
  type Column,
  DataTable,
  describeError,
  EmptyState,
  FileSize,
  Page,
  useConfirm,
  useToast,
} from '../../../ui';
import {
  CREATE_ADDRESS,
  deletePath,
  storeAddress,
  storeType,
  UI_BLOB_STORES,
  type UiBlobStore,
  useUiBlobStores,
} from '../api';
import { QuotaBadge, StateBadge, typeLabel, usedBy } from '../components/bits';

function columns(t: Translate, language: Language): Column<UiBlobStore>[] {
  return [
    {
      id: 'name',
      header: t('blobstores.list.name'),
      cell: (row) => row.name,
      sortValue: (row) => row.name,
      primary: true,
    },
    {
      id: 'type',
      header: t('blobstores.list.type'),
      cell: (row) => typeLabel(t, row.type),
      sortValue: (row) => row.type,
    },
    {
      id: 'state',
      header: t('blobstores.list.state'),
      cell: (row) => <StateBadge store={row} />,
      sortValue: (row) => (row.unavailable ? 1 : 0),
    },
    {
      id: 'blobs',
      header: t('blobstores.list.blobCount'),
      cell: (row) => formatNumber(language, row.blobCount),
      sortValue: (row) => row.blobCount,
      align: 'right',
    },
    {
      id: 'size',
      header: t('blobstores.list.size'),
      cell: (row) => <FileSize bytes={row.totalSize} />,
      sortValue: (row) => row.totalSize,
      align: 'right',
    },
    {
      id: 'available',
      header: t('blobstores.list.available'),
      cell: (row) =>
        row.availableSpace === null ? (
          t('blobstores.list.unlimited')
        ) : (
          <FileSize bytes={row.availableSpace} />
        ),
      sortValue: (row) => row.availableSpace,
      align: 'right',
      hideOnPhone: true,
    },
    {
      id: 'quota',
      header: t('blobstores.list.quota'),
      cell: (row) => <QuotaBadge store={row} />,
      sortValue: (row) => (row.quotaStatus ? (row.quotaStatus.violation ? 2 : 1) : 0),
    },
    {
      id: 'usedBy',
      header: t('blobstores.list.usedBy'),
      cell: (row) => usedBy(t, row),
      sortValue: (row) => row.repositoryCount,
    },
  ];
}

export function addressOf(store: UiBlobStore): string | undefined {
  const type = storeType(store.type);
  return type ? storeAddress(type, store.name) : undefined;
}

export default function BlobStoresPage() {
  const { t, language } = useI18n();
  const navigate = useNavigate();
  const confirm = useConfirm();
  const toast = useToast();
  const invalidate = useInvalidateApi();
  const canCreate = useCan('astrum:blobstores:create');
  const canDelete = useCan('astrum:blobstores:delete');
  const query = useUiBlobStores();

  const remove = async (store: UiBlobStore) => {
    const agreed = await confirm({
      title: t('blobstores.delete.title', { name: store.name }),
      description: t('blobstores.delete.text'),
      confirmLabel: t('blobstores.delete.confirm'),
      danger: true,
    });
    if (!agreed) {
      return;
    }
    try {
      await api.delete(deletePath(store.name));
      toast.success(t('blobstores.delete.deleted', { name: store.name }));
      await invalidate(UI_BLOB_STORES);
    } catch (error) {
      toast.error(describeError(t, error));
    }
  };

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
      <DataTable
        label={t('blobstores.list.title')}
        columns={columns(t, language)}
        rows={query.data}
        getRowId={(row) => row.name}
        loading={query.isPending}
        error={query.error}
        onRetry={() => void query.refetch()}
        rowHref={addressOf}
        initialSort={{ column: 'name', direction: 'asc' }}
        rowActions={(row) => {
          const address = addressOf(row);
          return [
            ...(address
              ? [
                  {
                    id: 'edit',
                    label: t('blobstores.list.edit'),
                    icon: Settings2,
                    onSelect: () => navigate(address),
                  },
                ]
              : []),
            ...(canDelete
              ? [
                  {
                    id: 'delete',
                    label: t('blobstores.list.delete'),
                    icon: Trash2,
                    danger: true,
                    disabled: !row.deletable,
                    onSelect: () => void remove(row),
                  },
                ]
              : []),
          ];
        }}
        empty={
          <EmptyState
            icon={HardDrive}
            title={t('blobstores.list.empty')}
            description={t('blobstores.list.emptyText')}
          />
        }
      />
    </Page>
  );
}
