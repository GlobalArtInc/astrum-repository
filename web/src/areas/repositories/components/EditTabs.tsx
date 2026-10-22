import { Badge, Button, Table, Text } from '@radix-ui/themes';
import {
  CircleCheck,
  CircleX,
  Clock,
  Database,
  ExternalLink,
  FileBox,
  HardDrive,
  Package,
  ScrollText,
  Zap,
} from 'lucide-react';
import type { ReactNode } from 'react';
import { Link } from 'react-router';
import { INTERNAL, REST, useApi, withQuery } from '../../../api';
import { useCan } from '../../../auth/state';
import { useT } from '../../../i18n';
import { DateTime, FileSize, FormatLogo, formatName } from '../../../ui';
import { CATEGORY_COLORS, CATEGORY_LABELS, categoryOf, kindLabel, summaryOf } from '../../audit/events';
import { useUiBlobStores } from '../../blobstores/api';
import type { AuditEvent, UiRepository } from '../api';
import { Count } from './bits';
import { OnlineDot } from './RepositoryHeader';
import styles from './tabs.module.css';

export function Card({
  icon,
  title,
  count,
  action,
  plain,
  children,
}: {
  icon?: ReactNode;
  title: string;
  count?: ReactNode;
  action?: ReactNode;
  plain?: boolean;
  children?: ReactNode;
}) {
  return (
    <section className={styles.card} aria-label={title}>
      <div className={styles.cardHead} data-plain={plain || undefined}>
        <h3 className={styles.cardTitle}>
          {icon}
          {title}
          {count !== undefined && <span className={styles.count}>{count}</span>}
        </h3>
        {action}
      </div>
      {children}
    </section>
  );
}

function Item({
  term,
  upper,
  children,
  big,
}: {
  term: ReactNode;
  upper?: boolean;
  big?: boolean;
  children: ReactNode;
}) {
  return (
    <div>
      <dt className={styles.term} data-upper={upper || undefined}>
        {term}
      </dt>
      <dd className={styles.value} data-big={big || undefined}>
        {children}
      </dd>
    </div>
  );
}

function htmlIndex(name: string): string {
  return `${REST}/repository/browse/${encodeURIComponent(name)}/`;
}

