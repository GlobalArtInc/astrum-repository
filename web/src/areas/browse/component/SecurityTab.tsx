import { Button, Flex, Heading, Skeleton, Table, Text } from '@radix-ui/themes';
import type { UseQueryResult } from '@tanstack/react-query';
import { useId, useState } from 'react';
import { useInvalidateApi, V1 } from '../../../api';
import { useCan } from '../../../auth/state';
import { formatNumber, type Translate, useI18n, useT } from '../../../i18n';
import { DateTime, EmptyState, ErrorState, StatusBadge, type Tone } from '../../../ui';
import {
  type ComponentSecurity,
  isUnavailable,
  type PolicyRef,
  type Reason,
  type ScanStatus,
  type Verdict,
  type Vulnerability,
} from '../../security/findings/model';
import { SeverityBadge } from '../../security/findings/SeverityBadge';
import { ExceptionDialog } from '../../security/policy/ExceptionDialog';
import type { ExceptionForm } from '../../security/policy/exceptions';
import { EXCEPTION_CREATE } from '../../security/policy/permissions';
import styles from './SecurityTab.module.css';

export interface ComponentCoordinates {
  format: string;
  namespace: string;
  name: string;
  version: string;
}

export interface SecurityEntry {
  id: string;
  repository?: string;
  coordinates?: ComponentCoordinates;
  result: UseQueryResult<ComponentSecurity, Error>;
}

export interface ExceptionTarget {
  advisory: string;
  policy: string;
  coordinates: ComponentCoordinates;
}

type ExceptionAdder = (target: ExceptionTarget) => void;

interface SecurityTabProps {
  entries: SecurityEntry[];
}

interface PanelProps {
  repository?: string;
  coordinates?: ComponentCoordinates;
  data: ComponentSecurity;
  onAdd?: ExceptionAdder;
}

interface VerdictProps {
  policy: PolicyRef | null;
  verdict: Verdict | null;
}

const STATE_TONES = new Map<string, Tone>([
  ['pass', 'success'],
  ['report', 'warning'],
  ['block', 'danger'],
  ['unscanned', 'neutral'],
  ['error', 'danger'],
]);

const STATE_LABELS = new Map<string, string>([
  ['pass', 'browse.security.state.pass'],
  ['report', 'browse.security.state.report'],
  ['block', 'browse.security.state.block'],
  ['unscanned', 'browse.security.state.unscanned'],
  ['error', 'browse.security.state.error'],
]);

const SCAN_LABELS = new Map<string, string>([
  ['queued', 'browse.security.scanState.queued'],
  ['unscanned', 'browse.security.scanState.unscanned'],
  ['scanned', 'browse.security.scanState.scanned'],
  ['incomplete', 'browse.security.scanState.incomplete'],
  ['unsupported', 'browse.security.scanState.unsupported'],
  ['failed', 'browse.security.scanState.failed'],
  ['no-data', 'browse.security.scanState.noData'],
]);

function labelled(t: Translate, labels: Map<string, string>, code: string): string {
  const key = labels.get(code);
  return key === undefined ? code : t(key);
}

function reasonText(t: Translate, reason: Reason): string {
  switch (reason.kind) {
    case 'vulnerability':
      return t('browse.security.reason.vulnerability', {
        advisory: reason.advisory ?? '',
        package: reason.package ?? '',
      });
    case 'exception':
      return t('browse.security.reason.exception', { reason: reason.reason ?? '' });
    case 'unscanned':
      return t('browse.security.reason.unscanned', { scan: reason.scan ?? '' });
    case 'would-block':
      return t('browse.security.reason.wouldBlock');
    case 'policy-error':
      return t('browse.security.reason.policyError', { message: reason.message ?? '' });
    case 'truncated':
      return t('browse.security.reason.truncated', { omitted: reason.omitted ?? 0 });
    default:
      return t('browse.security.reason.other', { kind: reason.kind });
  }
}

