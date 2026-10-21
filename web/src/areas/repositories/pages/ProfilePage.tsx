import { Badge, Button, IconButton } from '@radix-ui/themes';
import {
  ChevronRight,
  CircleX,
  Clock,
  Database,
  ExternalLink,
  FileBox,
  FolderTree,
  Ghost,
  HardDrive,
  Info,
  KeyRound,
  RefreshCw,
  Shield,
  User,
  Users,
  Zap,
} from 'lucide-react';
import type { ReactNode } from 'react';
import { Link, useNavigate, useParams } from 'react-router';
import { path, useApi, useInvalidateApi } from '../../../api';
import { useCan, useSession } from '../../../auth/state';
import { useT } from '../../../i18n';
import { CopyButton, EmptyState, ErrorState, FileSize, formatName, Page } from '../../../ui';
import { storeAddress, storeType, useUiBlobStores } from '../../blobstores/api';
import {
  documentPath,
  editAddress,
  LIST_ADDRESS,
  UI_REPOSITORIES,
  type UiRepository,
  useUiRepositories,
} from '../api';
import { Count } from '../components/bits';
import { AuditCard, Card, storeTypeName, TypeChip } from '../components/EditTabs';
import { ProfileTabs } from '../components/ProfileTabs';
import styles from '../components/profile.module.css';
import { RepositoryBadges, RepositoryButtons } from '../components/RepositoryHeader';
import tabs from '../components/tabs.module.css';
import { useRepositoryActions } from '../components/useRepositoryActions';
import { Logo } from '../components/Wizard';
import { type PrivilegeRow, type RoleRow, repositoryPrivileges, rolesWith } from '../model/access';
import type { RepositoryDocument } from '../model/document';
import { cleanupPolicies, fromDocument } from '../model/form';
import { apiFormat } from '../model/spec';
import { findRepository, Loading } from './RepositoryPage';

export function profileAddress(name: string, tab?: string): string {
  const base = path`/admin/repository/repositories/${name}/profile`;
  return tab ? `${base}?tab=${encodeURIComponent(tab)}` : base;
}

function Field({ term, children }: { term: string; children: ReactNode }) {
  return (
    <div>
      <dt className={tabs.term} data-upper>
        {term}
      </dt>
      <dd className={tabs.value}>{children}</dd>
    </div>
  );
}

function OnOff({ on }: { on: boolean }) {
  const t = useT();
  return (
    <Badge color={on ? 'green' : 'gray'} variant="soft" radius="small">
      {on ? t('repositories.profile.enabled') : t('repositories.profile.disabled')}
    </Badge>
  );
}

function Configure({ to }: { to: string }) {
  const t = useT();
  return (
    <Link to={to} className={styles.configure}>
      {t('repositories.profile.configure')}
      <ChevronRight size={14} aria-hidden="true" />
    </Link>
  );
}