export function SummaryTab({ repository, online }: { repository: UiRepository; online: boolean }) {
  const t = useT();
  const stores = useUiBlobStores();
  const store = stores.data?.find((candidate) => candidate.name === repository.blobStoreName);
  return (
    <div className={styles.stack}>
      <section className={styles.card} aria-label={t('repositories.summary.label')}>
        <dl className={styles.grid} style={{ margin: 0 }}>
          <Item term={t('repositories.summary.status')} upper>
            <OnlineDot online={online} />
            <span style={{ fontSize: 12 }}>
              {online ? t('repositories.status.online') : t('repositories.status.offline')}
            </span>
          </Item>
          <Item term={t('repositories.summary.format')} upper>
            <FormatLogo format={repository.format} />
            {formatName(repository.format)}
          </Item>
          <Item term={t('repositories.summary.type')} upper>
            <TypeChip type={repository.type} />
          </Item>
          <Item term={t('repositories.summary.url')} upper>
            <a
              className={`${styles.link} ${styles.ellipsis}`}
              href={repository.url}
              target="_blank"
              rel="noreferrer"
            >
              {repository.url}
            </a>
            <ExternalLink size={14} aria-hidden="true" />
          </Item>
          <Item term={t('repositories.summary.htmlIndex')} upper>
            <a
              className={`${styles.link} ${styles.ellipsis}`}
              href={htmlIndex(repository.name)}
              target="_blank"
              rel="noreferrer"
            >
              {htmlIndex(repository.name)}
            </a>
            <ExternalLink size={14} aria-hidden="true" />
          </Item>
          {repository.domain && (
            <Item term={t('repositories.summary.domain')} upper>
              <a
                className={`${styles.link} ${styles.ellipsis}`}
                href={repository.domain.url}
                target="_blank"
                rel="noreferrer"
              >
                {repository.domain.url}
              </a>
              <ExternalLink size={14} aria-hidden="true" />
            </Item>
          )}
          {repository.blobStoreName && (
            <Item term={t('repositories.summary.blobStore')} upper>
              <HardDrive size={14} aria-hidden="true" />
              {repository.blobStoreName}
            </Item>
          )}
        </dl>
      </section>
      {repository.blobStoreName && (
        <Card icon={<HardDrive size={18} />} title={t('repositories.summary.blobStoreCard')} plain>
          <dl className={styles.grid} data-columns="5" style={{ margin: 0 }}>
            <Item term={t('repositories.summary.name')}>{repository.blobStoreName}</Item>
            <Item term={t('repositories.summary.storeType')}>{store ? storeTypeName(store.type) : '—'}</Item>
            <Item term={t('repositories.summary.totalSize')}>
              {store ? <FileSize bytes={store.totalSize} /> : '—'}
            </Item>
            <Item term={t('repositories.summary.available')}>
              {store && store.availableSpace !== null ? <FileSize bytes={store.availableSpace} /> : '—'}
            </Item>
            <Item term={t('repositories.summary.blobCount')}>
              {store ? <Count value={store.blobCount} /> : '—'}
            </Item>
            <Item term={t('repositories.summary.storeStatus')}>
              {store?.unavailable ? (
                <>
                  <CircleX size={14} aria-hidden="true" />
                  {t('repositories.summary.unavailable')}
                </>
              ) : (
                <span className={styles.ok} style={{ display: 'inline-flex', gap: 6, alignItems: 'center' }}>
                  <CircleCheck size={14} aria-hidden="true" />
                  {t('repositories.summary.availableStatus')}
                </span>
              )}
            </Item>
          </dl>
        </Card>
      )}
      <Card icon={<Package size={18} />} title={t('repositories.summary.metrics')} plain>
        <dl className={styles.grid} data-columns="2" style={{ margin: 0 }}>
          <Item
            term={
              <span style={{ display: 'inline-flex', gap: 4, alignItems: 'center' }}>
                <Package size={12} aria-hidden="true" />
                {t('repositories.summary.components')}
              </span>
            }
            big
          >
            <Count value={repository.componentCount} />
          </Item>
          <Item
            term={
              <span style={{ display: 'inline-flex', gap: 4, alignItems: 'center' }}>
                <FileBox size={12} aria-hidden="true" />
                {t('repositories.summary.assets')}
              </span>
            }
            big
          >
            <Count value={repository.assetCount} />
          </Item>
        </dl>
      </Card>
    </div>
  );
}

export function storeTypeName(type: string): string {
  const lower = type.toLowerCase();
  return lower === 's3' ? 'S3' : lower.charAt(0).toUpperCase() + lower.slice(1);
}

const TYPE_LABELS = {
  hosted: 'ui.repositoryType.hosted',
  proxy: 'ui.repositoryType.proxy',
  group: 'ui.repositoryType.group',
} as const;

export function TypeChip({ type }: { type: UiRepository['type'] }) {
  const t = useT();
  return (
    <Badge color="green" variant="soft" radius="full">
      <Database size={12} aria-hidden="true" />
      {t(TYPE_LABELS[type])}
    </Badge>
  );
}

export function auditLogAddress(name: string): string {
  return withQuery('/audit', { repositoryName: name });
}