function VerdictBanner({ policy, verdict }: VerdictProps) {
  const t = useT();
  if (policy === null) {
    return (
      <Text as="p" size="2" color="gray">
        {t('browse.security.noPolicy')}
      </Text>
    );
  }
  return (
    <Flex direction="column" gap="2" className={styles.banner}>
      <Flex gap="2" align="center" wrap="wrap">
        <Text size="2" weight="bold">
          {t('browse.security.policy', { name: policy.name })}
        </Text>
        {verdict !== null && (
          <>
            <StatusBadge tone={STATE_TONES.get(verdict.state) ?? 'neutral'}>
              {labelled(t, STATE_LABELS, verdict.state)}
            </StatusBadge>
            <StatusBadge tone={verdict.blocked ? 'danger' : 'success'}>
              {verdict.blocked ? t('browse.security.blocked') : t('browse.security.notBlocked')}
            </StatusBadge>
          </>
        )}
      </Flex>
      {verdict === null ? (
        <Text as="p" size="2" color="gray">
          {t('browse.security.notJudged')}
        </Text>
      ) : (
        <>
          <Text as="p" size="1" color="gray">
            {t('browse.security.judgedAt')} <DateTime value={verdict.computedAt} />
          </Text>
          <Text as="p" size="2" weight="medium">
            {t('browse.security.reasons')}
          </Text>
          {verdict.reasons.length === 0 ? (
            <Text as="p" size="2" color="gray">
              {t('browse.security.noReasons')}
            </Text>
          ) : (
            <ul className={styles.reasons}>
              {verdict.reasons.map((reason) => (
                <li key={`${reason.kind}:${reasonText(t, reason)}`}>
                  <Text size="2">{reasonText(t, reason)}</Text>
                </li>
              ))}
            </ul>
          )}
        </>
      )}
    </Flex>
  );
}

interface ScanProps {
  scan: ScanStatus;
}

function ScanFacts({ scan }: ScanProps) {
  const t = useT();
  const { language } = useI18n();
  return (
    <dl className={styles.facts}>
      <div className={styles.fact}>
        <dt>{t('browse.security.scan')}</dt>
        <dd>{labelled(t, SCAN_LABELS, scan.state)}</dd>
      </div>
      <div className={styles.fact}>
        <dt>{t('browse.security.scannedAt')}</dt>
        <dd>{scan.scannedAt === null ? '—' : <DateTime value={scan.scannedAt} />}</dd>
      </div>
      <div className={styles.fact}>
        <dt>{t('browse.security.packages')}</dt>
        <dd>{scan.packages === null ? '—' : formatNumber(language, scan.packages)}</dd>
      </div>
    </dl>
  );
}

interface TableProps {
  rows: Vulnerability[];
  onAdd?: (row: Vulnerability) => void;
}

function VulnerabilityTable({ rows, onAdd }: TableProps) {
  const t = useT();
  const { language } = useI18n();
  if (rows.length === 0) {
    return (
      <Text as="p" size="2" color="gray">
        {t('browse.security.none')}
      </Text>
    );
  }
  return (
    <div className={styles.tableFrame}>
      <Table.Root variant="surface" size="2">
        <caption className="visually-hidden">{t('browse.security.vulnerabilities')}</caption>
        <Table.Header>
          <Table.Row>
            <Table.ColumnHeaderCell>{t('browse.security.columns.advisory')}</Table.ColumnHeaderCell>
            <Table.ColumnHeaderCell>{t('browse.security.columns.severity')}</Table.ColumnHeaderCell>
            <Table.ColumnHeaderCell>{t('browse.security.columns.score')}</Table.ColumnHeaderCell>
            <Table.ColumnHeaderCell>{t('browse.security.columns.fixed')}</Table.ColumnHeaderCell>
            <Table.ColumnHeaderCell>{t('browse.security.columns.markers')}</Table.ColumnHeaderCell>
            {onAdd && (
              <Table.ColumnHeaderCell>{t('browse.security.columns.exception')}</Table.ColumnHeaderCell>
            )}
          </Table.Row>
        </Table.Header>
        <Table.Body>
          {rows.map((row) => (
            <Table.Row key={row.id}>
              <Table.RowHeaderCell>
                <Text as="span" size="2" className={styles.advisory}>
                  {row.id}
                </Text>
                {row.aliases.length > 0 && (
                  <Text as="span" size="1" color="gray" className={styles.summary}>
                    {t('browse.security.aliases', { aliases: row.aliases.join(', ') })}
                  </Text>
                )}
                {row.summary !== '' && (
                  <Text as="span" size="1" color="gray" className={styles.summary}>
                    {row.summary}
                  </Text>
                )}
              </Table.RowHeaderCell>
              <Table.Cell>
                <SeverityBadge severity={row.severity} />
              </Table.Cell>
              <Table.Cell>{row.score === null ? '—' : formatNumber(language, row.score)}</Table.Cell>
              <Table.Cell>{row.fixed ?? '—'}</Table.Cell>
              <Table.Cell>
                <Flex gap="2" wrap="wrap">
                  {row.malicious && <StatusBadge tone="danger">{t('browse.security.malicious')}</StatusBadge>}
                  {row.excepted && <StatusBadge tone="neutral">{t('browse.security.excepted')}</StatusBadge>}
                </Flex>
              </Table.Cell>
              {onAdd && (
                <Table.Cell>
                  {!row.excepted && (
                    <Button size="1" variant="soft" onClick={() => onAdd(row)}>
                      {t('browse.security.addException')}
                    </Button>
                  )}
                </Table.Cell>
              )}
            </Table.Row>
          ))}
        </Table.Body>
      </Table.Root>
    </div>
  );
}