function RepositoryTab({
  repository,
  document,
}: {
  repository: UiRepository;
  document: RepositoryDocument | undefined;
}) {
  const t = useT();
  const stores = useUiBlobStores();
  const store = stores.data?.find((candidate) => candidate.name === repository.blobStoreName);
  const values = document ? fromDocument(document) : null;
  const policies = document ? cleanupPolicies(document) : [];
  const kind = store ? storeType(store.type) : null;
  return (
    <div className={tabs.stack}>
      <Card
        icon={<Database size={18} />}
        title={t('repositories.profile.configuration')}
        action={<Configure to={editAddress(repository.name)} />}
      >
        <h4 className={styles.subTitle}>{t('repositories.profile.general')}</h4>
        <dl className={tabs.grid} data-columns="2" style={{ margin: 0 }}>
          <Field term={t('repositories.profile.name')}>{repository.name}</Field>
          <Field term={t('repositories.profile.format')}>
            <Logo format={repository.format} size={16} />
            {formatName(repository.format)}
          </Field>
          <Field term={t('repositories.profile.type')}>
            <TypeChip type={repository.type} />
          </Field>
          <Field term={t('repositories.profile.online')}>
            <OnOff on={repository.online} />
          </Field>
          <Field term={t('repositories.profile.url')}>
            <span className={tabs.mono}>{repository.url}</span>
          </Field>
        </dl>
        {values && repository.type === 'proxy' && (
          <>
            <h4 className={styles.subTitle}>{t('repositories.profile.proxy')}</h4>
            <dl className={tabs.grid} data-columns="2" style={{ margin: 0 }}>
              <Field term={t('repositories.profile.remote')}>
                <span className={tabs.mono}>{values.remoteUrl}</span>
              </Field>
              <Field term={t('repositories.profile.contentMaxAge')}>{values.contentMaxAge ?? '—'}</Field>
              <Field term={t('repositories.profile.metadataMaxAge')}>{values.metadataMaxAge ?? '—'}</Field>
              <Field term={t('repositories.profile.blocked')}>
                <OnOff on={values.blocked} />
              </Field>
              <Field term={t('repositories.profile.autoBlock')}>
                <OnOff on={values.autoBlock} />
              </Field>
            </dl>
          </>
        )}
        {values && repository.type === 'group' && (
          <>
            <h4 className={styles.subTitle}>{t('repositories.profile.group')}</h4>
            <dl className={tabs.grid} data-columns="2" style={{ margin: 0 }}>
              <Field term={t('repositories.profile.members')}>{values.memberNames.join(', ') || '—'}</Field>
            </dl>
          </>
        )}
        <h4 className={styles.subTitle}>{t('repositories.profile.storage')}</h4>
        <dl className={tabs.grid} data-columns="2" style={{ margin: 0 }}>
          <Field term={t('repositories.profile.blobStore')}>{repository.blobStoreName ?? '—'}</Field>
          <Field term={t('repositories.profile.strict')}>
            <OnOff on={values?.strictContentTypeValidation ?? false} />
          </Field>
          {values && repository.type === 'hosted' && (
            <Field term={t('repositories.profile.writePolicy')}>{values.writePolicy || '—'}</Field>
          )}
          <Field term={t('repositories.profile.cleanup')}>
            <span className={tabs.muted} style={{ fontSize: 14 }}>
              {policies.length > 0 ? policies.join(', ') : t('repositories.profile.none')}
            </span>
          </Field>
        </dl>
      </Card>
      {repository.blobStoreName && (
        <Card
          icon={<HardDrive size={18} />}
          title={t('repositories.profile.blobStoreTitle', { name: repository.blobStoreName })}
          action={kind ? <Configure to={storeAddress(kind, repository.blobStoreName)} /> : undefined}
        >
          {store ? (
            <dl className={tabs.grid} style={{ margin: 0, gridTemplateColumns: 'repeat(3, minmax(0, 1fr))' }}>
              <Field term={t('repositories.profile.name')}>{store.name}</Field>
              <Field term={t('repositories.profile.type')}>{storeTypeName(store.type)}</Field>
              <Field term={t('repositories.profile.status')}>
                <Badge color={store.unavailable ? 'red' : 'green'} variant="soft" radius="small">
                  {store.unavailable
                    ? t('repositories.profile.unavailable')
                    : t('repositories.profile.available')}
                </Badge>
              </Field>
              <Field term={t('repositories.profile.totalSize')}>
                <FileSize bytes={store.totalSize} />
              </Field>
              <Field term={t('repositories.profile.free')}>
                {store.availableSpace === null ? '—' : <FileSize bytes={store.availableSpace} />}
              </Field>
              <Field term={t('repositories.profile.blobCount')}>
                <Count value={store.blobCount} />
              </Field>
              <Field term={t('repositories.profile.location')}>
                {store.path ?? store.bucket ?? store.name}
              </Field>
            </dl>
          ) : (
            <p className={tabs.text}>{t('repositories.profile.storeMissing')}</p>
          )}
        </Card>
      )}
    </div>
  );
}

function MembershipTab({ repository }: { repository: UiRepository }) {
  const t = useT();
  return (
    <div className={tabs.stack}>
      <div className={styles.callout}>
        <Info size={16} aria-hidden="true" />
        <span>
          {t.rich('repositories.profile.membershipText', { name: <strong>{repository.name}</strong> })}
        </span>
      </div>
      {repository.memberOf.length === 0 ? (
        <p className={tabs.text}>{t('repositories.profile.noGroups')}</p>
      ) : (
        <div className={styles.groups}>
          <span className={styles.groupsHead}>{t('repositories.profile.groupRepository')}</span>
          {repository.memberOf.map((group) => (
            <Link key={group} to={profileAddress(group)} className={styles.groupRow}>
              <FolderTree size={16} aria-hidden="true" />
              <span className={styles.grow}>{group}</span>
              <ExternalLink size={14} aria-hidden="true" />
              <ChevronRight size={14} aria-hidden="true" />
            </Link>
          ))}
        </div>
      )}
    </div>
  );
}

