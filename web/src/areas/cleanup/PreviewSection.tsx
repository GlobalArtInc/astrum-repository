import { Button, Callout, Flex, Select, Spinner, Table, Text } from '@radix-ui/themes';
import { useQuery } from '@tanstack/react-query';
import { Calculator, Download, Info } from 'lucide-react';
import { useEffect, useMemo, useState } from 'react';
import { api, type RepositoryType, useApi, V1 } from '../../api';
import { useI18n, useT } from '../../i18n';
import { describeError, FileSize } from '../../ui';
import styles from './Cleanup.module.css';
import {
  ALL_FORMATS,
  csvHref,
  hasCriteria,
  PAGE_SIZE,
  type PolicyValues,
  PREVIEW,
  type PreviewPage,
  SUMMARY,
  type Summary,
  toRequest,
} from './model';

interface Repository {
  name: string;
  format: string;
  type: RepositoryType;
}

function useDebounced<T>(value: T, delay: number): T {
  const [current, setCurrent] = useState(value);
  useEffect(() => {
    const timer = window.setTimeout(() => setCurrent(value), delay);
    return () => window.clearTimeout(timer);
  }, [value, delay]);
  return current;
}

function SummaryCard({
  ready,
  repository,
  request,
}: {
  ready: boolean;
  repository: string;
  request: ReturnType<typeof toRequest>;
}) {
  const t = useT();
  const { language } = useI18n();
  const [summary, setSummary] = useState<Summary | null>(null);
  const [failure, setFailure] = useState<unknown>(null);
  const [calculating, setCalculating] = useState(false);
  const key = JSON.stringify({ ...request, repository });
  const [seen, setSeen] = useState(key);
  if (seen !== key) {
    setSeen(key);
    setSummary(null);
    setFailure(null);
  }
  const number = new Intl.NumberFormat(language);

  const calculate = async () => {
    setCalculating(true);
    setFailure(null);
    try {
      setSummary(await api.post<Summary>(SUMMARY, { ...request, repository }));
    } catch (error) {
      setFailure(error);
    } finally {
      setCalculating(false);
    }
  };

  return (
    <div className={styles.summary}>
      <div className={styles.summaryHead}>
        <div>
          <p className={styles.summaryTitle}>{t('cleanup.summary.title')}</p>
          <p className={styles.summaryText}>{t('cleanup.summary.text')}</p>
        </div>
        <Button
          type="button"
          size="3"
          variant="outline"
          color="gray"
          className={styles.secondary}
          disabled={!ready || calculating}
          onClick={() => void calculate()}
        >
          {calculating ? <Spinner size="2" /> : <Calculator size={16} aria-hidden="true" />}
          {calculating ? t('cleanup.summary.calculating') : t('cleanup.summary.calculate')}
        </Button>
      </div>
      {failure !== null && (
        <Text as="p" size="2" color="red" role="alert" className={styles.summaryResult}>
          {t('cleanup.summary.failed')} {describeError(t, failure)}
        </Text>
      )}
      {summary &&
        (summary.componentCount === 0 && summary.complete ? (
          <Callout.Root variant="surface" className={styles.summaryResult}>
            <Callout.Icon>
              <Info size={16} />
            </Callout.Icon>
            <Callout.Text>
              {t.rich('cleanup.summary.nothing', { repository: <b key="r">{summary.repository}</b> })}
            </Callout.Text>
          </Callout.Root>
        ) : (
          <div className={styles.summaryResult}>
            <Text as="p" size="2">
              {t.rich(summary.complete ? 'cleanup.summary.result' : 'cleanup.summary.atLeast', {
                repository: <b key="r">{summary.repository}</b>,
              })}
            </Text>
            <dl className={styles.figures}>
              <div>
                <dt>{t('cleanup.summary.components')}</dt>
                <dd>{number.format(summary.componentCount)}</dd>
              </div>
              <div>
                <dt>{t('cleanup.summary.files')}</dt>
                <dd>{number.format(summary.assetCount)}</dd>
              </div>
              <div>
                <dt>{t('cleanup.summary.size')}</dt>
                <dd>
                  <FileSize bytes={summary.totalSize} />
                </dd>
              </div>
            </dl>
            {!summary.complete && (
              <Text as="p" size="1" color="amber" mt="2">
                {t('cleanup.summary.incomplete')}
              </Text>
            )}
          </div>
        ))}
    </div>
  );
}

