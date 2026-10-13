import { Badge, Button } from '@radix-ui/themes';
import {
  CircleCheck,
  CircleX,
  Clock,
  Database,
  ExternalLink,
  FileBox,
  Ghost,
  HardDrive,
  KeyRound,
  Package,
  ScrollText,
  User,
  Users,
  Zap,
} from 'lucide-react';
import type { ReactNode } from 'react';
import { Link } from 'react-router';
import { INTERNAL, useApi, withQuery } from '../../../api';
import { useCan } from '../../../auth/state';
import { type Translate, useT } from '../../../i18n';
import { DateTime, FileSize, FormatLogo, formatName } from '../../../ui';
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

const KINDS: Record<string, string> = {
  created: 'repositories.audit.kinds.created',
  updated: 'repositories.audit.kinds.updated',
  deleted: 'repositories.audit.kinds.deleted',
};

export function eventLabel(t: Translate, kind: string): string {
  const key = KINDS[kind];
  return key ? t(key) : kind;
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
      <div className={styles.tableWrap}>
        <table className={styles.table}>
          <caption className="visually-hidden">{t('repositories.audit.label')}</caption>
          <thead>
            <tr>
              <th>{t('repositories.audit.time')}</th>
              <th>{t('repositories.audit.category')}</th>
              <th>{t('repositories.audit.event')}</th>
              <th>{t('repositories.audit.summary')}</th>
              <th>{t('repositories.audit.initiator')}</th>
            </tr>
          </thead>
          <tbody>
            {events.map((event) => (
              <tr key={event.id}>
                <td className={styles.muted}>
                  <DateTime value={event.at} />
                </td>
                <td>
                  <Badge color="purple" variant="soft" radius="small">
                    {t('repositories.audit.repositoryCategory')}
                  </Badge>
                </td>
                <td>{eventLabel(t, event.type)}</td>
                <td>
                  {t('repositories.audit.sentence', {
                    name: event.context,
                    event: eventLabel(t, event.type).toLowerCase(),
                    initiator: event.initiator,
                  })}
                </td>
                <td className={styles.muted}>{event.initiator}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
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

interface PrivilegeRow {
  name: string;
  description?: string;
  type: string;
  format?: string;
  repository?: string;
  actions?: string[];
}

interface RoleRow {
  id: string;
  privileges: string[];
  roles?: string[];
}

interface UserRow {
  userId: string;
  roles: string[];
}

export function repositoryPrivileges(
  privileges: readonly PrivilegeRow[],
  repository: UiRepository,
  formatWide: boolean,
): PrivilegeRow[] {
  return privileges
    .filter(
      (privilege) =>
        (privilege.type === 'repository-admin' || privilege.type === 'repository-view') &&
        (privilege.repository === repository.name ||
          (formatWide &&
            privilege.type === 'repository-view' &&
            privilege.repository === '*' &&
            privilege.format === repository.format)),
    )
    .sort((left, right) => left.name.localeCompare(right.name));
}

export function rolesWith(roles: readonly RoleRow[], privileges: ReadonlySet<string>): string[] {
  const byId = new Map(roles.map((role) => [role.id, role]));
  const grants = (id: string, seen: Set<string>): boolean => {
    const role = byId.get(id);
    if (!role || seen.has(id)) {
      return false;
    }
    seen.add(id);
    return (
      role.privileges.some((name) => privileges.has(name)) ||
      (role.roles ?? []).some((inner) => grants(inner, seen))
    );
  };
  return roles.filter((role) => grants(role.id, new Set())).map((role) => role.id);
}

export function AccessTab({
  repository,
  formatWide = true,
}: {
  repository: UiRepository;
  formatWide?: boolean;
}) {
  const t = useT();
  const canPrivileges = useCan('astrum:privileges:read');
  const canRoles = useCan('astrum:roles:read');
  const canUsers = useCan('astrum:users:read');
  const canAnonymous = useCan('astrum:settings:read');
  const privileges = useApi<PrivilegeRow[]>(canPrivileges ? '/service/rest/v1/security/privileges' : null);
  const roles = useApi<RoleRow[]>(canRoles ? '/service/rest/v1/security/roles' : null);
  const users = useApi<UserRow[]>(canUsers ? '/service/rest/v1/security/users' : null);
  const anonymous = useApi<{ enabled: boolean; userId: string }>(
    canAnonymous ? '/service/rest/v1/security/anonymous' : null,
  );
  const found = repositoryPrivileges(privileges.data ?? [], repository, formatWide);
  const names = new Set(found.map((privilege) => privilege.name));
  const roleIds = rolesWith(roles.data ?? [], names);
  const roleSet = new Set(roleIds);
  const userIds = (users.data ?? [])
    .filter((user) => user.roles.some((role) => roleSet.has(role)))
    .map((user) => user.userId);
  const anonymousOn =
    anonymous.data?.enabled === true &&
    (users.data ?? []).some(
      (user) => user.userId === anonymous.data?.userId && user.roles.some((role) => roleSet.has(role)),
    );
  return (
    <div className={styles.stack}>
      <Card
        icon={<KeyRound size={18} />}
        title={t('repositories.access.privileges')}
        count={t('repositories.access.privilegeCount', { count: found.length })}
      >
        {found.length === 0 ? (
          <p className={styles.text}>{t('repositories.access.noPrivileges')}</p>
        ) : (
          <ul className={styles.rows}>
            {found.map((privilege) => (
              <li key={privilege.name} className={styles.row}>
                <span>
                  <span className={styles.mono}>{privilege.name}</span>{' '}
                  {privilege.description && <span className={styles.muted}>({privilege.description})</span>}
                </span>
                <Badge variant="soft" radius="small" size="1">
                  {(privilege.actions ?? []).join(', ')}
                </Badge>
              </li>
            ))}
          </ul>
        )}
      </Card>
      <Card
        icon={<Users size={18} />}
        title={t('repositories.access.roles')}
        count={t('repositories.access.roleCount', { count: roleIds.length })}
      >
        {roleIds.length === 0 ? (
          <p className={styles.text}>{t('repositories.access.noRoles')}</p>
        ) : (
          <ul className={styles.rows}>
            {roleIds.map((id) => (
              <li key={id} className={styles.mono}>
                {id}
              </li>
            ))}
          </ul>
        )}
      </Card>
      <Card
        icon={<User size={18} />}
        title={t('repositories.access.users')}
        count={t('repositories.access.userCount', { count: userIds.length })}
      >
        {userIds.length === 0 ? (
          <p className={styles.text}>{t('repositories.access.noUsers')}</p>
        ) : (
          <ul className={styles.rows}>
            {userIds.map((id) => (
              <li key={id} className={styles.mono}>
                {id}
              </li>
            ))}
          </ul>
        )}
      </Card>
      <Card
        icon={<Ghost size={18} />}
        title={t('repositories.access.anonymous')}
        count={
          <span style={{ display: 'inline-flex', gap: 4, alignItems: 'center' }}>
            {anonymousOn ? (
              <CircleCheck size={12} aria-hidden="true" />
            ) : (
              <CircleX size={12} aria-hidden="true" />
            )}
            {anonymousOn ? t('repositories.access.enabled') : t('repositories.access.disabled')}
          </span>
        }
      >
        <p className={styles.text}>
          {anonymousOn ? t('repositories.access.anonymousYes') : t('repositories.access.anonymousNo')}
        </p>
      </Card>
    </div>
  );
}
