import { Button } from '@radix-ui/themes';
import { Plus } from 'lucide-react';
import { useMemo, useState } from 'react';
import { Link, useNavigate } from 'react-router';
import { useApi } from '../../api';
import { useCan } from '../../auth/state';
import { useT } from '../../i18n';
import { ErrorState, Page } from '../../ui';
import {
  DOCS,
  HelpCard,
  type ListColumn,
  ListTable,
  SearchBox,
  sortList,
  useCollator,
} from '../security/shared/AdminList';
import list from '../security/shared/AdminList.module.css';
import { type ContentSelector, SELECTORS, selectorHref } from './model';
import styles from './Selectors.module.css';

export function filterSelectors(selectors: readonly ContentSelector[], query: string): ContentSelector[] {
  const words = query.trim().toLowerCase().split(/\s+/).filter(Boolean);
  return selectors.filter((selector) => {
    const text = `${selector.name} ${selector.description ?? ''}`.toLowerCase();
    return words.every((word) => text.includes(word));
  });
}

export default function SelectorsPage() {
  const t = useT();
  const navigate = useNavigate();
  const collator = useCollator();
  const canCreate = useCan('astrum:selectors:create');
  const selectors = useApi<ContentSelector[]>(SELECTORS);
  const [query, setQuery] = useState('');
  const [sort, setSort] = useState<{ column: string; direction: 'asc' | 'desc' }>({
    column: 'name',
    direction: 'asc',
  });
  const columns: ListColumn<ContentSelector>[] = [
    {
      id: 'name',
      header: t('selectors.list.name'),
      sortValue: (row) => row.name,
      cell: (row) => <span className={styles.name}>{row.name}</span>,
    },
    { id: 'type', header: t('selectors.list.type'), cell: () => 'CSEL' },
    {
      id: 'description',
      header: t('selectors.list.description'),
      sortValue: (row) => row.description ?? '',
      cell: (row) => <span className={styles.muted}>{row.description}</span>,
    },
    {
      id: 'expression',
      header: t('selectors.list.expression'),
      sortValue: (row) => row.expression,
      cell: (row) => row.expression,
    },
  ];
  const column = columns.find((candidate) => candidate.id === sort.column);
  const rows = useMemo(
    () => sortList(filterSelectors(selectors.data ?? [], query), column?.sortValue, sort.direction, collator),
    [selectors.data, query, column, sort.direction, collator],
  );
  return (
    <Page
      gutter="small"
      title={t('selectors.list.title')}
      description={t('selectors.list.intro')}
      breadcrumbs={[
        { label: t('selectors.crumbs.settings'), to: '/settings' },
        { label: t('selectors.list.title') },
      ]}
      actions={
        canCreate && (
          <Button asChild size="3" className={list.bigButton}>
            <Link to="/admin/repository/selectors/create">
              <Plus size={16} aria-hidden="true" />
              {t('selectors.list.create')}
            </Link>
          </Button>
        )
      }
    >
      <div className={styles.listPage}>
        <HelpCard
          flush
          title={t('selectors.list.helpTitle')}
          link={`${DOCS}/content-selectors`}
          linkLabel={t('selectors.list.more')}
        >
          {t('selectors.list.about')}
        </HelpCard>
        <div className={list.toolbarEnd}>
          <SearchBox value={query} onChange={setQuery} placeholder={t('selectors.list.search')} />
        </div>
        {selectors.error ? (
          <ErrorState error={selectors.error} onRetry={() => void selectors.refetch()} />
        ) : (
          <ListTable
            plain
            chevron
            label={t('selectors.list.listLabel')}
            columns={columns}
            rows={rows}
            loading={selectors.isPending}
            getRowId={(row) => row.name}
            rowLabel={(row) => t('selectors.list.open', { name: row.name })}
            onOpen={(row) => navigate(selectorHref(row.name))}
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
                {query ? t('selectors.list.noMatches') : t('selectors.list.empty')}
              </div>
            }
          />
        )}
      </div>
    </Page>
  );
}