export function PreviewSection({
  values,
  criteria,
  valid,
}: {
  values: PolicyValues;
  criteria: Set<string>;
  valid: boolean;
}) {
  const t = useT();
  const repositories = useApi<Repository[]>(`${V1}/repositories`);
  const [repository, setRepository] = useState('');
  const [page, setPage] = useState(0);
  const choices = useMemo(
    () =>
      (repositories.data ?? [])
        .filter(
          (candidate) =>
            candidate.type !== 'group' &&
            (values.format === ALL_FORMATS || candidate.format === values.format),
        )
        .sort((left, right) => left.name.localeCompare(right.name)),
    [repositories.data, values.format],
  );
  const request = useMemo(() => toRequest(values, criteria), [values, criteria]);
  const ready = valid && repository !== '' && hasCriteria(values, criteria);
  const body = useDebounced(ready ? JSON.stringify({ ...request, repository }) : null, 400);
  const requestKey = JSON.stringify({ ...request, repository });
  const [seenKey, setSeenKey] = useState(requestKey);
  if (seenKey !== requestKey) {
    setSeenKey(requestKey);
    setPage(0);
  }

  useEffect(() => {
    if (repository !== '' && !choices.some((candidate) => candidate.name === repository)) {
      setRepository('');
    }
  }, [choices, repository]);

  const preview = useQuery({
    queryKey: ['cleanup-preview', body, page],
    queryFn: () =>
      api.post<PreviewPage>(`${PREVIEW}?page=${page}&size=${PAGE_SIZE}`, JSON.parse(body ?? '{}')),
    enabled: body !== null,
  });

  const total = preview.data?.total ?? 0;
  const results = preview.data?.results ?? [];
  const pages = total > 0 ? Math.ceil(total / PAGE_SIZE) : 0;

  return (
    <div>
      <p className={styles.previewText}>{t('cleanup.preview.text')}</p>
      <div className={styles.previewBar}>
        <div className={styles.repositoryPicker}>
          <Select.Root value={repository === '' ? undefined : repository} onValueChange={setRepository}>
            <Select.Trigger
              placeholder={t('cleanup.preview.chooseRepository')}
              aria-label={t('cleanup.preview.repository')}
            />
            <Select.Content position="popper">
              {choices.map((choice) => (
                <Select.Item key={choice.name} value={choice.name}>
                  {choice.name}
                </Select.Item>
              ))}
            </Select.Content>
          </Select.Root>
        </div>
        {ready ? (
          <Button asChild size="3" variant="outline" color="gray" className={styles.secondary}>
            <a href={csvHref(values, criteria, repository)} download>
              <Download size={16} aria-hidden="true" />
              {t('cleanup.preview.csv')}
            </a>
          </Button>
        ) : (
          <Button
            type="button"
            size="3"
            variant="outline"
            color="gray"
            className={styles.secondary}
            disabled
            title={t('cleanup.preview.needRepository')}
          >
            <Download size={16} aria-hidden="true" />
            {t('cleanup.preview.csv')}
          </Button>
        )}
      </div>
      {choices.length === 0 && !repositories.isPending && (
        <Text as="p" size="2" color="gray" mt="2">
          {t('cleanup.preview.noRepositories')}
        </Text>
      )}
      <SummaryCard ready={ready} repository={repository} request={request} />
      {ready && (
        <div className={styles.results}>
          {preview.isFetching && !preview.data && (
            <Flex justify="center" py="4" gap="2" align="center">
              <Spinner />
              <Text size="2" color="gray">
                {t('cleanup.preview.loading')}
              </Text>
            </Flex>
          )}
          {preview.error && (
            <Text size="2" color="red" role="alert">
              {describeError(t, preview.error)}
            </Text>
          )}
          {preview.data && (
            <>
              {total > 0 && (
                <Callout.Root color="amber" variant="soft">
                  <Callout.Icon>
                    <Info size={16} />
                  </Callout.Icon>
                  <Callout.Text>
                    {t('cleanup.preview.showing', {
                      from: page * PAGE_SIZE + 1,
                      to: page * PAGE_SIZE + results.length,
                      total,
                    })}
                  </Callout.Text>
                </Callout.Root>
              )}
              <Table.Root variant="ghost" className={styles.table} aria-label={t('cleanup.preview.results')}>
                <Table.Header>
                  <Table.Row>
                    <Table.ColumnHeaderCell>{t('cleanup.preview.name')}</Table.ColumnHeaderCell>
                    <Table.ColumnHeaderCell>{t('cleanup.preview.group')}</Table.ColumnHeaderCell>
                    <Table.ColumnHeaderCell>{t('cleanup.preview.version')}</Table.ColumnHeaderCell>
                  </Table.Row>
                </Table.Header>
                <Table.Body>
                  {results.map((component) => (
                    <Table.Row key={component.id}>
                      <Table.Cell className={styles.mono}>{component.name}</Table.Cell>
                      <Table.Cell className={styles.mono}>{component.group || '—'}</Table.Cell>
                      <Table.Cell>{component.version || '—'}</Table.Cell>
                    </Table.Row>
                  ))}
                  {results.length === 0 && (
                    <Table.Row>
                      <Table.Cell colSpan={3} justify="center">
                        <Text color="gray">{t('cleanup.preview.none')}</Text>
                      </Table.Cell>
                    </Table.Row>
                  )}
                </Table.Body>
              </Table.Root>
              {pages > 1 && (
                <div className={styles.pager}>
                  <Button
                    type="button"
                    variant="soft"
                    color="gray"
                    disabled={page === 0}
                    onClick={() => setPage(page - 1)}
                  >
                    {t('cleanup.preview.previous')}
                  </Button>
                  <Text size="2" color="gray">
                    {t('cleanup.preview.page', { page: page + 1, pages })}
                  </Text>
                  <Button
                    type="button"
                    variant="soft"
                    color="gray"
                    disabled={page + 1 >= pages}
                    onClick={() => setPage(page + 1)}
                  >
                    {t('cleanup.preview.next')}
                  </Button>
                </div>
              )}
            </>
          )}
        </div>
      )}
    </div>
  );
}
