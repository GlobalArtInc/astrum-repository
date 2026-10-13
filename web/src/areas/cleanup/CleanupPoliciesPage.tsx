import { Button, Text } from '@radix-ui/themes';
import { CircleHelp, ExternalLink, Plus } from 'lucide-react';
import { useMemo, useState } from 'react';
import { Link, useNavigate, useSearchParams } from 'react-router';
import { useApi } from '../../api';
import { useCan } from '../../auth/state';
import { useT } from '../../i18n';
import { ErrorState, Page } from '../../ui';
import { type ListColumn, ListTable, SearchBox, sortList, useCollator } from '../security/shared/AdminList';
import list from '../security/shared/AdminList.module.css';
import styles from './Cleanup.module.css';
import {
  CREATE,
  DOCS,
  formatLabel,
  LIST_PAGE,
  matchesFilter,
  POLICIES,
  type Policy,
  policyHref,
} from './model';

export default function CleanupPoliciesPage() {
  const t = useT();
  const navigate = useNavigate();
  const collator = useCollator();
  const canCreate = useCan(CREATE);
  const [params, setParams] = useSearchParams();
  const filter = params.get('q') ?? '';
  const policies = useApi<Policy[]>(POLICIES);
  const all = t('cleanup.formats.all');
  const [sort, setSort] = useState<{ column: string; direction: 'asc' | 'desc' }>({
    column: 'name',
    direction: 'asc',
  });
  const columns: ListColumn<Policy>[] = [
    {
      id: 'name',
      header: t('cleanup.list.name'),
      width: '36%',
      cell: (policy) => policy.name,
      sortValue: (policy) => policy.name,
    },
    {
      id: 'format',
      header: t('cleanup.list.format'),
      width: '29%',
      cell: (policy) => formatLabel(policy.format, all),
      sortValue: (policy) => formatLabel(policy.format, all),
    },
    {
      id: 'description',
      header: t('cleanup.list.description'),
      cell: (policy) => policy.notes || '—',
      sortValue: (policy) => policy.notes ?? '',
    },
  ];
  const column = columns.find((candidate) => candidate.id === sort.column);
  const rows = useMemo(
    () =>
      sortList(
        (policies.data ?? []).filter((policy) =>
          matchesFilter(policy, filter, formatLabel(policy.format, all)),
        ),
        column?.sortValue,
        sort.direction,
        collator,
      ),
    [policies.data, filter, all, column, sort.direction, collator],
  );
  const search = (value: string) =>
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
    );
  return (
    <Page
      className={styles.page}
      title={t('cleanup.list.title')}
      description={t('cleanup.list.intro')}
      breadcrumbs={[
        { label: t('cleanup.crumbs.settings'), to: '/settings' },
        { label: t('cleanup.list.title') },
      ]}
      actions={
        canCreate && (
          <Button asChild size="3" className={`${list.bigButton} ${styles.createButton}`}>
            <Link to={`${LIST_PAGE}/create`}>
              <Plus size={16} aria-hidden="true" />
              {t('cleanup.list.create')}
            </Link>
          </Button>
        )
      }
    >
      <div className={styles.stack}>
        <section className={styles.help} aria-labelledby="cleanup-help-title">
          <Text as="div" size="2" weight="medium" id="cleanup-help-title" className={styles.helpTitle}>
            <CircleHelp size={14} aria-hidden="true" />
            {t('cleanup.list.whatTitle')}
          </Text>
          <Text as="p" size="2" className={styles.helpText}>
            {t('cleanup.list.whatText')}
          </Text>
          <a href={DOCS} target="_blank" rel="noreferrer" className={styles.external}>
            {t('cleanup.list.learnMore')}
            <ExternalLink size={12} aria-hidden="true" />
          </a>
        </section>
        <div className={list.toolbarEnd}>
          <SearchBox value={filter} onChange={search} placeholder={t('cleanup.list.filter')} />
        </div>
        {policies.error ? (
          <ErrorState error={policies.error} onRetry={() => void policies.refetch()} />
        ) : (
          <ListTable
            plain
            chevron
            label={t('cleanup.list.title')}
            columns={columns}
            rows={rows}
            loading={policies.isPending}
            getRowId={(policy) => policy.name}
            rowLabel={(policy) => t('cleanup.list.open', { name: policy.name })}
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
                {filter ? t('cleanup.list.noMatches', { filter }) : t('cleanup.list.empty')}
              </div>
            }
          />
        )}
      </div>
    </Page>
  );
}
