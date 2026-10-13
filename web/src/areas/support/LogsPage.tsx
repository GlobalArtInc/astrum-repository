import { FileText } from 'lucide-react';
import { useMemo, useState } from 'react';
import { useNavigate } from 'react-router';
import { useApi } from '../../api';
import { useT } from '../../i18n';
import { DateTime, ErrorState, FileSize, Page } from '../../ui';
import {
  HelpCard,
  type ListColumn,
  ListTable,
  SearchBox,
  sortList,
  useCollator,
} from '../security/shared/AdminList';
import list from '../security/shared/AdminList.module.css';
import { DOCS, LOGS, LOGS_PAGE, type LogFile } from './api';
import styles from './Support.module.css';

export default function LogsPage() {
  const t = useT();
  const navigate = useNavigate();
  const collator = useCollator();
  const files = useApi<LogFile[]>(LOGS);
  const [filter, setFilter] = useState('');
  const [sort, setSort] = useState<{ column: string; direction: 'asc' | 'desc' }>({
    column: 'name',
    direction: 'asc',
  });
  const columns: ListColumn<LogFile>[] = [
    {
      id: 'name',
      header: t('support.logs.fileName'),
      sortValue: (file) => file.fileName,
      cell: (file) => (
        <span className={styles.fileName}>
          <FileText size={16} aria-hidden="true" />
          {file.fileName}
        </span>
      ),
    },
    {
      id: 'size',
      header: t('support.logs.size'),
      width: '120px',
      sortValue: (file) => file.size,
      cell: (file) => <FileSize bytes={file.size} />,
    },
    {
      id: 'modified',
      header: t('support.logs.modified'),
      width: '200px',
      sortValue: (file) => file.lastModified ?? 0,
      cell: (file) =>
        file.lastModified ? <DateTime value={new Date(file.lastModified).toISOString()} /> : '—',
    },
  ];
  const column = columns.find((candidate) => candidate.id === sort.column);
  const rows = useMemo(() => {
    const words = filter.trim().toLowerCase();
    const shown = (files.data ?? []).filter((file) => file.fileName.toLowerCase().includes(words));
    return sortList(shown, column?.sortValue, sort.direction, collator);
  }, [files.data, filter, column, sort.direction, collator]);
  return (
    <Page
      gutter="small"
      title={t('support.logs.title')}
      description={t('support.logs.intro')}
      breadcrumbs={[
        { label: t('support.crumbs.settings'), to: '/settings' },
        { label: t('support.logs.title') },
      ]}
    >
      <div className={styles.stack}>
        <div className={styles.filterRow}>
          <SearchBox value={filter} onChange={setFilter} placeholder={t('support.logs.filter')} />
        </div>
        {files.error ? (
          <ErrorState error={files.error} onRetry={() => void files.refetch()} />
        ) : (
          <ListTable
            plain
            chevron
            label={t('support.logs.title')}
            columns={columns}
            rows={rows}
            loading={files.isPending}
            getRowId={(file) => file.fileName}
            rowLabel={(file) => t('support.logs.open', { name: file.fileName })}
            onOpen={(file) => navigate(`${LOGS_PAGE}/${encodeURIComponent(file.fileName)}`)}
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
                {filter ? t('support.logs.noMatches') : t('support.logs.empty')}
              </div>
            }
          />
        )}
        <HelpCard title={t('support.logs.helpTitle')} link={DOCS} linkLabel={t('support.common.docs')}>
          {t('support.logs.helpText')}
        </HelpCard>
      </div>
    </Page>
  );
}
