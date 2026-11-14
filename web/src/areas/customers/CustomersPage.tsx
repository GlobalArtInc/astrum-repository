import { Badge, Button } from '@radix-ui/themes';
import { Plus, Users } from 'lucide-react';
import { useState } from 'react';
import { useApi } from '../../api';
import { useCan } from '../../auth/state';
import { formatNumber, useI18n, useT } from '../../i18n';
import { type Column, DataTable, EmptyState, FileSize, Page, RelativeTime } from '../../ui';
import { CUSTOMERS, type Customer, customerPageHref } from './api';
import { CreateCustomerDialog } from './CreateCustomerDialog';
import styles from './Customers.module.css';
import { Expiry } from './Expiry';
import { CREATE } from './model';

export default function CustomersPage() {
  const t = useT();
  const { language } = useI18n();
  const canCreate = useCan(CREATE);
  const customers = useApi<Customer[]>(CUSTOMERS);
  const [creating, setCreating] = useState(false);
  const createButton = (
    <Button size="3" onClick={() => setCreating(true)}>
      <Plus size={16} aria-hidden="true" />
      {t('customers.list.create')}
    </Button>
  );
  const columns: Column<Customer>[] = [
    {
      id: 'name',
      header: t('customers.list.name'),
      primary: true,
      sortValue: (row) => row.name,
      cell: (row) => (
        <span className={styles.name}>
          {row.name}
          {row.disabled && (
            <Badge color="gray" variant="soft">
              {t('customers.list.disabled')}
            </Badge>
          )}
        </span>
      ),
    },
    {
      id: 'id',
      header: t('customers.list.id'),
      sortValue: (row) => row.id,
      cell: (row) => <code className={styles.code}>{row.id}</code>,
    },
    {
      id: 'reference',
      header: t('customers.list.reference'),
      sortValue: (row) => row.reference,
      cell: (row) => row.reference,
    },
    {
      id: 'tokens',
      header: t('customers.list.tokens'),
      align: 'right',
      sortValue: (row) => row.tokens,
      cell: (row) => row.tokens,
    },
    {
      id: 'expiry',
      header: t('customers.list.nearestExpiry'),
      sortValue: (row) => row.nearestExpiry ?? '',
      cell: (row) => <Expiry at={row.nearestExpiry} />,
    },
    {
      id: 'downloads',
      header: t('customers.list.downloads'),
      align: 'right',
      sortValue: (row) => row.downloads30d,
      cell: (row) => formatNumber(language, row.downloads30d),
    },
    {
      id: 'bytes',
      header: t('customers.list.bytes'),
      align: 'right',
      sortValue: (row) => row.bytes30d,
      cell: (row) => <FileSize bytes={row.bytes30d} />,
    },
    {
      id: 'lastAccess',
      header: t('customers.list.lastAccess'),
      sortValue: (row) => row.lastAccessAt ?? '',
      cell: (row) =>
        row.lastAccessAt === null ? t('customers.list.never') : <RelativeTime value={row.lastAccessAt} />,
    },
  ];
  return (
    <Page
      gutter="small"
      title={t('customers.list.title')}
      description={t('customers.list.intro')}
      breadcrumbs={[
        { label: t('customers.crumbs.settings'), to: '/settings' },
        { label: t('customers.list.title') },
      ]}
      actions={canCreate && createButton}
    >
      <DataTable
        label={t('customers.list.tableLabel')}
        columns={columns}
        rows={customers.data}
        loading={customers.isPending}
        error={customers.error}
        onRetry={() => void customers.refetch()}
        getRowId={(row) => row.id}
        rowHref={(row) => customerPageHref(row.id)}
        initialSort={{ column: 'name', direction: 'asc' }}
        empty={
          <EmptyState
            icon={Users}
            title={t('customers.list.emptyTitle')}
            description={t('customers.list.emptyText')}
            action={canCreate && createButton}
          />
        }
      />
      <CreateCustomerDialog open={creating} onOpenChange={setCreating} />
    </Page>
  );
}
