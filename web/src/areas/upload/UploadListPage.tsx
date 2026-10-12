import { TextField as RadixTextField } from '@radix-ui/themes';
import { Search, Upload } from 'lucide-react';
import { useId, useMemo, useState } from 'react';
import { useT } from '../../i18n';
import {
  type Column,
  DataTable,
  EmptyState,
  FieldShell,
  FormatBadge,
  formatName,
  Page,
  RelativeTime,
  SelectField,
} from '../../ui';
import type { UploadRepository } from './model';
import { useUploadTargets } from './targets';
import styles from './Upload.module.css';

const ALL = '*';

export default function UploadListPage() {
  const t = useT();
  const filterId = useId();
  const { targets, loading, error, retry } = useUploadTargets();
  const [filter, setFilter] = useState('');
  const [format, setFormat] = useState(ALL);
  const formats = useMemo(
    () =>
      [...new Set(targets.map((target) => target.format))]
        .map((id) => ({ value: id, label: formatName(id) }))
        .sort((left, right) => left.label.localeCompare(right.label)),
    [targets],
  );
  const wanted = filter.trim().toLowerCase();
  const shown = targets.filter(
    (target) =>
      (format === ALL || target.format === format) &&
      (wanted === '' || target.name.toLowerCase().includes(wanted)),
  );
  const columns: Column<UploadRepository>[] = [
    {
      id: 'name',
      header: t('upload.list.name'),
      primary: true,
      cell: (row) => row.name,
      sortValue: (row) => row.name,
    },
    {
      id: 'format',
      header: t('upload.list.format'),
      cell: (row) => <FormatBadge format={row.format} />,
      sortValue: (row) => formatName(row.format),
    },
    {
      id: 'components',
      header: t('upload.list.components'),
      align: 'right',
      cell: (row) => (row.componentCount === null ? '—' : row.componentCount),
      sortValue: (row) => row.componentCount,
    },
    {
      id: 'last',
      header: t('upload.list.lastUpload'),
      hideOnPhone: true,
      cell: (row) => (row.lastUploaded ? <RelativeTime value={row.lastUploaded} /> : '—'),
      sortValue: (row) => (row.lastUploaded ? new Date(row.lastUploaded) : null),
    },
  ];
  return (
    <Page title={t('upload.list.title')} description={t('upload.list.description')}>
      {targets.length > 0 && (
        <div className={styles.filters}>
          <FieldShell id={filterId} label={t('upload.list.filter')}>
            <RadixTextField.Root
              id={filterId}
              type="search"
              value={filter}
              placeholder={t('upload.list.filterPlaceholder')}
              onChange={(event) => setFilter(event.target.value)}
            >
              <RadixTextField.Slot>
                <Search size={16} aria-hidden="true" />
              </RadixTextField.Slot>
            </RadixTextField.Root>
          </FieldShell>
          <SelectField
            label={t('upload.list.format')}
            value={format}
            options={[{ value: ALL, label: t('upload.list.allFormats') }, ...formats]}
            onChange={setFormat}
          />
        </div>
      )}
      <DataTable
        label={t('upload.list.label')}
        columns={columns}
        rows={loading ? undefined : shown}
        getRowId={(row) => row.name}
        loading={loading}
        error={error}
        onRetry={retry}
        rowHref={(row) => `/upload/${encodeURIComponent(row.name)}`}
        initialSort={{ column: 'name', direction: 'asc' }}
        empty={
          targets.length === 0 ? (
            <EmptyState
              icon={Upload}
              title={t('upload.list.noneTitle')}
              description={t('upload.list.noneText')}
            />
          ) : (
            <EmptyState icon={Search} title={t('upload.list.noMatches')} />
          )
        }
      />
    </Page>
  );
}
