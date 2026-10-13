import { Button } from '@radix-ui/themes';
import { Eye, Lock, Pencil, Plus, Trash2 } from 'lucide-react';
import { useMemo, useState } from 'react';
import { Link, useNavigate } from 'react-router';
import { api, path, useApi, useInvalidateApi } from '../../../api';
import { useCan } from '../../../auth/state';
import { useT } from '../../../i18n';
import { ConfirmDialog, ErrorState, Page, useToast } from '../../../ui';
import { ROLES, type Role, USERS } from '../api';
import {
  DOCS,
  FilterPanel,
  type FilterState,
  HelpCard,
  type ListColumn,
  ListLayout,
  ListPager,
  ListTable,
  SearchBox,
  sortList,
  useCollator,
  usePaging,
} from '../shared/AdminList';
import list from '../shared/AdminList.module.css';
import { roleHref, roleProfileHref } from '../shared/links';
import styles from './Roles.module.css';

const PAGE_SIZE = 25;
const CUSTOM = 'custom';
const SYSTEM = 'system';

export function filterRoles(roles: readonly Role[], filters: FilterState, query: string): Role[] {
  const words = query.trim().toLowerCase().split(/\s+/).filter(Boolean);
  const management = filters.management ?? [];
  return roles.filter((role) => {
    if (management.length > 0 && !management.includes(role.readOnly ? SYSTEM : CUSTOM)) {
      return false;
    }
    const text = `${role.id} ${role.name} ${role.description ?? ''}`.toLowerCase();
    return words.every((word) => text.includes(word));
  });
}

export default function RolesPage() {
  const t = useT();
  const toast = useToast();
  const navigate = useNavigate();
  const invalidate = useInvalidateApi();
  const collator = useCollator();
  const canCreate = useCan('astrum:roles:create');
  const canUpdate = useCan('astrum:roles:update');
  const canDelete = useCan('astrum:roles:delete');
  const roles = useApi<Role[]>(ROLES);
  const [query, setQuery] = useState('');
  const [filters, setFilters] = useState<FilterState>({});
  const [deleting, setDeleting] = useState<Role | null>(null);
  const [sort, setSort] = useState<{ column: string; direction: 'asc' | 'desc' }>({
    column: 'name',
    direction: 'asc',
  });

  const all = roles.data ?? [];
  const sections = [
    {
      id: 'management',
      title: t('security.roles.management'),
      options: [
        {
          value: CUSTOM,
          label: t('security.roles.custom'),
          count: all.filter((role) => !role.readOnly).length,
        },
        {
          value: SYSTEM,
          label: t('security.roles.system'),
          count: all.filter((role) => role.readOnly).length,
        },
      ].filter((option) => option.count > 0),
    },
  ];

  const columns: ListColumn<Role>[] = [
    {
      id: 'name',
      header: t('security.roles.name'),
      sortValue: (row) => row.name || row.id,
      width: '26%',
      cell: (row) => (
        <span className={styles.name}>
          <span>{row.name || row.id}</span>
          {row.readOnly && (
            <Lock size={12} aria-label={t('security.roles.readOnly')} className={styles.lock} />
          )}
        </span>
      ),
    },
    {
      id: 'id',
      header: t('security.roles.id'),
      sortValue: (row) => row.id,
      width: '20%',
      cell: (row) => row.id,
    },
    {
      id: 'description',
      header: t('security.roles.descriptionColumn'),
      sortValue: (row) => row.description ?? '',
      cell: (row) => row.description,
    },
  ];
  const column = columns.find((candidate) => candidate.id === sort.column);
  const rows = useMemo(
    () => sortList(filterRoles(all, filters, query), column?.sortValue, sort.direction, collator),
    [all, filters, query, column, sort.direction, collator],
  );
  const paging = usePaging(rows, PAGE_SIZE);

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
      gutter="small"
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
      <ListLayout
        filters={
          <FilterPanel
            sections={sections}
            selected={filters}
            onChange={(section, values) => {
              setFilters({ ...filters, [section]: values });
              paging.setPage(0);
            }}
          />
        }
      >
        <SearchBox
          value={query}
          onChange={(value) => {
            setQuery(value);
            paging.setPage(0);
          }}
          placeholder={t('security.roles.search')}
        />
        {roles.error ? (
          <ErrorState error={roles.error} onRetry={() => void roles.refetch()} />
        ) : (
          <>
            <ListTable
              label={t('security.roles.listLabel')}
              columns={columns}
              rows={paging.visible}
              loading={roles.isPending}
              getRowId={(row) => `${row.source}/${row.id}`}
              rowLabel={(row) => t('security.roles.open', { id: row.id })}
              onOpen={(row) => navigate(roleProfileHref(row.id))}
              buttons={(row) => [
                {
                  label: t('security.roles.open', { id: row.id }),
                  icon: Eye,
                  onClick: () => navigate(roleProfileHref(row.id)),
                },
                ...(!row.readOnly && canUpdate
                  ? [
                      {
                        label: t('security.roles.edit', { id: row.id }),
                        icon: Pencil,
                        onClick: () => navigate(roleHref(row.id)),
                      },
                    ]
                  : []),
                ...(!row.readOnly && canDelete
                  ? [
                      {
                        label: t('security.roles.remove', { id: row.id }),
                        icon: Trash2,
                        danger: true,
                        onClick: () => setDeleting(row),
                      },
                    ]
                  : []),
              ]}
              sort={sort}
              onSort={(id) =>
                setSort(
                  sort.column === id
                    ? { column: id, direction: sort.direction === 'asc' ? 'desc' : 'asc' }
                    : { column: id, direction: 'asc' },
                )
              }
              empty={
                <div className={list.count}>
                  {query || Object.values(filters).some((values) => values.length > 0)
                    ? t('security.roles.noMatches')
                    : t('security.roles.empty')}
                </div>
              }
            />
            <ListPager
              range={t('security.roles.range', { from: paging.from, to: paging.to, total: paging.total })}
              page={paging.page}
              pages={paging.pages}
              onPage={paging.setPage}
            />
          </>
        )}
        <HelpCard title={t('security.roles.helpTitle')} link={`${DOCS}/roles`}>
          {t('security.roles.helpText')}
        </HelpCard>
      </ListLayout>
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
