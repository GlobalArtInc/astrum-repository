import { Button } from '@radix-ui/themes';
import { useMemo, useState } from 'react';
import { Link, useNavigate } from 'react-router';
import { useApi } from '../../../api';
import { useCan } from '../../../auth/state';
import { useT } from '../../../i18n';
import { ErrorState, Page } from '../../../ui';
import { POLICIES } from '../api';
import { type ListColumn, ListTable, SearchBox, sortList, useCollator } from '../shared/AdminList';
import list from '../shared/AdminList.module.css';
import { MODE_OPTIONS, type Mode, POLICIES_PAGE, type Policy, policyHref } from './model';
import { POLICY_CREATE, POLICY_READ } from './permissions';

export default function PoliciesPage() {
  const t = useT();
  const navigate = useNavigate();
  const collator = useCollator();
  const canCreate = useCan(POLICY_CREATE);
  const canRead = useCan(POLICY_READ);
  const [filter, setFilter] = useState('');
  const [sort, setSort] = useState<{ column: string; direction: 'asc' | 'desc' }>({
    column: 'name',
    direction: 'asc',
  });
  const policies = useApi<Policy[]>(canRead ? POLICIES : null);
  const modeText = (mode: Mode) => t(MODE_OPTIONS.find((option) => option.value === mode)?.label ?? mode);
  const columns: ListColumn<Policy>[] = [
    {
      id: 'name',
      header: t('security.policy.columns.name'),
      width: '28%',
      cell: (policy) => policy.name,
      sortValue: (policy) => policy.name,
    },
    {
      id: 'vulnerabilities',
      header: t('security.policy.columns.vulnerabilities'),
      cell: (policy) => modeText(policy.vulnerabilities.mode),
      sortValue: (policy) => policy.vulnerabilities.mode,
    },
    {
      id: 'licenses',
      header: t('security.policy.columns.licenses'),
      cell: (policy) => modeText(policy.licenses.mode),
      sortValue: (policy) => policy.licenses.mode,
    },
    {
      id: 'signatures',
      header: t('security.policy.columns.signatures'),
      cell: (policy) => modeText(policy.signatures.mode),
      sortValue: (policy) => policy.signatures.mode,
    },
    {
      id: 'revision',
      header: t('security.policy.columns.revision'),
      cell: (policy) => policy.revision,
      sortValue: (policy) => policy.revision,
    },
  ];
  const column = columns.find((candidate) => candidate.id === sort.column);
  const rows = useMemo(
    () =>
      sortList(
        (policies.data ?? []).filter((policy) =>
          policy.name.toLowerCase().includes(filter.trim().toLowerCase()),
        ),
        column?.sortValue,
        sort.direction,
        collator,
      ),
    [policies.data, filter, column, sort.direction, collator],
  );
  return (
    <Page
      title={t('security.policy.title')}
      description={t('security.policy.intro')}
      breadcrumbs={[
        { label: t('security.crumbs.settings'), to: '/settings' },
        { label: t('security.policy.title') },
      ]}
      actions={
        canCreate && (
          <Button asChild size="3" className={list.bigButton}>
            <Link to={`${POLICIES_PAGE}/create`}>{t('security.policy.create')}</Link>
          </Button>
        )
      }
    >
      <div className={list.toolbarEnd}>
        <SearchBox value={filter} onChange={setFilter} placeholder={t('security.policy.filter')} />
      </div>
      {policies.error ? (
        <ErrorState error={policies.error} onRetry={() => void policies.refetch()} />
      ) : (
        <ListTable
          plain
          chevron
          label={t('security.policy.title')}
          columns={columns}
          rows={rows}
          loading={policies.isPending}
          getRowId={(policy) => policy.name}
          rowLabel={(policy) => t('security.policy.open', { name: policy.name })}
          onOpen={(policy) => navigate(policyHref(policy.name))}
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
              {filter ? t('security.policy.noMatches', { filter }) : t('security.policy.empty')}
            </div>
          }
        />
      )}
    </Page>
  );
}
