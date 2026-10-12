import { Button, Select, Text } from '@radix-ui/themes';
import { KeyRound, Lock, Plus, Trash2 } from 'lucide-react';
import { useMemo, useState } from 'react';
import { Link, useSearchParams } from 'react-router';
import { api, path, useApi, useInvalidateApi } from '../../../api';
import { useCan } from '../../../auth/state';
import { useT } from '../../../i18n';
import { type Column, ConfirmDialog, DataTable, EmptyState, Page, StatusBadge, useToast } from '../../../ui';
import { PRIVILEGE_TYPE_IDS, PRIVILEGES, type Privilege, ROLES } from '../api';
import styles from '../shared/Form.module.css';
import { ListSearch } from '../shared/ListSearch';
import { privilegeHref } from '../shared/links';
import { typeLabel } from './labels';
import { permissionOf } from './model';

const ALL = 'all';

export function filterPrivileges(privileges: readonly Privilege[], type: string, query: string): Privilege[] {
  const words = query.trim().toLowerCase().split(/\s+/).filter(Boolean);
  return privileges.filter((privilege) => {
    if (type !== '' && type !== ALL && privilege.type !== type) {
      return false;
    }
    const text = `${privilege.name} ${privilege.description} ${permissionOf(privilege) ?? ''}`.toLowerCase();
    return words.every((word) => text.includes(word));
  });
}

export default function PrivilegesPage() {
  const t = useT();
  const toast = useToast();
  const invalidate = useInvalidateApi();
  const canCreate = useCan('astrum:privileges:create');
  const canDelete = useCan('astrum:privileges:delete');
  const [params, setParams] = useSearchParams();
  const type = params.get('type') ?? ALL;
  const query = params.get('q') ?? '';
  const [deleting, setDeleting] = useState<Privilege | null>(null);
  const privileges = useApi<Privilege[]>(PRIVILEGES);

  const setParam = (name: string, value: string, empty: string) => {
    setParams(
      (previous) => {
        const next = new URLSearchParams(previous);
        if (value === empty) {
          next.delete(name);
        } else {
          next.set(name, value);
        }
        return next;
      },
      { replace: true },
    );
  };

  const rows = useMemo(
    () => (privileges.data ? filterPrivileges(privileges.data, type, query) : undefined),
    [privileges.data, type, query],
  );

  const columns: Column<Privilege>[] = [
    {
      id: 'name',
      header: t('security.privileges.name'),
      cell: (row) => row.name,
      sortValue: (row) => row.name,
      primary: true,
    },
    {
      id: 'type',
      header: t('security.privileges.type'),
      cell: (row) => <StatusBadge tone="info">{typeLabel(t, row.type)}</StatusBadge>,
      sortValue: (row) => typeLabel(t, row.type),
    },
    {
      id: 'permission',
      header: t('security.privileges.permission'),
      cell: (row) => <span className={styles.mono}>{permissionOf(row) ?? '—'}</span>,
      hideOnPhone: true,
    },
    {
      id: 'description',
      header: t('security.privileges.descriptionColumn'),
      cell: (row) => (
        <Text size="2" color="gray">
          {row.description}
        </Text>
      ),
      sortValue: (row) => row.description,
      hideOnPhone: true,
    },
    {
      id: 'builtIn',
      header: t('security.privileges.builtIn'),
      cell: (row) =>
        row.readOnly ? (
          <Lock size={14} aria-label={t('security.privileges.builtInYes')} />
        ) : (
          <span className="visually-hidden">{t('security.privileges.builtInNo')}</span>
        ),
      sortValue: (row) => (row.readOnly ? 0 : 1),
      align: 'center',
      width: '96px',
    },
  ];

  const remove = async () => {
    if (!deleting) {
      return;
    }
    await api.delete(`${PRIVILEGES}${path`/${deleting.name}`}`);
    toast.success(t('security.privileges.deleted', { name: deleting.name }));
    await Promise.all([invalidate(PRIVILEGES), invalidate(ROLES)]);
  };

  return (
    <Page
      title={t('security.privileges.title')}
      description={t('security.privileges.intro')}
      breadcrumbs={[
        { label: t('security.crumbs.settings'), to: '/settings' },
        { label: t('security.privileges.title') },
      ]}
      actions={
        canCreate && (
          <Button asChild>
            <Link to="/admin/security/privileges/create">
              <Plus size={16} aria-hidden="true" />
              {t('security.privileges.create')}
            </Link>
          </Button>
        )
      }
    >
      <div className={styles.toolbar}>
        <ListSearch
          value={query}
          onChange={(value) => setParam('q', value, '')}
          label={t('security.privileges.search')}
        />
        <Select.Root value={type} onValueChange={(value) => setParam('type', value, ALL)}>
          <Select.Trigger aria-label={t('security.privileges.typeFilter')} />
          <Select.Content position="popper">
            <Select.Item value={ALL}>{t('security.privileges.allTypes')}</Select.Item>
            {PRIVILEGE_TYPE_IDS.map((id) => (
              <Select.Item key={id} value={id}>
                {typeLabel(t, id)}
              </Select.Item>
            ))}
          </Select.Content>
        </Select.Root>
      </div>
      <DataTable
        label={t('security.privileges.title')}
        columns={columns}
        rows={rows}
        getRowId={(row) => row.name}
        loading={privileges.isPending}
        error={privileges.error}
        onRetry={() => void privileges.refetch()}
        rowHref={(row) => privilegeHref(row.name)}
        initialSort={{ column: 'name', direction: 'asc' }}
        pageSize={50}
        empty={
          <EmptyState
            icon={KeyRound}
            title={
              query || type !== ALL ? t('security.privileges.noMatches') : t('security.privileges.empty')
            }
          />
        }
        rowActions={
          canDelete
            ? (row) =>
                row.readOnly
                  ? []
                  : [
                      {
                        id: 'delete',
                        label: t('security.form.delete'),
                        icon: Trash2,
                        danger: true,
                        onSelect: setDeleting,
                      },
                    ]
            : undefined
        }
      />
      <ConfirmDialog
        open={deleting !== null}
        onOpenChange={(open) => !open && setDeleting(null)}
        onConfirm={remove}
        title={t('security.privileges.deleteTitle', { name: deleting?.name ?? '' })}
        description={t('security.privileges.deleteText')}
        confirmLabel={t('security.form.delete')}
        danger
      />
    </Page>
  );
}
