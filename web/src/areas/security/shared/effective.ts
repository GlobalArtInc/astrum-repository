import type { Privilege, Role } from '../api';

export interface EffectivePrivilege {
  name: string;
  privilege?: Privilege;
  paths: string[][];
}

export interface Effective {
  privileges: EffectivePrivilege[];
  missingRoles: string[];
}

export function effectivePrivileges(
  roleIds: readonly string[],
  roles: ReadonlyMap<string, Role>,
  privileges: ReadonlyMap<string, Privilege>,
): Effective {
  const found = new Map<string, EffectivePrivilege>();
  const seenPaths = new Map<string, Set<string>>();
  const missing = new Set<string>();

  const visit = (id: string, trail: string[]) => {
    if (trail.includes(id)) {
      return;
    }
    const role = roles.get(id);
    if (!role) {
      missing.add(id);
      return;
    }
    const path = [...trail, id];
    for (const name of role.privileges) {
      let entry = found.get(name);
      if (!entry) {
        entry = { name, privilege: privileges.get(name), paths: [] };
        found.set(name, entry);
        seenPaths.set(name, new Set());
      }
      const key = path.join('\u0000');
      const known = seenPaths.get(name);
      if (known && !known.has(key)) {
        known.add(key);
        entry.paths.push(path);
      }
    }
    for (const nested of role.roles) {
      visit(nested, path);
    }
  };

  for (const id of roleIds) {
    visit(id, []);
  }

  const privilegesList = [...found.values()]
    .map((entry) => ({
      ...entry,
      paths: [...entry.paths].sort(
        (left, right) => left.length - right.length || left.join('/').localeCompare(right.join('/')),
      ),
    }))
    .sort((left, right) => left.name.localeCompare(right.name));
  return { privileges: privilegesList, missingRoles: [...missing].sort() };
}
