import { Callout, Heading, Select, Text } from '@radix-ui/themes';
import { keepPreviousData, useQuery } from '@tanstack/react-query';
import { Info, SearchX } from 'lucide-react';
import { useId } from 'react';
import { Link } from 'react-router';
import { ApiError, api, useApi } from '../../api';
import { useT } from '../../i18n';
import { type Column, DataTable, EmptyState, FieldShell, FormatBadge } from '../../ui';
import {
  ALL_REPOSITORIES,
  browseHref,
  PREVIEW,
  type Preview,
  type PreviewMatch,
  REPOSITORIES,
  type Repository,
} from './model';
import styles from './Selectors.module.css';
import { useDebounced } from './useDebounced';

export interface PreviewState {
  data: Preview | undefined;
  problem: string | null;
  error: unknown;
  loading: boolean;
  checked: boolean;
}

export function expressionProblem(error: unknown): string | null {
  if (error instanceof ApiError && error.status === 400) {
    return error.fieldErrors.find((problem) => problem.id === 'expression')?.message ?? error.message;
  }
  return null;
}

export function usePreview(expression: string, repository: string): PreviewState {
  const settled = useDebounced(expression.trim(), 400);
  const query = useQuery<Preview, ApiError>({
    queryKey: ['selectors', 'preview', settled, repository],
    queryFn: ({ signal }) => api.post<Preview>(PREVIEW, { expression: settled, repository }, { signal }),
    enabled: settled !== '',
    retry: false,
    staleTime: 10_000,
    placeholderData: keepPreviousData,
  });
  const problem = settled === '' ? null : expressionProblem(query.error);
  return {
    data: settled === '' || query.isError ? undefined : query.data,
    problem,
    error: problem ? null : query.error,
    loading: settled !== '' && (query.isFetching || settled !== expression.trim()),
    checked: settled !== '' && query.isSuccess && !query.isPlaceholderData,
  };
}

export function SelectorPreview({
  state,
  repository,
  onRepositoryChange,
}: {
  state: PreviewState;
  repository: string;
  onRepositoryChange(value: string): void;
}) {
  const t = useT();
  const pickerId = useId();
  const repositories = useApi<Repository[]>(REPOSITORIES);
  const names = (repositories.data ?? [])
    .map((item) => item.name)
    .sort((left, right) => left.localeCompare(right));

  const columns: Column<PreviewMatch>[] = [
    {
      id: 'path',
      header: t('selectors.preview.path'),
      cell: (row) => (
        <Link to={browseHref(row.repository, row.path)} className={styles.mono}>
          {row.path}
        </Link>
      ),
      primary: true,
    },
    { id: 'repository', header: t('selectors.preview.repository'), cell: (row) => row.repository },
    {
      id: 'format',
      header: t('selectors.preview.format'),
      cell: (row) => <FormatBadge format={row.format} />,
    },
  ];

  return (
    <section className={styles.preview} aria-labelledby={`${pickerId}-title`}>
      <Heading as="h2" size="4" id={`${pickerId}-title`}>
        {t('selectors.preview.title')}
      </Heading>
      <Text size="2" color="gray">
        {t('selectors.preview.intro')}
      </Text>
      <div className={styles.toolbar}>
        <div className={styles.picker}>
          <FieldShell id={pickerId} label={t('selectors.preview.repositoryPicker')}>
            <Select.Root value={repository} onValueChange={onRepositoryChange}>
              <Select.Trigger id={pickerId} style={{ width: '100%' }} />
              <Select.Content position="popper">
                <Select.Item value={ALL_REPOSITORIES}>{t('selectors.preview.allRepositories')}</Select.Item>
                {names.map((name) => (
                  <Select.Item key={name} value={name}>
                    {name}
                  </Select.Item>
                ))}
              </Select.Content>
            </Select.Root>
          </FieldShell>
        </div>
      </div>
      {state.data?.more && (
        <Callout.Root color="gray" variant="surface" role="status">
          <Callout.Icon>
            <Info size={16} />
          </Callout.Icon>
          <Callout.Text>{t('selectors.preview.more', { count: state.data.items.length })}</Callout.Text>
        </Callout.Root>
      )}
      {state.problem ? (
        <Text size="2" color="gray">
          {t('selectors.preview.fixFirst')}
        </Text>
      ) : (
        <DataTable
          label={t('selectors.preview.results')}
          columns={columns}
          rows={state.data?.items ?? (state.loading ? undefined : [])}
          getRowId={(row) => row.assetId}
          loading={state.loading && !state.data}
          error={state.error ?? undefined}
          pageSize={25}
          paging="more"
          empty={
            <EmptyState
              icon={SearchX}
              title={state.checked ? t('selectors.preview.none') : t('selectors.preview.waiting')}
            />
          }
        />
      )}
    </section>
  );
}
