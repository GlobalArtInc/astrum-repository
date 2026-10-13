import { Badge, Button } from '@radix-ui/themes';
import { Plus } from 'lucide-react';
import { useMemo, useState } from 'react';
import { Link, useNavigate } from 'react-router';
import { useApi, withQuery } from '../../../api';
import { useCan } from '../../../auth/state';
import { useT } from '../../../i18n';
import { ErrorState, Page } from '../../../ui';
import { DEFAULT_SOURCE, USERS, type User } from '../api';
import {
  countBy,
  DOCS,
  FilterPanel,
  type FilterState,
  HelpCard,
  type ListColumn,
  ListLayout,
  ListTable,
  SearchBox,
  sortList,
  useCollator,
} from '../shared/AdminList';
import list from '../shared/AdminList.module.css';
import { userHref } from '../shared/links';
import { fullName, sourceLabel } from './model';

const ACTIVE = 'active';
const INACTIVE = 'inactive';

export function filterUsers(users: readonly User[], filters: FilterState, query: string): User[] {
  const words = query.trim().toLowerCase();
  const sources = filters.source ?? [];
  const statuses = filters.status ?? [];
  return users.filter((user) => {
    if (sources.length > 0 && !sources.includes(user.source)) {
      return false;
    }
    if (statuses.length > 0 && !statuses.includes(user.status === 'active' ? ACTIVE : INACTIVE)) {
      return false;
    }
    return !words || user.userId.toLowerCase().includes(words);
  });
}

export default function UsersPage() {
  const t = useT();
  const navigate = useNavigate();
  const collator = useCollator();
  const canCreate = useCan('astrum:users:create');
  const users = useApi<User[]>(withQuery(USERS, { source: DEFAULT_SOURCE }));
  const [query, setQuery] = useState('');
  const [filters, setFilters] = useState<FilterState>({});
  const [sort, setSort] = useState<{ column: string; direction: 'asc' | 'desc' }>({
    column: 'userId',
    direction: 'asc',
  });
  const all = users.data ?? [];
  const sources = countBy(all, (user) => user.source);
  const active = all.filter((user) => user.status === 'active').length;
  const sections = [
    {
      id: 'source',
      title: t('security.users.source'),
      options: [...sources.keys()].map((source) => ({
        value: source,
        label: source === DEFAULT_SOURCE ? t('security.users.sourceLocal') : sourceLabel(t, source),
        count: sources.get(source) ?? 0,
      })),
    },
    {
      id: 'status',
      title: t('security.users.status'),
      options: [
        { value: ACTIVE, label: t('security.users.statusActive'), count: active },
        { value: INACTIVE, label: t('security.users.statusInactive'), count: all.length - active },
      ],
    },
  ];
  const columns: ListColumn<User>[] = [
    {
      id: 'userId',
      header: t('security.users.userId'),
      sortValue: (row) => row.userId,
      cell: (row) => row.userId,
    },
    {
      id: 'firstName',
      header: t('security.users.firstName'),
      sortValue: (row) => row.firstName ?? '',
      cell: (row) => row.firstName,
    },
    {
      id: 'lastName',
      header: t('security.users.lastName'),
      sortValue: (row) => row.lastName ?? '',
      cell: (row) => row.lastName,
    },
    {
      id: 'email',
      header: t('security.users.email'),
      sortValue: (row) => row.emailAddress ?? '',
      cell: (row) => row.emailAddress,
    },
    {
      id: 'source',
      header: t('security.users.source'),
      sortValue: (row) => row.source,
      cell: (row) =>
        row.source === DEFAULT_SOURCE ? t('security.users.sourceLocal') : sourceLabel(t, row.source),
    },
    {
      id: 'status',
      header: t('security.users.status'),
      sortValue: (row) => row.status,
      cell: (row) =>
        row.status === 'active' ? (
          <Badge color="green" variant="soft">
            {t('security.users.statusActive')}
          </Badge>
        ) : (
          <Badge color="gray" variant="soft">
            {t('security.users.statusInactive')}
          </Badge>
        ),
    },
  ];
  const column = columns.find((candidate) => candidate.id === sort.column);
  const rows = useMemo(
    () => sortList(filterUsers(all, filters, query), column?.sortValue, sort.direction, collator),
    [all, filters, query, column, sort.direction, collator],
  );

  return (
    <Page
      gutter="small"
      title={t('security.users.title')}
      description={t('security.users.intro')}
      breadcrumbs={[
        { label: t('security.crumbs.settings'), to: '/settings' },
        { label: t('security.users.title') },
      ]}
      actions={
        canCreate && (
          <Button asChild size="3" className={list.bigButton}>
            <Link to="/admin/security/users/create">
              <Plus size={16} aria-hidden="true" />
              {t('security.users.create')}
            </Link>
          </Button>
        )
      }
    >
      <ListLayout
        tight
        filters={
          <FilterPanel
            sections={sections}
            selected={filters}
            onChange={(section, values) => setFilters({ ...filters, [section]: values })}
          />
        }
      >
        <SearchBox value={query} onChange={setQuery} placeholder={t('security.users.search')} />
        {users.error ? (
          <ErrorState error={users.error} onRetry={() => void users.refetch()} />
        ) : (
          <>
            <ListTable
              label={t('security.users.listLabel')}
              columns={columns}
              rows={rows}
              loading={users.isPending}
              getRowId={(row) => `${row.source}/${row.userId}`}
              rowLabel={(row) => t('security.users.edit', { name: fullName(row) || row.userId })}
              onOpen={(row) => navigate(userHref(row.userId, row.source))}
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
                    ? t('security.users.noMatches')
                    : t('security.users.empty')}
                </div>
              }
            />
            <p className={list.count}>{t('security.users.shown', { count: rows.length })}</p>
          </>
        )}
        <HelpCard title={t('security.users.helpTitle')} link={`${DOCS}/users`}>
          {t('security.users.helpText')}
        </HelpCard>
      </ListLayout>
    </Page>
  );
}
