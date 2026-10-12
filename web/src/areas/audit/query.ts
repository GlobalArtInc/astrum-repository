import { withQuery } from '../../api';

export const AUDIT = '/service/rest/internal/audit';
export const AUDIT_FACETS = '/service/rest/internal/ui/audit/facets';
export const PAGE_SIZE = 50;

export interface AuditEvent {
  id: number;
  at: string;
  domain: string;
  type: string;
  context: string;
  initiator: string;
  attributes: Record<string, unknown> | null;
}

export interface AuditFacets {
  domains: string[];
  types: string[];
  initiators: string[];
}

export interface AuditFilters {
  q: string;
  domain: string;
  type: string;
  initiator: string;
  repository: string;
  since: string;
  until: string;
}

export const EMPTY_FILTERS: AuditFilters = {
  q: '',
  domain: '',
  type: '',
  initiator: '',
  repository: '',
  since: '',
  until: '',
};

const DATE = /^\d{4}-\d{2}-\d{2}$/;

export function readFilters(params: URLSearchParams): AuditFilters {
  const date = (value: string | null) => (value && DATE.test(value) ? value : '');
  return {
    q: params.get('q') ?? '',
    domain: params.get('domain') ?? '',
    type: params.get('type') ?? '',
    initiator: params.get('initiator') ?? '',
    repository: params.get('repositoryName') ?? '',
    since: date(params.get('since')),
    until: date(params.get('until')),
  };
}

export function writeFilters(filters: AuditFilters): URLSearchParams {
  const params = new URLSearchParams();
  const entries: Array<[string, string]> = [
    ['q', filters.q],
    ['domain', filters.domain],
    ['type', filters.type],
    ['initiator', filters.initiator],
    ['repositoryName', filters.repository],
    ['since', filters.since],
    ['until', filters.until],
  ];
  for (const [key, value] of entries) {
    if (value.trim() !== '') {
      params.set(key, value.trim());
    }
  }
  return params;
}

export function isFiltered(filters: AuditFilters): boolean {
  return Object.values(filters).some((value) => value.trim() !== '');
}

export function auditPath(filters: AuditFilters, before?: number | null): string {
  return withQuery(AUDIT, {
    limit: PAGE_SIZE,
    q: filters.q.trim(),
    domain: filters.domain,
    type: filters.type,
    initiator: filters.initiator,
    repository: filters.repository,
    since: filters.since,
    until: filters.until,
    before: before ?? undefined,
  });
}

export function nextBefore(page: readonly AuditEvent[]): number | undefined {
  if (page.length < PAGE_SIZE) {
    return undefined;
  }
  return page[page.length - 1]?.id;
}

export function attributeEntries(attributes: AuditEvent['attributes']): Array<[string, string]> {
  if (!attributes || typeof attributes !== 'object') {
    return [];
  }
  return Object.entries(attributes).map(([key, value]) => [
    key,
    typeof value === 'string' ? value : JSON.stringify(value),
  ]);
}

export const DOMAIN_LABELS: Record<string, string> = {
  repository: 'audit.domain.repository',
  blobstore: 'audit.domain.blobstore',
  component: 'audit.domain.component',
  asset: 'audit.domain.asset',
  'security.user': 'audit.domain.user',
  'security.role': 'audit.domain.role',
  'security.privilege': 'audit.domain.privilege',
  'security.realm': 'audit.domain.realm',
  'security.anonymous': 'audit.domain.anonymous',
  'security.contentselector': 'audit.domain.contentSelector',
  'security.ssrf': 'audit.domain.ssrf',
  'security.session': 'audit.domain.session',
  'security.accesstoken': 'audit.domain.accessToken',
  'security.npmtoken': 'audit.domain.npmToken',
};

export const TYPE_LABELS: Record<string, string> = {
  created: 'audit.type.created',
  updated: 'audit.type.updated',
  deleted: 'audit.type.deleted',
};

export function knownDomains(seen: readonly string[] = []): string[] {
  return [...new Set([...Object.keys(DOMAIN_LABELS), ...seen])];
}

export function knownTypes(seen: readonly string[] = []): string[] {
  return [...new Set([...Object.keys(TYPE_LABELS), ...seen])];
}
