import { Button, Text } from '@radix-ui/themes';
import { Lock, Plus, ShieldCheck, Trash2 } from 'lucide-react';
import { useMemo, useState } from 'react';
import { Link, useSearchParams } from 'react-router';
import { api, path, useApi, useInvalidateApi } from '../../../api';
import { useCan } from '../../../auth/state';
import { useT } from '../../../i18n';
import { type Column, ConfirmDialog, DataTable, EmptyState, Page, useToast } from '../../../ui';
import { ROLES, type Role, USERS } from '../api';
import styles from '../shared/Form.module.css';
import { ListSearch } from '../shared/ListSearch';
import { roleHref } from '../shared/links';

export function filterRoles(roles: readonly Role[], query: string): Role[] {
  const words = query.trim().toLowerCase().split(/\s+/).filter(Boolean);
  return roles.filter((role) => {
    const text = `${role.id} ${role.name} ${role.description}`.toLowerCase();
    return words.every((word) => text.includes(word));
  });
}

export default function RolesPage() {
  const t = useT();
  const toast = useToast();
  const invalidate = useInvalidateApi();
  const canCreate = useCan('astrum:roles:create');
  const canDelete = useCan('astrum:roles:delete');
  const [params, setParams] = useSearchParams();
  const query = params.get('q') ?? '';
  const [deleting, setDeleting] = useState<Role | null>(null);
  const roles = useApi<Role[]>(ROLES);
  const rows = useMemo(() => (roles.data ? filterRoles(roles.data, query) : undefined), [roles.data, query]);

  const columns: Column<Role>[] = [
    {
      id: 'id',
      header: t('security.roles.id'),
      cell: (row) => row.id,
      sortValue: (row) => row.id,
      primary: true,
    },
    { id: 'name', header: t('security.roles.name'), cell: (row) => row.name, sortValue: (row) => row.name },
    {
      id: 'description',
      header: t('security.roles.descriptionColumn'),
      cell: (row) => (
        <Text size="2" color="gray">
          {row.description}
        </Text>
      ),
      hideOnPhone: true,
    },
    {
      id: 'privileges',
      header: t('security.roles.privileges'),
      cell: (row) => row.privileges.length,
      sortValue: (row) => row.privileges.length,
      align: 'right',
    },
    {
      id: 'roles',
      header: t('security.roles.roles'),
      cell: (row) => row.roles.length,
      sortValue: (row) => row.roles.length,
      align: 'right',
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
    await api.delete(`${ROLES}${path`/${deleting.id}`}`);
    toast.success(t('security.roles.deleted', { id: deleting.id }));
    await Promise.all([invalidate(ROLES), invalidate(USERS)]);
  };

  return (
    <Page
      title={t('security.roles.title')}
      description={t('security.roles.intro')}
      breadcrumbs={[
        { label: t('security.crumbs.settings'), to: '/settings' },
        { label: t('security.roles.title') },
      ]}
      actions={
        canCreate && (
          <Button asChild>
            <Link to="/admin/security/roles/create">
              <Plus size={16} aria-hidden="true" />
              {t('security.roles.create')}
            </Link>
          </Button>
        )
      }
    >
      <div className={styles.toolbar}>
        <ListSearch
          value={query}
          label={t('security.roles.search')}
          onChange={(value) =>
            setParams(
              (previous) => {
                const next = new URLSearchParams(previous);
                if (value) {
                  next.set('q', value);
                } else {
                  next.delete('q');
                }
                return next;
              },
              { replace: true },
            )
          }
        />
      </div>
      <DataTable
        label={t('security.roles.title')}
        columns={columns}
        rows={rows}
        getRowId={(row) => row.id}
        loading={roles.isPending}
        error={roles.error}
        onRetry={() => void roles.refetch()}
        rowHref={(row) => roleHref(row.id)}
        initialSort={{ column: 'id', direction: 'asc' }}
        pageSize={50}
        empty={
          <EmptyState
            icon={ShieldCheck}
            title={query ? t('security.roles.noMatches') : t('security.roles.empty')}
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
        title={t('security.roles.deleteTitle', { id: deleting?.id ?? '' })}
        description={t('security.roles.deleteText')}
        confirmLabel={t('security.form.delete')}
        danger
      />
    </Page>
  );
}