function UsageTab({ repository }: { repository: UiRepository }) {
  const t = useT();
  const stores = useUiBlobStores();
  const store = stores.data?.find((candidate) => candidate.name === repository.blobStoreName);
  return (
    <div className={tabs.stack}>
      <div className={styles.stats}>
        <div className={styles.stat}>
          <HardDrive size={20} aria-hidden="true" />
          <span className={styles.statValue}>
            {repository.totalSize === null ? '—' : <FileSize bytes={repository.totalSize} />}
          </span>
          <span className={styles.statLabel}>{t('repositories.profile.size')}</span>
        </div>
        <div className={styles.stat}>
          <FileBox size={20} aria-hidden="true" />
          <span className={styles.statValue}>
            <Count value={repository.assetCount} />
          </span>
          <span className={styles.statLabel}>{t('repositories.profile.assets')}</span>
        </div>
        <div className={styles.stat}>
          <Database size={20} aria-hidden="true" />
          <span className={styles.statValue}>{repository.blobStoreName ?? '—'}</span>
          <span className={styles.statLabel}>
            {store && store.availableSpace !== null
              ? t.rich('repositories.profile.freeSpace', { size: <FileSize bytes={store.availableSpace} /> })
              : ''}
          </span>
        </div>
      </div>
      <Card title={t('repositories.profile.configuration')}>
        <dl className={tabs.grid} data-columns="2" style={{ margin: 0 }}>
          <div>
            <dt className={tabs.term}>{t('repositories.profile.format')}</dt>
            <dd className={tabs.value}>{repository.format}</dd>
          </div>
          <div>
            <dt className={tabs.term}>{t('repositories.profile.type')}</dt>
            <dd className={tabs.value} style={{ textTransform: 'capitalize' }}>
              {repository.type}
            </dd>
          </div>
          <div style={{ gridColumn: '1 / -1' }}>
            <dt className={tabs.term}>{t('repositories.profile.repositoryUrl')}</dt>
            <dd className={tabs.value} style={{ justifyContent: 'space-between' }}>
              <span className={tabs.mono}>{repository.url}</span>
              <CopyButton text={repository.url} label={t('repositories.actions.copyUrl')} />
            </dd>
          </div>
          <div>
            <dt className={tabs.term}>{t('repositories.profile.status')}</dt>
            <dd className={tabs.value}>
              <Badge color={repository.online ? 'green' : 'gray'} variant="soft" radius="small">
                {repository.online ? t('repositories.status.online') : t('repositories.status.offline')}
              </Badge>
            </dd>
          </div>
          <div>
            <dt className={tabs.term}>{t('repositories.profile.blobStore')}</dt>
            <dd className={tabs.value}>{repository.blobStoreName ?? '—'}</dd>
          </div>
        </dl>
      </Card>
    </div>
  );
}

function Plain({
  icon,
  title,
  to,
  children,
}: {
  icon: ReactNode;
  title: string;
  to?: string;
  children: ReactNode;
}) {
  const t = useT();
  return (
    <section className={styles.plainSection} aria-label={title}>
      <div className={styles.plainHead}>
        <h3 className={styles.plainTitle}>
          {icon}
          {title}
        </h3>
        {to && (
          <Link to={to} className={styles.configure}>
            {t('repositories.profile.show')}
            <ChevronRight size={14} aria-hidden="true" />
          </Link>
        )}
      </div>
      {children}
    </section>
  );
}

