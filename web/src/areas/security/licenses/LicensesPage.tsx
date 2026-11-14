import { Badge, Button, Flex, Select, TextField } from '@radix-ui/themes';
import { Download } from 'lucide-react';
import { useState } from 'react';
import { useApi } from '../../../api';
import { useT } from '../../../i18n';
import { type Column, DataTable, EmptyState, FORMATS, FormatBadge, formatName, Page } from '../../../ui';
import { usePagedSearch } from '../../search/usePagedSearch';
import { REPOSITORIES, type Repository } from '../api';
import {
  LICENCE_STATES,
  type LicenceFilters,
  type LicenceRow,
  type LicenceState,
  type LicenceVerdict,
} from './api';
import styles from './Licences.module.css';
import { exportPath, licencePath, NO_FILTERS, rowKey, STATE_COLORS, VERDICT_COLORS } from './model';

const ALL = 'all';

const STATE_LABELS: Record<LicenceState, string> = {
  spdx: 'licences.state.spdx',
  mapped: 'licences.state.mapped',
  unknown: 'licences.state.unknown',
  none: 'licences.state.none',
};

const VERDICT_LABELS: Record<LicenceVerdict, string> = {
  pass: 'licences.verdict.pass',
  blocked: 'licences.verdict.blocked',
  reported: 'licences.verdict.reported',
  unjudged: 'licences.verdict.unjudged',
};

export default function LicensesPage() {
  const t = useT();
  const [filters, setFilters] = useState<LicenceFilters>(NO_FILTERS);
  const repositories = useApi<Repository[]>(REPOSITORIES);
  const licences = usePagedSearch<LicenceRow>(licencePath(filters, null), 1);
  const formats = Object.keys(FORMATS).sort();

  const change = (patch: Partial<LicenceFilters>) => setFilters((current) => ({ ...current, ...patch }));

  const columns: Column<LicenceRow>[] = [
    {
      id: 'repository',
      header: t('licences.column.repository'),
      cell: (row) => row.repository,
      primary: true,
    },
    { id: 'format', header: t('licences.column.format'), cell: (row) => <FormatBadge format={row.format} /> },
    {
      id: 'namespace',
      header: t('licences.column.namespace'),
      cell: (row) => row.namespace,
      hideOnPhone: true,
    },
    { id: 'name', header: t('licences.column.name'), cell: (row) => row.name },
    { id: 'version', header: t('licences.column.version'), cell: (row) => row.version, hideOnPhone: true },
    { id: 'declared', header: t('licences.column.declared'), cell: (row) => row.declared },
    {
      id: 'expression',
      header: t('licences.column.expression'),
      cell: (row) => row.expression ?? '',
      hideOnPhone: true,
    },
    { id: 'ids', header: t('licences.column.ids'), cell: (row) => row.ids.join(', '), hideOnPhone: true },
    {
      id: 'state',
      header: t('licences.column.state'),
      cell: (row) => (
        <Badge color={STATE_COLORS[row.state]} variant="soft" radius="full">
          {t(STATE_LABELS[row.state])}
        </Badge>
      ),
    },
    {
      id: 'verdict',
      header: t('licences.column.verdict'),
      cell: (row) => (
        <Badge color={VERDICT_COLORS[row.verdict]} variant="soft" radius="full">
          {t(VERDICT_LABELS[row.verdict])}
        </Badge>
      ),
    },
  ];

  return (
    <Page
      title={t('licences.title')}
      description={t('licences.intro')}
      breadcrumbs={[
        { label: t('security.crumbs.settings'), to: '/settings' },
        { label: t('licences.title') },
      ]}
      actions={
        <Button asChild variant="outline">
          <a href={exportPath(filters)}>
            <Download size={14} aria-hidden="true" />
            {t('licences.export')}
          </a>
        </Button>
      }
    >
      <Flex gap="3" wrap="wrap" align="end" className={styles.filters}>
        <div className={styles.field}>
          <span className={styles.label}>{t('licences.filter.repository')}</span>
          <Select.Root
            value={filters.repository || ALL}
            onValueChange={(value) => change({ repository: value === ALL ? '' : value })}
          >
            <Select.Trigger aria-label={t('licences.filter.repository')} />
            <Select.Content position="popper">
              <Select.Item value={ALL}>{t('licences.filter.repositoryAll')}</Select.Item>
              {(repositories.data ?? []).map((repository) => (
                <Select.Item key={repository.name} value={repository.name}>
                  {repository.name}
                </Select.Item>
              ))}
            </Select.Content>
          </Select.Root>
        </div>
        <div className={styles.field}>
          <span className={styles.label}>{t('licences.filter.license')}</span>
          <TextField.Root
            aria-label={t('licences.filter.license')}
            placeholder={t('licences.filter.licensePlaceholder')}
            value={filters.license}
            onChange={(event) => change({ license: event.target.value })}
          />
        </div>
        <div className={styles.field}>
          <span className={styles.label}>{t('licences.filter.state')}</span>
          <Select.Root
            value={filters.state || ALL}
            onValueChange={(value) => change({ state: value === ALL ? '' : value })}
          >
            <Select.Trigger aria-label={t('licences.filter.state')} />
            <Select.Content position="popper">
              <Select.Item value={ALL}>{t('licences.filter.stateAll')}</Select.Item>
              {LICENCE_STATES.map((state) => (
                <Select.Item key={state} value={state}>
                  {t(STATE_LABELS[state])}
                </Select.Item>
              ))}
            </Select.Content>
          </Select.Root>
        </div>
        <div className={styles.field}>
          <span className={styles.label}>{t('licences.filter.format')}</span>
          <Select.Root
            value={filters.format || ALL}
            onValueChange={(value) => change({ format: value === ALL ? '' : value })}
          >
            <Select.Trigger aria-label={t('licences.filter.format')} />
            <Select.Content position="popper">
              <Select.Item value={ALL}>{t('licences.filter.formatAll')}</Select.Item>
              {formats.map((format) => (
                <Select.Item key={format} value={format}>
                  {formatName(format)}
                </Select.Item>
              ))}
            </Select.Content>
          </Select.Root>
        </div>
      </Flex>
      <DataTable
        label={t('licences.table')}
        columns={columns}
        rows={licences.items}
        getRowId={rowKey}
        loading={licences.loading}
        error={licences.error}
        onRetry={licences.retry}
        empty={<EmptyState title={t('licences.empty.title')} description={t('licences.empty.text')} />}
      />
      {licences.hasMore && (
        <div className={styles.more}>
          <Button variant="outline" onClick={licences.loadMore} loading={licences.loadingMore}>
            {t('licences.loadMore')}
          </Button>
        </div>
      )}
    </Page>
  );
}
