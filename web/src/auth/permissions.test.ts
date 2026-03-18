import { describe, expect, it } from 'vitest';
import { implies, overlaps, PermissionSet, parsePermission } from './permissions';

function grants(granted: string, wanted: string): boolean {
  const left = parsePermission(granted);
  const right = parsePermission(wanted);
  if (!left || !right) {
    throw new Error('bad permission in test');
  }
  return implies(left, right);
}

describe('permissions match like the server', () => {
  it('grants everything under a prefix', () => {
    expect(grants('astrum:*', 'astrum:users:read')).toBe(true);
    expect(grants('astrum', 'astrum:users:read')).toBe(true);
    expect(grants('*', 'astrum:repository-view:maven2:central:read')).toBe(true);
    expect(grants('astrum:*', 'other:users:read')).toBe(false);
  });

  it('matches values and lists', () => {
    expect(grants('astrum:users:read,create', 'astrum:users:read')).toBe(true);
    expect(grants('astrum:users:read,create', 'astrum:users:create,read')).toBe(true);
    expect(grants('astrum:users:read', 'astrum:users:read,create')).toBe(false);
    expect(grants('astrum:users:read', 'astrum:users:*')).toBe(false);
    expect(grants('astrum:users:*', 'astrum:users:delete')).toBe(true);
  });

  it('matches repository permissions', () => {
    const granted = 'astrum:repository-view:maven2:*:read,browse';
    expect(grants(granted, 'astrum:repository-view:maven2:central:read')).toBe(true);
    expect(grants(granted, 'astrum:repository-view:maven2:releases:browse')).toBe(true);
    expect(grants(granted, 'astrum:repository-view:npm:central:read')).toBe(false);
    expect(grants(granted, 'astrum:repository-view:maven2:central:add')).toBe(false);
    expect(grants('astrum:repository-view:*:*:*', 'astrum:repository-view:npm:x:delete')).toBe(true);
  });

  it('needs wildcards at the end of a longer grant', () => {
    expect(grants('astrum:users:read:*', 'astrum:users:read')).toBe(true);
    expect(grants('astrum:users:read:own', 'astrum:users:read')).toBe(false);
  });

  it('ignores case and spaces', () => {
    expect(grants('Astrum:Users:READ', 'astrum:users:read')).toBe(true);
    expect(grants('astrum:users: read , create', 'astrum:users:create')).toBe(true);
  });

  it('refuses malformed permissions', () => {
    for (const bad of ['', ' ', 'astrum::read', 'astrum:users:', 'astrum:a,,b']) {
      expect(parsePermission(bad)).toBeNull();
    }
  });
});

describe('overlapping permissions', () => {
  const overlap = (granted: string, pattern: string) =>
    overlaps(parsePermission(granted) ?? { parts: [] }, parsePermission(pattern) ?? { parts: [] });

  it('finds a grant that allows something under a pattern', () => {
    expect(overlap('astrum:repository-view:maven2:central:read', 'astrum:repository-view')).toBe(true);
    expect(overlap('astrum:*', 'astrum:repository-admin')).toBe(true);
    expect(overlap('astrum:repository-view:maven2:central:read', 'astrum:repository-view:*:*:browse')).toBe(
      false,
    );
    expect(overlap('astrum:repository-view:maven2:*:browse,read', 'astrum:repository-view:*:*:browse')).toBe(
      true,
    );
    expect(overlap('astrum:users:read', 'astrum:repository-view')).toBe(false);
  });
});

describe('a permission set', () => {
  const set = new PermissionSet([
    'astrum:search:read',
    'astrum:repository-view:maven2:*:browse,read',
    'astrum:repository-content-selector:team:raw:*:read',
    'not a permission::',
  ]);

  it('answers single permissions', () => {
    expect(set.has('astrum:search:read')).toBe(true);
    expect(set.has('astrum:users:read')).toBe(false);
    expect(set.has('broken::')).toBe(false);
  });

  it('evaluates needs', () => {
    expect(set.satisfies(undefined)).toBe(true);
    expect(set.satisfies('astrum:search:read')).toBe(true);
    expect(set.satisfies({ all: ['astrum:search:read', 'astrum:users:read'] })).toBe(false);
    expect(set.satisfies({ any: ['astrum:users:read', 'astrum:search:read'] })).toBe(true);
    expect(set.satisfies({ some: 'astrum:repository-content-selector' })).toBe(true);
    expect(set.satisfies({ some: 'astrum:repository-admin' })).toBe(false);
    expect(
      set.satisfies({ any: [{ some: 'astrum:repository-admin' }, { all: ['astrum:search:read'] }] }),
    ).toBe(true);
  });
});
