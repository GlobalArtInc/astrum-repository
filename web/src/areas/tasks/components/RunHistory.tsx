import { Button, Flex, Heading, Spinner, Table, Text } from '@radix-ui/themes';
import { Clock, History, ScrollText } from 'lucide-react';
import { useState } from 'react';
import { path, useApi } from '../../../api';
import { useT } from '../../../i18n';
import { EmptyState, ErrorState } from '../../../ui';
import { isActive, type LogPage, type Run, scheduleKind, type Task, UI_TASKS } from '../model';
import styles from '../Tasks.module.css';
import { duration, ResultBadge, resultLabel, scheduleLabel, useLocalTime } from './bits';

const POLL = 2000;

function RunLog({ task, run }: { task: Task; run: Run }) {
  const t = useT();
  const time = useLocalTime();
  const finished = run.state !== 'running' && run.state !== 'waiting';
  const log = useApi<LogPage>(`${UI_TASKS}${path`/${task.id}/runs/${run.id}/log`}`, {
    refetchInterval: (query) => (query.state.data?.finished ? false : POLL),
  });
  if (log.isPending) {
    return (
      <Flex justify="center" py="4">
        <Spinner />
      </Flex>
    );
  }
  if (log.error) {
    return <ErrorState error={log.error} onRetry={() => void log.refetch()} />;
  }
  return (
    <pre className={styles.log} role="log" aria-label={t('tasks.history.log')}>
      {log.data.lines.map((line) => (
        <div key={line.line} data-level={line.level}>
          {`${time(line.at)}  ${line.level.padEnd(5)}  ${line.message}`}
        </div>
      ))}
      {log.data.lines.length === 0 && t('tasks.history.emptyLog')}
      {!finished && !log.data.finished && <Spinner size="1" />}
    </pre>
  );
}

export function RunHistory({ task }: { task: Task }) {
  const t = useT();
  const time = useLocalTime();
  const [open, setOpen] = useState<number | null>(null);
  const runs = useApi<Run[]>(`${UI_TASKS}${path`/${task.id}/runs`}`, {
    refetchInterval: isActive(task) ? POLL : false,
  });
  const last = runs.data?.find((run) => run.state !== 'waiting') ?? null;
  const kind = scheduleKind(task.frequency);
  const scheduleText =
    kind === 'cron' && task.frequency.cronExpression
      ? t('tasks.history.cron', { expression: task.frequency.cronExpression })
      : scheduleLabel(t, kind);
  return (
    <div className={styles.section}>
      <Flex align="center" gap="2">
        <History size={18} color="var(--accent-9)" aria-hidden="true" />
        <Heading as="h2" size="4" weight="medium">
          {t('tasks.history.title')}
        </Heading>
      </Flex>
      {runs.isPending ? (
        <Flex justify="center" py="6">
          <Spinner size="3" />
        </Flex>
      ) : runs.error ? (
        <ErrorState error={runs.error} onRetry={() => void runs.refetch()} />
      ) : (
        <>
          <div className={styles.card}>
            {last ? (
              <div className={styles.lastRun}>
                <Flex direction="column" gap="1">
                  <Heading as="h3" size="3" weight="medium">
                    {t('tasks.history.lastExecution')}
                  </Heading>
                  <Text size="1" color="gray">
                    {t('tasks.history.started')}
                  </Text>
                  <Text size="2">{time(last.startedAt ?? last.queuedAt)}</Text>
                  <Text size="1" color="gray" mt="2">
                    {t('tasks.history.details')}
                  </Text>
                  <Text size="2">{last.message || t('tasks.history.noDetails')}</Text>
                </Flex>
                <Flex direction="column" gap="1" align="end">
                  <Text size="1" color="gray">
                    {t('tasks.history.result')}
                  </Text>
                  <ResultBadge state={last.state} label={resultLabel(t, last.state)} />
                </Flex>
              </div>
            ) : (
              <EmptyState
                icon={History}
                title={t('tasks.history.never')}
                description={t('tasks.history.neverText')}
              />
            )}
            <hr className={styles.divider} />
            <Flex direction="column" gap="1">
              <Text size="2" color="gray">
                {t('tasks.history.nextRun')}
              </Text>
              <Flex align="center" gap="2">
                <Clock size={14} color="var(--accent-9)" aria-hidden="true" />
                <Text size="2">{task.nextRun ? time(task.nextRun) : t('tasks.summary.none')}</Text>
              </Flex>
            </Flex>
            <hr className={styles.divider} />
            <Flex direction="column" gap="1">
              <Text size="2" color="gray">
                {t('tasks.history.schedule')}
              </Text>
              <Text size="1" style={{ fontFamily: 'var(--code-font-family)' }}>
                {scheduleText}
              </Text>
            </Flex>
          </div>
          {runs.data.length > 0 && (
            <Table.Root variant="surface" size="1" aria-label={t('tasks.history.runs')}>
              <Table.Header>
                <Table.Row>
                  <Table.ColumnHeaderCell>{t('tasks.history.started')}</Table.ColumnHeaderCell>
                  <Table.ColumnHeaderCell>{t('tasks.history.trigger')}</Table.ColumnHeaderCell>
                  <Table.ColumnHeaderCell>{t('tasks.history.duration')}</Table.ColumnHeaderCell>
                  <Table.ColumnHeaderCell>{t('tasks.history.result')}</Table.ColumnHeaderCell>
                  <Table.ColumnHeaderCell>
                    <span className="visually-hidden">{t('tasks.history.log')}</span>
                  </Table.ColumnHeaderCell>
                </Table.Row>
              </Table.Header>
              <Table.Body>
                {runs.data.map((run) => (
                  <RunRow
                    key={run.id}
                    task={task}
                    run={run}
                    open={open === run.id}
                    toggle={() => setOpen(open === run.id ? null : run.id)}
                  />
                ))}
              </Table.Body>
            </Table.Root>
          )}
        </>
      )}
    </div>
  );
}

function RunRow({ task, run, open, toggle }: { task: Task; run: Run; open: boolean; toggle(): void }) {
  const t = useT();
  const time = useLocalTime();
  return (
    <>
      <Table.Row>
        <Table.Cell>{time(run.startedAt ?? run.queuedAt)}</Table.Cell>
        <Table.Cell>
          {run.trigger === 'schedule'
            ? t('tasks.history.scheduled')
            : t('tasks.history.manual', { user: run.initiator })}
        </Table.Cell>
        <Table.Cell>{duration(t, run.durationMs)}</Table.Cell>
        <Table.Cell>
          <ResultBadge state={run.state} label={resultLabel(t, run.state)} />
        </Table.Cell>
        <Table.Cell justify="end">
          <Button size="1" variant="ghost" onClick={toggle} aria-expanded={open}>
            <ScrollText size={14} aria-hidden="true" />
            {open ? t('tasks.history.hideLog') : t('tasks.history.showLog')}
          </Button>
        </Table.Cell>
      </Table.Row>
      {open && (
        <Table.Row>
          <Table.Cell colSpan={5}>
            {run.message && (
              <Text as="p" size="2" mb="2">
                {run.message}
              </Text>
            )}
            <RunLog task={task} run={run} />
          </Table.Cell>
        </Table.Row>
      )}
    </>
  );
}