export function AuditCard({ name }: { name: string }) {
  const t = useT();
  const since = new Date(Date.now() - 30 * 24 * 3600 * 1000);
  since.setUTCHours(0, 0, 0, 0);
  const query = useApi<AuditEvent[]>(
    withQuery(`${INTERNAL}/audit`, { repository: name, limit: 100, since: since.toISOString() }),
  );
  const events = query.data ?? [];
  if (query.data && events.length === 0) {
    return (
      <section className={styles.card} aria-label={t('repositories.audit.title')}>
        <div className={styles.empty}>
          <ScrollText size={40} aria-hidden="true" />
          <p className={styles.emptyTitle}>{t('repositories.audit.empty')}</p>
          <p className={styles.text}>{t('repositories.audit.emptyText')}</p>
          <Button asChild variant="outline" highContrast>
            <Link to={auditLogAddress(name)}>
              {t('repositories.audit.openLog')}
              <ExternalLink size={14} aria-hidden="true" />
            </Link>
          </Button>
        </div>
      </section>
    );
  }
  return (
    <Card
      icon={<ScrollText size={18} />}
      title={t('repositories.audit.title')}
      count={t('repositories.audit.count', { count: events.length })}
      action={
        <Button asChild>
          <Link to={auditLogAddress(name)}>
            {t('repositories.audit.fullLog')}
            <ExternalLink size={14} aria-hidden="true" />
          </Link>
        </Button>
      }
    >
      <Table.Root variant="surface" size="2">
        <caption className="visually-hidden">{t('repositories.audit.label')}</caption>
        <Table.Header>
          <Table.Row>
            <Table.ColumnHeaderCell>{t('repositories.audit.time')}</Table.ColumnHeaderCell>
            <Table.ColumnHeaderCell>{t('repositories.audit.category')}</Table.ColumnHeaderCell>
            <Table.ColumnHeaderCell>{t('repositories.audit.event')}</Table.ColumnHeaderCell>
            <Table.ColumnHeaderCell>{t('repositories.audit.summary')}</Table.ColumnHeaderCell>
            <Table.ColumnHeaderCell>{t('repositories.audit.initiator')}</Table.ColumnHeaderCell>
          </Table.Row>
        </Table.Header>
        <Table.Body>
          {events.map((event) => {
            const category = categoryOf(event);
            return (
              <Table.Row key={event.id}>
                <Table.Cell className={styles.nowrap}>
                  <Text color="gray">
                    <DateTime value={event.at} />
                  </Text>
                </Table.Cell>
                <Table.Cell>
                  <Badge size="1" color={CATEGORY_COLORS[category]}>
                    {t(CATEGORY_LABELS[category])}
                  </Badge>
                </Table.Cell>
                <Table.Cell className={styles.nowrap}>
                  <Text weight="medium">{kindLabel(t, event)}</Text>
                </Table.Cell>
                <Table.Cell className={styles.summary}>{summaryOf(t, event)}</Table.Cell>
                <Table.Cell>
                  <Text color="gray">{event.initiator}</Text>
                </Table.Cell>
              </Table.Row>
            );
          })}
        </Table.Body>
      </Table.Root>
    </Card>
  );
}

interface RepositoryTask {
  id: string;
  name: string;
  typeName: string;
  enabled: boolean;
  nextRun: string | null;
  properties: Record<string, string>;
}

export function tasksOf(tasks: readonly RepositoryTask[], repository: string): RepositoryTask[] {
  const name = repository.toLowerCase();
  return tasks.filter((task) => (task.properties.repositoryName ?? '').toLowerCase() === name);
}

export function TasksTab({ repository }: { repository: string }) {
  const t = useT();
  const canRead = useCan('astrum:tasks:read');
  const tasks = useApi<RepositoryTask[]>(canRead ? `${INTERNAL}/ui/tasks` : null);
  const found = tasksOf(tasks.data ?? [], repository);
  return (
    <div className={styles.stack}>
      <Card
        icon={<Clock size={18} />}
        title={t('repositories.tasks.title')}
        count={t('repositories.tasks.count', { count: found.length })}
      >
        {found.length === 0 ? (
          <p className={styles.text}>{t('repositories.tasks.empty')}</p>
        ) : (
          <ul className={styles.rows}>
            {found.map((task) => (
              <li key={task.id} className={styles.row}>
                <span>
                  <Link to={`/admin/system/tasks/${encodeURIComponent(task.id)}`} className={styles.link}>
                    {task.name}
                  </Link>{' '}
                  <span className={styles.muted}>({task.typeName})</span>
                </span>
                <span className={styles.muted}>
                  {task.nextRun ? <DateTime value={task.nextRun} /> : t('repositories.tasks.notScheduled')}
                </span>
              </li>
            ))}
          </ul>
        )}
      </Card>
      <Card
        icon={<Zap size={18} />}
        title={t('repositories.tasks.capabilities')}
        count={t('repositories.tasks.capabilityCount', { count: 0 })}
      >
        <p className={styles.text}>{t('repositories.tasks.noCapabilities')}</p>
      </Card>
    </div>
  );
}
