import { Button, Select, Text } from '@radix-ui/themes';
import { Plus, Trash2, UsersRound } from 'lucide-react';
import { useState } from 'react';
import { Link, useSearchParams } from 'react-router';
import { api, useApi, useInvalidateApi, withQuery } from '../../../api';
import { useCan, useSession } from '../../../auth/state';
import { useT } from '../../../i18n';
import { type Column, ConfirmDialog, DataTable, EmptyState, Page, StatusBadge, useToast } from '../../../ui';
import { DEFAULT_SOURCE, USER_SOURCES, USERS, type User, type UserSource } from '../api';
import styles from '../shared/Form.module.css';
import { ListSearch } from '../shared/ListSearch';
import { userHref } from '../shared/links';
import { useDebounced } from '../shared/useDebounced';
import { deleteQuery, FALLBACK_SOURCES, fullName, STATUS_TONES, sourceLabel, statusLabel } from './model';

export default function UsersPage() {
  const t = useT();
  const toast = useToast();
  const invalidate = useInvalidateApi();
  const { user: me } = useSession();
  const canCreate = useCan('astrum:users:create');
  const canDelete = useCan('astrum:users:delete');
  const [params, setParams] = useSearchParams();
  const source = params.get('source') ?? DEFAULT_SOURCE;
  const query = params.get('q') ?? '';
  const search = useDebounced(query.trim(), 250);
  const [deleting, setDeleting] = useState<User | null>(null);
  const sources = useApi<UserSource[]>(USER_SOURCES);
  const users = useApi<User[]>(withQuery(USERS, { source, userId: search }), {
    placeholderData: (previous) => previous,
  });
  const known = sources.data ?? FALLBACK_SOURCES;

  const setParam = (name: string, value: string, empty: string) =>
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

  const columns: Column<User>[] = [
    {
      id: 'userId',
      header: t('security.users.userId'),
      cell: (row) => row.userId,
      sortValue: (row) => row.userId,
      primary: true,
    },
    {
      id: 'name',
      header: t('security.users.name'),
      cell: (row) => fullName(row),
      sortValue: (row) => fullName(row),
    },
    {
      id: 'email',
      header: t('security.users.email'),
      cell: (row) => row.emailAddress ?? '',
      sortValue: (row) => row.emailAddress ?? '',
      hideOnPhone: true,
    },
    {
      id: 'status',
      header: t('security.users.status'),
      cell: (row) => (
        <StatusBadge tone={STATUS_TONES[row.status] ?? 'neutral'}>{statusLabel(t, row.status)}</StatusBadge>
      ),
      sortValue: (row) => statusLabel(t, row.status),
    },
    {
      id: 'roles',
      header: t('security.users.roles'),
      cell: (row) => (
        <Text size="2" color="gray">
          {row.roles.join(', ')}
        </Text>
      ),
      hideOnPhone: true,
    },
  ];

  const remove = async () => {
    if (!deleting) {
      return;
    }
    await api.delete(deleteQuery(deleting, known));
    toast.success(t('security.users.deleted', { id: deleting.userId }));
    await invalidate(USERS);
  };

  const isMe = (row: User) => me !== null && me.userId === row.userId && me.source === row.source;

  return (
    <Page
      title={t('security.users.title')}
      description={t('security.users.intro')}
      breadcrumbs={[
        { label: t('security.crumbs.settings'), to: '/settings' },
        { label: t('security.users.title') },
      ]}
      actions={
        canCreate && (
          <Button asChild>
            <Link to="/admin/security/users/create">
              <Plus size={16} aria-hidden="true" />
              {t('security.users.create')}
            </Link>
          </Button>
        )
      }
    >
      <div className={styles.toolbar}>
        <ListSearch
          value={query}
          onChange={(value) => setParam('q', value, '')}
          label={t('security.users.search')}
        />
        <Select.Root value={source} onValueChange={(value) => setParam('source', value, DEFAULT_SOURCE)}>
          <Select.Trigger aria-label={t('security.users.sourceFilter')} />
          <Select.Content position="popper">
            {known.map((item) => (
              <Select.Item key={item.id} value={item.id}>
                {sourceLabel(t, item.id)}
              </Select.Item>
            ))}
          </Select.Content>
        </Select.Root>
      </div>
      <DataTable
        label={t('security.users.title')}
        columns={columns}
        rows={users.data}
        getRowId={(row) => `${row.source}/${row.userId}`}
        loading={users.isPending}
        error={users.error}
        onRetry={() => void users.refetch()}
        rowHref={(row) => userHref(row.userId, row.source)}
        initialSort={{ column: 'userId', direction: 'asc' }}
        pageSize={50}
        empty={
          <EmptyState
            icon={UsersRound}
            title={search ? t('security.users.noMatches') : t('security.users.empty')}
            description={source === DEFAULT_SOURCE ? undefined : t('security.users.externalEmpty')}
          />
        }
        rowActions={
          canDelete
            ? (row) =>
                isMe(row)
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
        title={t('security.users.deleteTitle', { id: deleting?.userId ?? '' })}
        description={t('security.users.deleteText')}
        confirmLabel={t('security.form.delete')}
        danger
      />
    </Page>
  );
}
