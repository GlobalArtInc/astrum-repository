type Part = '*' | ReadonlySet<string>;

export interface Permission {
  readonly parts: readonly Part[];
}

export type Need = string | { all: Need[] } | { any: Need[] } | { some: string };

export function parsePermission(text: string): Permission | null {
  const trimmed = text.trim();
  if (trimmed === '') {
    return null;
  }
  const parts: Part[] = [];
  for (const part of trimmed.split(':')) {
    const values = part.split(',').map((value) => value.trim().toLowerCase());
    if (values.some((value) => value === '')) {
      return null;
    }
    parts.push(values.includes('*') ? '*' : new Set(values));
  }
  return { parts };
}

function isSubset(wanted: ReadonlySet<string>, granted: ReadonlySet<string>): boolean {
  for (const value of wanted) {
    if (!granted.has(value)) {
      return false;
    }
  }
  return true;
}

function intersects(left: ReadonlySet<string>, right: ReadonlySet<string>): boolean {
  for (const value of left) {
    if (right.has(value)) {
      return true;
    }
  }
  return false;
}

export function implies(granted: Permission, wanted: Permission): boolean {
  for (const [index, want] of wanted.parts.entries()) {
    const grant = granted.parts[index];
    if (grant === undefined) {
      return true;
    }
    if (grant === '*') {
      continue;
    }
    if (want === '*' || !isSubset(want, grant)) {
      return false;
    }
  }
  return granted.parts.slice(wanted.parts.length).every((part) => part === '*');
}

export function overlaps(granted: Permission, pattern: Permission): boolean {
  const length = Math.max(granted.parts.length, pattern.parts.length);
  for (let index = 0; index < length; index += 1) {
    const grant = granted.parts[index];
    const want = pattern.parts[index];
    if (grant === undefined || want === undefined) {
      return true;
    }
    if (grant === '*' || want === '*') {
      continue;
    }
    if (!intersects(grant, want)) {
      return false;
    }
  }
  return true;
}

export class PermissionSet {
  private readonly granted: Permission[];

  constructor(permissions: readonly string[]) {
    this.granted = permissions.map(parsePermission).filter((permission) => permission !== null);
  }

  has(wanted: string): boolean {
    const permission = parsePermission(wanted);
    return permission !== null && this.granted.some((granted) => implies(granted, permission));
  }

  hasSome(pattern: string): boolean {
    const permission = parsePermission(pattern);
    return permission !== null && this.granted.some((granted) => overlaps(granted, permission));
  }

  satisfies(need: Need | undefined): boolean {
    if (need === undefined) {
      return true;
    }
    if (typeof need === 'string') {
      return this.has(need);
    }
    if ('all' in need) {
      return need.all.every((inner) => this.satisfies(inner));
    }
    if ('any' in need) {
      return need.any.some((inner) => this.satisfies(inner));
    }
    return this.hasSome(need.some);
  }
}
