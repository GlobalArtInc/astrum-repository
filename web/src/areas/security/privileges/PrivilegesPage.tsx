import { Button } from '@radix-ui/themes';
import { Eye, Lock, Plus } from 'lucide-react';
import { Fragment, useMemo, useState } from 'react';
import { Link, useNavigate } from 'react-router';
import { useApi } from '../../../api';
import { useCan } from '../../../auth/state';
import { useT } from '../../../i18n';
import { ErrorState, Page } from '../../../ui';
import { PRIVILEGES, type Privilege } from '../api';
import {
  countBy,
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
import { privilegeProfileHref } from '../shared/links';
import { typeLabel } from './labels';
import { permissionOf } from './model';
import styles from './Privileges.module.css';

const PAGE_SIZE = 25;
const LOCKED = 'locked';
const UNLOCKED = 'unlocked';

export function filterPrivileges(
  privileges: readonly Privilege[],
  filters: FilterState,
  query: string,
): Privilege[] {
  const words = query.trim().toLowerCase().split(/\s+/).filter(Boolean);
  const status = filters.status ?? [];
  const types = filters.type ?? [];
  return privileges.filter((privilege) => {
    if (status.length > 0 && !status.includes(privilege.readOnly ? LOCKED : UNLOCKED)) {
      return false;
    }
    if (types.length > 0 && !types.includes(privilege.type)) {
      return false;
    }
    const text = `${privilege.name} ${privilege.description} ${permissionOf(privilege) ?? ''}`.toLowerCase();
    return words.every((word) => text.includes(word));
  });
}

export function Permission({ value }: { value: string }) {
  const parts = value.split(/(?<=[:,])/);
  return (
    <code className={styles.permission}>
      {parts.map((part, index) => (
        // biome-ignore lint/suspicious/noArrayIndexKey: the parts of one string keep their order
        <Fragment key={index}>
          {part}
          {index < parts.length - 1 && <wbr />}
        </Fragment>
      ))}
    </code>
  );
}

export default function PrivilegesPage() {
  const t = useT();
  const navigate = useNavigate();
  const collator = useCollator();
  const canCreate = useCan('astrum:privileges:create');
  const privileges = useApi<Privilege[]>(PRIVILEGES);
  const [query, setQuery] = useState('');
  const [filters, setFilters] = useState<FilterState>({});
  const [sort, setSort] = useState<{ column: string; direction: 'asc' | 'desc' }>({
    column: 'name',
    direction: 'asc',
  });

  const all = privileges.data ?? [];
  const sections = useMemo(() => {
    const types = countBy(all, (privilege) => privilege.type);
    return [
      {
        id: 'status',
        title: t('security.privileges.status'),
        options: [
          {
            value: LOCKED,
            label: t('security.privileges.locked'),
            count: all.filter((privilege) => privilege.readOnly).length,
          },
          {
            value: UNLOCKED,
            label: t('security.privileges.unlocked'),
            count: all.filter((privilege) => !privilege.readOnly).length,
          },
        ],
      },
      {
        id: 'type',
        title: t('security.privileges.type'),
        options: [...types.keys()].sort().map((type) => ({
          value: type,
          label: typeLabel(t, type),
          count: types.get(type) ?? 0,
        })),
      },
    ];
  }, [all, t]);

  const columns: ListColumn<Privilege>[] = [
    {
      id: 'name',
      header: t('security.privileges.name'),
      sortValue: (row) => row.name,
      width: '20%',
      cell: (row) => (
        <span className={styles.name}>
          {row.readOnly && (
            <Lock size={14} aria-label={t('security.privileges.lockedIcon')} className={styles.lock} />
          )}
          <span>{row.name}</span>
        </span>
      ),
    },
    {
      id: 'description',
      header: t('security.privileges.descriptionColumn'),
      sortValue: (row) => row.description,
      cell: (row) => row.description,
    },
    {
      id: 'type',
      header: t('security.privileges.type'),
      sortValue: (row) => typeLabel(t, row.type),
      cell: (row) => typeLabel(t, row.type),
    },
    {
      id: 'permission',
      header: t('security.privileges.permission'),
      sortValue: (row) => permissionOf(row) ?? '',
      cell: (row) => {
        const permission = permissionOf(row);
        return permission ? <Permission value={permission} /> : null;
      },
    },
  ];

  const column = columns.find((candidate) => candidate.id === sort.column);
  const rows = useMemo(
    () => sortList(filterPrivileges(all, filters, query), column?.sortValue, sort.direction, collator),
    [all, filters, query, column, sort.direction, collator],
  );
  const paging = usePaging(rows, PAGE_SIZE);

  return (
    <Page
      gutter="small"
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
          placeholder={t('security.privileges.search')}
        />
        {privileges.error ? (
          <ErrorState error={privileges.error} onRetry={() => void privileges.refetch()} />
        ) : (
          <>
            <ListTable
              label={t('security.privileges.listLabel')}
              columns={columns}
              rows={paging.visible}
              loading={privileges.isPending}
              getRowId={(row) => row.name}
              rowLabel={(row) => t('security.privileges.open', { name: row.name })}
              onOpen={(row) => navigate(privilegeProfileHref(row.name))}
              buttons={(row) => [
                {
                  label: t('security.privileges.openProfile', { name: row.name }),
                  icon: Eye,
                  onClick: () => navigate(privilegeProfileHref(row.name)),
                },
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
                    ? t('security.privileges.noMatches')
                    : t('security.privileges.empty')}
                </div>
              }
            />
            <ListPager
              range={t('security.privileges.range', {
                from: paging.from,
                to: paging.to,
                total: paging.total,
              })}
              page={paging.page}
              pages={paging.pages}
              onPage={paging.setPage}
            />
          </>
        )}
        <HelpCard title={t('security.privileges.helpTitle')} link={`${DOCS}/privileges`}>
          {t('security.privileges.helpText')}
        </HelpCard>
      </ListLayout>
    </Page>
  );
}