function SecurityTab({ repository }: { repository: UiRepository }) {
  const t = useT();
  const canPrivileges = useCan('astrum:privileges:read');
  const canRoles = useCan('astrum:roles:read');
  const canUsers = useCan('astrum:users:read');
  const canAnonymous = useCan('astrum:settings:read');
  const privileges = useApi<PrivilegeRow[]>(canPrivileges ? '/service/rest/v1/security/privileges' : null);
  const roles = useApi<RoleRow[]>(canRoles ? '/service/rest/v1/security/roles' : null);
  const users = useApi<Array<{ userId: string; roles: string[] }>>(
    canUsers ? '/service/rest/v1/security/users' : null,
  );
  const anonymous = useApi<{ enabled: boolean }>(canAnonymous ? '/service/rest/v1/security/anonymous' : null);
  const found = repositoryPrivileges(privileges.data ?? [], repository, false);
  const roleIds = rolesWith(roles.data ?? [], new Set(found.map((privilege) => privilege.name)));
  const userIds = (users.data ?? [])
    .filter((user) => user.roles.some((role) => roleIds.includes(role)))
    .map((user) => user.userId);
  return (
    <div className={tabs.stack} style={{ gap: 24 }}>
      <Plain
        icon={<KeyRound size={18} />}
        title={t('repositories.profile.privileges')}
        to="/admin/security/privileges"
      >
        {found.length === 0 ? (
          <p className={tabs.text}>{t('repositories.profile.noPrivileges')}</p>
        ) : (
          found.map((privilege) => (
            <div key={privilege.name} className={styles.privilege}>
              <span>
                <span className={styles.privilegeName}>{privilege.name}</span>{' '}
                {privilege.description && <span className={tabs.muted}>({privilege.description})</span>}
              </span>
              <Badge variant="soft" radius="small" size="1">
                {(privilege.actions ?? []).join(', ')}
              </Badge>
            </div>
          ))
        )}
      </Plain>
      <Plain icon={<Users size={18} />} title={t('repositories.access.roles')} to="/admin/security/roles">
        {roleIds.length === 0 ? (
          <p className={tabs.text}>{t('repositories.profile.noRoles')}</p>
        ) : (
          roleIds.map((id) => (
            <span key={id} className={tabs.mono}>
              {id}
            </span>
          ))
        )}
      </Plain>
      <Plain icon={<User size={18} />} title={t('repositories.access.users')} to="/admin/security/users">
        {userIds.length === 0 ? (
          <p className={tabs.text}>{t('repositories.profile.noUsers')}</p>
        ) : (
          userIds.map((id) => (
            <span key={id} className={tabs.mono}>
              {id}
            </span>
          ))
        )}
      </Plain>
      <Plain
        icon={<Ghost size={18} />}
        title={t('repositories.access.anonymous')}
        to="/admin/security/anonymous"
      >
        <div className={styles.featureRow} style={{ justifyContent: 'flex-start', gap: 16 }}>
          <span className={tabs.muted} style={{ fontSize: 14 }}>
            {t('repositories.profile.status')}
          </span>
          <OnOff on={anonymous.data?.enabled === true} />
        </div>
      </Plain>
    </div>
  );
}

function SystemTab() {
  const t = useT();
  return (
    <div className={tabs.stack} style={{ gap: 24 }}>
      <Plain icon={<Clock size={18} />} title={t('repositories.tasks.title')} to="/settings">
        <p className={tabs.text}>{t('repositories.profile.noTasks')}</p>
      </Plain>
      <Plain icon={<Zap size={18} />} title={t('repositories.tasks.capabilities')} to="/settings">
        <p className={tabs.text}>{t('repositories.profile.noCapabilities')}</p>
      </Plain>
    </div>
  );
}

function InstanceTab() {
  const t = useT();
  return (
    <div className={styles.instance}>
      <Card icon={<Shield size={18} />} title={t('repositories.profile.iqServer')}>
        <div className={styles.featureRow}>
          <span>{t('repositories.profile.status')}</span>
          <Badge color="red" variant="soft" radius="small">
            <CircleX size={12} aria-hidden="true" />
            {t('repositories.profile.disconnected')}
          </Badge>
        </div>
        <div className={styles.featureRow}>
          <span>{t('repositories.profile.url')}</span>
          <span className={tabs.mono}>{t('repositories.profile.notConfigured')}</span>
        </div>
        <span className={tabs.cardTitle} style={{ fontSize: 14, fontWeight: 500 }}>
          {t('repositories.profile.licenseFeatures')}
        </span>
        <div className={styles.features}>
          {['Lifecycle', 'Firewall'].map((feature) => (
            <div key={feature} className={styles.feature}>
              {feature}
              <span className={styles.featureValue}>
                <CircleX size={12} aria-hidden="true" />
                {t('repositories.profile.notAvailable')}
              </span>
            </div>
          ))}
        </div>
      </Card>
      <Card icon={<Zap size={18} />} title={t('repositories.profile.globalCapabilities')}>
        <div className={styles.featureRow}>
          <strong style={{ fontWeight: 500 }}>{t('repositories.profile.healthCheck')}</strong>
          <Badge color="gray" variant="soft" radius="small">
            {t('repositories.profile.notConfigured')}
          </Badge>
        </div>
        <p className={tabs.text} style={{ fontSize: 12 }}>
          {t('repositories.profile.healthCheckText')}
        </p>
        <div className={styles.featureRow}>
          <strong style={{ fontWeight: 500 }}>{t('repositories.profile.firewall')}</strong>
          <Badge color="gray" variant="soft" radius="small">
            {t('repositories.profile.licenseNeeded')}
          </Badge>
        </div>
        <p className={tabs.text} style={{ fontSize: 12 }}>
          {t('repositories.profile.firewallText')}
        </p>
        <p className={styles.note}>
          <Info size={12} aria-hidden="true" />
          {t('repositories.profile.instanceNote')}
        </p>
      </Card>
    </div>
  );
}

