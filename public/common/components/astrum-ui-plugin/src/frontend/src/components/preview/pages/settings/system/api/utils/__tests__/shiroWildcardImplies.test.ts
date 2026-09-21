

import { shiroWildcardImplies, userImpliesPermission, wildcardImplies } from '../shiroWildcardImplies';

describe('wildcardImplies', () => {
  it('matches exact permission', () => {
    expect(wildcardImplies('astrum:settings:read', 'astrum:settings:read')).toBe(true);
  });

  it('astrum:* implies astrum:settings:read', () => {
    expect(wildcardImplies('astrum:*', 'astrum:settings:read')).toBe(true);
  });

  it('narrow permission does not imply broader', () => {
    expect(wildcardImplies('astrum:settings:read', 'astrum:*')).toBe(false);
  });

  it('repository-admin style wildcards', () => {
    expect(wildcardImplies('astrum:repository-admin:*:*:read', 'astrum:repository-admin:maven:repo:read')).toBe(true);
  });
});

describe('shiroWildcardImplies', () => {
  it('returns true if any grant implies required', () => {
    expect(shiroWildcardImplies('astrum:foo:read', ['astrum:*', 'other:thing'])).toBe(true);
  });

  it('returns false when no grant matches', () => {
    expect(shiroWildcardImplies('astrum:foo:read', ['astrum:bar:read'])).toBe(false);
  });
});

describe('userImpliesPermission', () => {
  it('reads truthy keys from permission map', () => {
    const map: Record<string, boolean> = { 'astrum:*': true, 'astrum:bar:read': false };
    expect(userImpliesPermission('astrum:settings:read', map)).toBe(true);
  });
});
