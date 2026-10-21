import type { UiRepository } from '../api';

export interface PrivilegeRow {
  name: string;
  description?: string;
  type: string;
  format?: string;
  repository?: string;
  actions?: string[];
}

export interface RoleRow {
  id: string;
  name?: string;
  description?: string;
  source?: string;
  readOnly?: boolean;
  privileges: string[];
  roles?: string[];
}

export interface UserRow {
  userId: string;
  firstName?: string | null;
  lastName?: string | null;
  emailAddress?: string | null;
  source?: string;
  status?: string;
  readOnly?: boolean;
  roles: string[];
}

export type Level = 'read' | 'write' | 'full';
export type RoleLevel = Level | 'custom' | 'none';

export const LEVELS: readonly Level[] = ['read', 'write', 'full'];

const LEVEL_ACTIONS: Record<Level, readonly string[]> = {
  read: ['browse', 'read'],
  write: ['browse', 'read', 'add', 'edit'],
  full: ['browse', 'read', 'add', 'edit', 'delete'],
};

export const ANONYMOUS_ROLE = 'anonymous-reader';

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
            (privilege.format === repository.format || privilege.format === '*'))),
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

export function ownPrivileges(
  privileges: readonly PrivilegeRow[],
  repository: UiRepository,
): Map<string, string> {
  const byAction = new Map<string, string>();
  for (const privilege of privileges) {
    if (privilege.type !== 'repository-view' || privilege.repository !== repository.name) {
      continue;
    }
    const [action, ...rest] = (privilege.actions ?? []).map((name) => name.toLowerCase());
    if (action !== undefined && action !== 'all' && action !== '*' && rest.length === 0) {
      byAction.set(action, privilege.name);
    }
  }
  return byAction;
}

export function levelOf(role: RoleRow, own: ReadonlyMap<string, string>): RoleLevel {
  const held = new Set(
    [...own.entries()].filter(([, name]) => role.privileges.includes(name)).map(([action]) => action),
  );
  if (held.size === 0) {
    return 'none';
  }
  const exact = [...LEVELS].reverse().find((level) => {
    const actions = LEVEL_ACTIONS[level];
    return actions.length === held.size && actions.every((action) => held.has(action));
  });
  return exact ?? 'custom';
}

export function canSetLevel(own: ReadonlyMap<string, string>, level: Level): boolean {
  return LEVEL_ACTIONS[level].every((action) => own.has(action));
}

export function withLevel(role: RoleRow, own: ReadonlyMap<string, string>, level: Level | 'none'): string[] {
  const owned = new Set(own.values());
  const kept = role.privileges.filter((name) => !owned.has(name));
  if (level === 'none') {
    return kept;
  }
  const added = LEVEL_ACTIONS[level].map((action) => own.get(action)).filter((name) => name !== undefined);
  return [...kept, ...added];
}

export function grantsDirectly(role: RoleRow, own: ReadonlyMap<string, string>): boolean {
  const names = new Set(own.values());
  return role.privileges.some((name) => names.has(name));
}

export interface AccessPaths {
  direct: boolean;
  wide: boolean;
  nested: boolean;
}

export function accessPaths(
  role: RoleRow,
  roles: readonly RoleRow[],
  own: ReadonlyMap<string, string>,
  all: ReadonlySet<string>,
): AccessPaths {
  const owned = new Set(own.values());
  const granting = new Set(rolesWith(roles, all));
  return {
    direct: role.privileges.some((name) => owned.has(name)),
    wide: role.privileges.some((name) => all.has(name) && !owned.has(name)),
    nested: (role.roles ?? []).some((id) => granting.has(id)),
  };
}

export function grantsOtherwise(
  role: RoleRow,
  roles: readonly RoleRow[],
  own: ReadonlyMap<string, string>,
  all: ReadonlySet<string>,
): boolean {
  const paths = accessPaths(role, roles, own, all);
  return paths.wide || paths.nested;
}

export interface AnonymousPlan {
  blocked: string[];
  roles: RoleRow[];
  sharedWith: string[];
}

export function closeForAnonymous(
  anonymous: UserRow,
  users: readonly UserRow[],
  roles: readonly RoleRow[],
  own: ReadonlyMap<string, string>,
  all: ReadonlySet<string>,
): AnonymousPlan {
  const granting = new Set(rolesWith(roles, all));
  const held = roles.filter((role) => anonymous.roles.includes(role.id) && granting.has(role.id));
  const blocked = held
    .filter((role) => role.readOnly || grantsOtherwise(role, roles, own, all))
    .map((role) => role.id);
  const editable = held.filter((role) => !role.readOnly && grantsDirectly(role, own));
  const ids = new Set(editable.map((role) => role.id));
  const sharedWith = users
    .filter((user) => user.userId !== anonymous.userId && user.roles.some((role) => ids.has(role)))
    .map((user) => user.userId);
  return { blocked, roles: editable, sharedWith };
}

export function openForAnonymous(
  anonymous: UserRow,
  users: readonly UserRow[],
  roles: readonly RoleRow[],
): RoleRow | null {
  const own = roles.filter(
    (role) =>
      anonymous.roles.includes(role.id) &&
      !role.readOnly &&
      !users.some((user) => user.userId !== anonymous.userId && user.roles.includes(role.id)),
  );
  return own.find((role) => role.id === ANONYMOUS_ROLE) ?? own[0] ?? null;
}

export function roleBody(role: RoleRow, privileges: readonly string[]) {
  return {
    id: role.id,
    name: role.name ?? role.id,
    description: role.description ?? '',
    privileges,
    roles: role.roles ?? [],
  };
}

export function userBody(user: UserRow, roles: readonly string[]) {
  return {
    userId: user.userId,
    firstName: user.firstName,
    lastName: user.lastName,
    emailAddress: user.emailAddress,
    source: user.source ?? 'default',
    status: user.status,
    roles,
  };
}
