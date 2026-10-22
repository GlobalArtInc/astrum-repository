import type { Translate } from '../../i18n';

export type Category = 'security' | 'repository' | 'configuration' | 'protection';

export const CATEGORIES: readonly Category[] = ['security', 'repository', 'configuration', 'protection'];

export const CATEGORY_LABELS: Record<Category, string> = {
  security: 'audit.category.security',
  repository: 'audit.category.repository',
  configuration: 'audit.category.configuration',
  protection: 'audit.category.protection',
};

export const CATEGORY_COLORS = {
  security: 'blue',
  repository: 'purple',
  configuration: 'gray',
  protection: 'orange',
} as const satisfies Record<Category, string>;

export const EVENT_TYPES = [
  'created',
  'updated',
  'deleted',
  'started',
  'finished',
  'failed',
  'login',
  'logout',
  'automatic-malware-removed',
] as const;

export const EVENT_TYPE_LABELS: Record<(typeof EVENT_TYPES)[number], string> = {
  created: 'audit.types.created',
  updated: 'audit.types.updated',
  deleted: 'audit.types.deleted',
  started: 'audit.types.started',
  finished: 'audit.types.finished',
  failed: 'audit.types.failed',
  login: 'audit.types.login',
  logout: 'audit.types.logout',
  'automatic-malware-removed': 'audit.types.malwareCleaned',
};

const ENTITIES: Record<string, { label: string; category: Category }> = {
  'security.user': { label: 'audit.entity.user', category: 'security' },
  'security.session': { label: 'audit.entity.user', category: 'security' },
  'security.role': { label: 'audit.entity.role', category: 'security' },
  'security.privilege': { label: 'audit.entity.privilege', category: 'security' },
  'security.ldap': { label: 'audit.entity.ldap', category: 'security' },
  'security.realm': { label: 'audit.entity.realms', category: 'security' },
  'security.anonymous': { label: 'audit.entity.anonymous', category: 'security' },
  'security.jwt': { label: 'audit.entity.jwt', category: 'security' },
  repository: { label: 'audit.entity.repository', category: 'repository' },
  'repository.asset': { label: 'audit.entity.asset', category: 'repository' },
  asset: { label: 'audit.entity.asset', category: 'repository' },
  'repository.component': { label: 'audit.entity.component', category: 'repository' },
  component: { label: 'audit.entity.component', category: 'repository' },
  blobstore: { label: 'audit.entity.blobStore', category: 'repository' },
  ContentSelector: { label: 'audit.entity.contentSelector', category: 'configuration' },
  'security.contentselector': { label: 'audit.entity.contentSelector', category: 'configuration' },
  cleanupPolicy: { label: 'audit.entity.cleanupPolicy', category: 'configuration' },
  cleanup: { label: 'audit.entity.cleanupPolicy', category: 'configuration' },
  tasks: { label: 'audit.entity.task', category: 'configuration' },
};

const KIND_LABELS: Record<string, string> = {
  activated: 'audit.kind.activated',
  'automatic-malware-removed': 'audit.kind.malwareRemoved',
  canceled: 'audit.kind.canceled',
  changed: 'audit.kind.updated',
  created: 'audit.kind.created',
  deleted: 'audit.kind.deleted',
  failed: 'audit.kind.failed',
  finished: 'audit.kind.finished',
  login: 'audit.kind.login',
  logout: 'audit.kind.logout',
  passivated: 'audit.kind.passivated',
  purged: 'audit.kind.purged',
  scheduled: 'audit.kind.scheduled',
  started: 'audit.kind.started',
  stopped: 'audit.kind.stopped',
  updated: 'audit.kind.updated',
  'updated-attribute': 'audit.kind.attributesUpdated',
  'updated-downloaded': 'audit.kind.downloaded',
  'updated-kind': 'audit.kind.kindChanged',
  'updated-uploaded': 'audit.kind.uploaded',
  'user-session-invalidation': 'audit.kind.sessionInvalidated',
};

const ACTIONS: Record<string, string> = {
  activated: 'audit.action.activated',
  canceled: 'audit.action.canceled',
  changed: 'audit.action.updated',
  created: 'audit.action.created',
  deleted: 'audit.action.deleted',
  failed: 'audit.action.failed',
  finished: 'audit.action.finished',
  login: 'audit.action.login',
  logout: 'audit.action.logout',
  passivated: 'audit.action.passivated',
  purged: 'audit.action.purged',
  scheduled: 'audit.action.scheduled',
  started: 'audit.action.started',
  stopped: 'audit.action.stopped',
  updated: 'audit.action.updated',
  'updated-attribute': 'audit.action.attributesUpdated',
  'updated-downloaded': 'audit.action.downloaded',
  'updated-kind': 'audit.action.kindChanged',
  'updated-uploaded': 'audit.action.uploaded',
  'user-session-invalidation': 'audit.action.sessionInvalidated',
};

export interface AuditEvent {
  id: number;
  domain: string;
  type: string;
  context: string | null;
  timestamp: string;
  initiator: string;
  nodeId?: string | null;
  attributes: Record<string, unknown> | null;
}

export function shownKind(event: Pick<AuditEvent, 'domain' | 'type'>): string {
  if (event.domain === 'security.session') {
    if (event.type === 'created') {
      return 'login';
    }
    if (event.type === 'deleted') {
      return 'logout';
    }
  }
  return event.type;
}

function isSignIn(kind: string): boolean {
  return kind === 'login' || kind === 'logout';
}

export function categoryOf(event: Pick<AuditEvent, 'domain' | 'type'>): Category {
  if (isSignIn(shownKind(event))) {
    return 'security';
  }
  return ENTITIES[event.domain]?.category ?? 'configuration';
}

function words(kind: string): string[] {
  return kind.split('-').filter((word) => word !== '');
}

export function kindLabel(t: Translate, event: Pick<AuditEvent, 'domain' | 'type'>): string {
  const kind = shownKind(event);
  const key = KIND_LABELS[kind];
  if (key) {
    return t(key);
  }
  return words(kind)
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
    .join(' ');
}

function text(value: unknown): string | null {
  return typeof value === 'string' && value !== '' ? value : null;
}

export function summaryOf(
  t: Translate,
  event: Pick<AuditEvent, 'domain' | 'type' | 'context' | 'initiator' | 'attributes'>,
): string {
  const kind = shownKind(event);
  const context = text(event.context);
  const attributes = event.attributes ?? {};
  if (isSignIn(kind)) {
    const name = context ?? text(attributes.principal) ?? t('audit.summary.unknown');
    return t('audit.summary.session', {
      name,
      action: kind === 'login' ? t('audit.summary.login') : t('audit.summary.logout'),
    });
  }
  if (kind === 'automatic-malware-removed') {
    const repository = text(attributes.repository);
    const name = context ?? t('audit.summary.unknown');
    return repository
      ? t('audit.summary.malwareInRepository', { name, repository })
      : t('audit.summary.malware', { name });
  }
  const entity = ENTITIES[event.domain];
  const action = ACTIONS[kind];
  return t('audit.summary.event', {
    entity: entity ? t(entity.label) : t('audit.entity.event'),
    name: context ?? t('audit.summary.unknown'),
    action: action ? t(action) : words(kind).join(' '),
    initiator: event.initiator,
  });
}