export default function ProfilePage() {
  const t = useT();
  const navigate = useNavigate();
  const invalidate = useInvalidateApi();
  const { name = '' } = useParams();
  const { state, can } = useSession();
  const list = useUiRepositories();
  const repository = findRepository(list.data, name);
  const visible = repository?.permissions.admin === true;
  const docPath =
    repository && visible
      ? documentPath(apiFormat(repository.format, state.formats), repository.type, repository.name)
      : null;
  const document = useApi<RepositoryDocument>(docPath);
  const actions = useRepositoryActions({ onDeleted: () => navigate(LIST_ADDRESS, { replace: true }) });
  const crumbs = [
    { label: t('repositories.crumbs.dashboard'), to: '/' },
    { label: repository?.name ?? name },
  ];

  if (list.isPending) {
    return (
      <Page title={name} breadcrumbs={crumbs}>
        <Loading />
      </Page>
    );
  }
  if (list.error) {
    return (
      <Page title={name} breadcrumbs={crumbs}>
        <ErrorState error={list.error} onRetry={() => void list.refetch()} />
      </Page>
    );
  }
  if (!repository || !visible) {
    return (
      <Page title={t('repositories.edit.notFoundTitle')} breadcrumbs={crumbs}>
        <EmptyState
          title={t('repositories.edit.notFoundTitle')}
          description={t('repositories.edit.notFound', { name })}
          action={
            <Button asChild variant="soft">
              <Link to={LIST_ADDRESS}>{t('repositories.edit.backToList')}</Link>
            </Button>
          }
        />
      </Page>
    );
  }
  const online = document.data ? document.data.online !== false : repository.online;
  const current = { ...repository, online };

  return (
    <Page
      title={repository.name}
      breadcrumbs={crumbs}
      description={
        <>
          <RepositoryBadges repository={repository} online={online} />
          <span className={styles.urlLine}>
            {t('repositories.profile.urlLabel')}
            <code className={styles.code}>{repository.url}</code>
          </span>
        </>
      }
      actions={
        <RepositoryButtons
          repository={repository}
          online={online}
          tone="accent"
          onRebuild={() => void actions.maintain(repository, 'rebuild-index')}
          onInvalidate={() => void actions.maintain(repository, 'invalidate-cache')}
          onToggleOnline={() => void actions.toggleOnline(current)}
          onDelete={() => actions.remove(repository)}
          extra={
            <IconButton
              variant="ghost"
              color="gray"
              aria-label={t('repositories.profile.refresh')}
              onClick={() => {
                void invalidate(UI_REPOSITORIES);
                if (docPath) {
                  void invalidate(docPath);
                }
              }}
            >
              <RefreshCw size={16} aria-hidden="true" />
            </IconButton>
          }
        />
      }
    >
      <div style={{ height: 24 }} />
      <ProfileTabs
        defaultTab="repository"
        tabs={[
          {
            value: 'repository',
            label: t('repositories.profile.tabs.repository'),
            content: <RepositoryTab repository={current} document={document.data} />,
          },
          {
            value: 'membership',
            label: t('repositories.profile.tabs.membership'),
            content: <MembershipTab repository={repository} />,
          },
          {
            value: 'usage',
            label: t('repositories.profile.tabs.usage'),
            content: <UsageTab repository={current} />,
          },
          ...(can('astrum:audit:read')
            ? [
                {
                  value: 'audit',
                  label: t('repositories.profile.tabs.audit'),
                  content: <AuditCard name={repository.name} />,
                },
              ]
            : []),
          {
            value: 'security',
            label: t('repositories.profile.tabs.security'),
            content: <SecurityTab repository={repository} />,
          },
          { value: 'system', label: t('repositories.profile.tabs.system'), content: <SystemTab /> },
          {
            value: 'instance-config',
            label: t('repositories.profile.tabs.instance'),
            content: <InstanceTab />,
          },
        ]}
      />
      {actions.dialog}
    </Page>
  );
}
