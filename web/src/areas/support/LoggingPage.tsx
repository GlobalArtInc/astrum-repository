import { Button } from '@radix-ui/themes';
import { Pencil, Plus, RotateCcw } from 'lucide-react';
import { useMemo, useState } from 'react';
import { Link, useNavigate } from 'react-router';
import { api, useApi, useInvalidateApi } from '../../api';
import { useCan } from '../../auth/state';
import { useT } from '../../i18n';
import { ConfirmDialog, ErrorState, Page, useToast } from '../../ui';
import {
  FilterPanel,
  type FilterState,
  HelpCard,
  type ListColumn,
  ListLayout,
  ListTable,
  SearchBox,
  sortList,
  useCollator,
} from '../security/shared/AdminList';
import list from '../security/shared/AdminList.module.css';
import { DOCS, LEVELS, type Level, LOGGERS, LOGGING_PAGE, LOGGING_UPDATE, type Logger } from './api';
import styles from './Support.module.css';

const TONES: Record<Level, string | undefined> = {
  OFF: styles.levelOff,
  ERROR: styles.levelError,
  WARN: styles.levelWarn,
  INFO: styles.levelInfo,
  DEBUG: styles.levelDebug,
  TRACE: styles.levelTrace,
};

export function LevelBadge({ level }: { level: Level }) {
  return <span className={`${styles.level} ${TONES[level]}`}>{level}</span>;
}

export function loggerHref(name: string) {
  return `${LOGGING_PAGE}/${encodeURIComponent(name)}`;
}

export function filterLoggers(loggers: readonly Logger[], filters: FilterState, query: string): Logger[] {
  const levels = filters.level ?? [];
  const words = query.trim().toLowerCase();
  return loggers.filter(
    (logger) =>
      (levels.length === 0 || levels.includes(logger.level)) && logger.name.toLowerCase().includes(words),
  );
}

export default function LoggingPage() {
  const t = useT();
  const toast = useToast();
  const navigate = useNavigate();
  const invalidate = useInvalidateApi();
  const collator = useCollator();
  const canUpdate = useCan(LOGGING_UPDATE);
  const loggers = useApi<Logger[]>(LOGGERS);
  const [query, setQuery] = useState('');
  const [filters, setFilters] = useState<FilterState>({});
  const [resetting, setResetting] = useState(false);
  const [sort, setSort] = useState<{ column: string; direction: 'asc' | 'desc' }>({
    column: 'name',
    direction: 'asc',
  });
  const all = loggers.data ?? [];
  const sections = [
    {
      id: 'level',
      title: t('support.logging.levelFilter'),
      options: LEVELS.map((level) => ({
        value: level,
        label: level,
        count: all.filter((logger) => logger.level === level).length,
      })),
    },
  ];
  const columns: ListColumn<Logger>[] = [
    {
      id: 'name',
      header: t('support.logging.name'),
      sortValue: (logger) => logger.name,
      cell: (logger) => logger.name,
    },
    {
      id: 'level',
      header: t('support.logging.level'),
      width: '160px',
      sortValue: (logger) => LEVELS.indexOf(logger.level),
      cell: (logger) => <LevelBadge level={logger.level} />,
    },
  ];
  const column = columns.find((candidate) => candidate.id === sort.column);
  const rows = useMemo(
    () => sortList(filterLoggers(all, filters, query), column?.sortValue, sort.direction, collator),
    [all, filters, query, column, sort.direction, collator],
  );
  const reset = async () => {
    await api.post(`${LOGGERS}/reset`);
    toast.success(t('support.logging.resetDone'));
    await invalidate(LOGGERS);
  };
  return (
    <Page
      gutter="small"
      title={t('support.logging.title')}
      description={t('support.logging.intro')}
      breadcrumbs={[
        { label: t('support.crumbs.settings'), to: '/settings' },
        { label: t('support.logging.title') },
      ]}
      actions={
        canUpdate && (
          <div className={styles.actions}>
            <Button size="3" variant="outline" color="gray" onClick={() => setResetting(true)}>
              <RotateCcw size={16} aria-hidden="true" />
              {t('support.logging.reset')}
            </Button>
            <Button asChild size="3">
              <Link to={`${LOGGING_PAGE}/create`}>
                <Plus size={16} aria-hidden="true" />
                {t('support.logging.create')}
              </Link>
            </Button>
          </div>
        )
      }
    >
      <ListLayout
        filters={
          <FilterPanel
            sections={sections}
            selected={filters}
            onChange={(section, values) => setFilters({ ...filters, [section]: values })}
          />
        }
      >
        <div className={styles.filterRow}>
          <SearchBox value={query} onChange={setQuery} placeholder={t('support.logging.filter')} />
        </div>
        {loggers.error ? (
          <ErrorState error={loggers.error} onRetry={() => void loggers.refetch()} />
        ) : (
          <>
            <div className={styles.showing}>
              {t('support.logging.showing', { shown: rows.length, total: all.length })}
            </div>
            <ListTable
              label={t('support.logging.title')}
              columns={columns}
              rows={rows}
              loading={loggers.isPending}
              getRowId={(logger) => logger.name}
              rowLabel={(logger) => t('support.logging.edit', { name: logger.name })}
              onOpen={(logger) => navigate(loggerHref(logger.name))}
              buttons={(logger) =>
                canUpdate
                  ? [
                      {
                        label: t('support.logging.edit', { name: logger.name }),
                        icon: Pencil,
                        onClick: () => navigate(loggerHref(logger.name)),
                      },
                    ]
                  : []
              }
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
                  {query || (filters.level ?? []).length > 0
                    ? t('support.logging.noMatches')
                    : t('support.logging.empty')}
                </div>
              }
            />
          </>
        )}
        <HelpCard title={t('support.logging.helpTitle')} link={DOCS} linkLabel={t('support.common.docs')}>
          {t('support.logging.helpText')}
        </HelpCard>
      </ListLayout>
      <ConfirmDialog
        open={resetting}
        onOpenChange={setResetting}
        title={t('support.logging.resetTitle')}
        description={t('support.logging.resetText')}
        confirmLabel={t('support.logging.resetConfirm')}
        danger
        onConfirm={reset}
      />
    </Page>
  );
}