function ComponentSecurityPanel({ repository, coordinates, data, onAdd }: PanelProps) {
  const t = useT();
  const headingId = useId();
  const policy = data.policy?.name;
  const addFor =
    onAdd !== undefined && policy !== undefined && coordinates !== undefined
      ? (row: Vulnerability) => onAdd({ advisory: row.id, policy, coordinates })
      : undefined;
  return (
    <section className={styles.card} aria-labelledby={headingId}>
      <Heading as="h2" size="5" weight="bold" id={headingId} className={styles.cardTitle}>
        {repository === undefined
          ? t('browse.component.securityTab')
          : t('browse.security.repository', { name: repository })}
      </Heading>
      <VerdictBanner policy={data.policy} verdict={data.verdict} />
      <ScanFacts scan={data.scan} />
      <VulnerabilityTable rows={data.vulnerabilities} onAdd={addFor} />
      {data.truncated && (
        <Text as="p" size="2" color="amber" role="status">
          {t('browse.security.truncated')}
        </Text>
      )}
      <dl className={styles.facts}>
        <div className={styles.fact}>
          <dt>{t('browse.security.license')}</dt>
          <dd>{data.license ?? t('browse.security.licenseMissing')}</dd>
        </div>
      </dl>
    </section>
  );
}

interface EntryProps {
  entry: SecurityEntry;
  onAdd?: ExceptionAdder;
}

function SecurityEntryView({ entry, onAdd }: EntryProps) {
  const t = useT();
  const { result, repository, coordinates } = entry;
  if (result.isError) {
    if (isUnavailable(result.error)) {
      return <EmptyState title={t('browse.security.noData')} />;
    }
    return <ErrorState error={result.error} onRetry={() => void result.refetch()} />;
  }
  if (result.data === undefined) {
    return <Skeleton height="200px" />;
  }
  return (
    <ComponentSecurityPanel
      repository={repository}
      coordinates={coordinates}
      data={result.data}
      onAdd={onAdd}
    />
  );
}

function prefill(target: ExceptionTarget): Partial<ExceptionForm> {
  return {
    rule: 'vulnerability',
    subject: 'advisory',
    policy: target.policy,
    advisory: target.advisory,
    format: target.coordinates.format,
    namespace: target.coordinates.namespace,
    name: target.coordinates.name,
    version: target.coordinates.version,
  };
}

export function SecurityTab({ entries }: SecurityTabProps) {
  const t = useT();
  const canCreate = useCan(EXCEPTION_CREATE);
  const invalidate = useInvalidateApi();
  const [target, setTarget] = useState<ExceptionTarget | null>(null);
  if (entries.length === 0) {
    return <EmptyState title={t('browse.security.noData')} />;
  }
  const close = (open: boolean) => {
    if (!open) {
      setTarget(null);
      void invalidate(`${V1}/components/`);
    }
  };
  return (
    <>
      <div className={styles.tab}>
        {entries.map((entry) => (
          <SecurityEntryView key={entry.id} entry={entry} onAdd={canCreate ? setTarget : undefined} />
        ))}
      </div>
      {canCreate && (
        <ExceptionDialog
          open={target !== null}
          onOpenChange={close}
          initial={target === null ? {} : prefill(target)}
          scoped
        />
      )}
    </>
  );
}
