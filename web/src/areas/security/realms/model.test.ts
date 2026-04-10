import { describe, expect, it } from 'vitest';
import {
  activate,
  activeRealms,
  deactivate,
  inactiveRealms,
  moveBy,
  moveTo,
  realmLabel,
  sameOrder,
} from './model';

const AVAILABLE = [
  { id: 'AccessTokenRealm', name: 'Access tokens' },
  { id: 'AstrumAuthenticatingRealm', name: 'Local users' },
  { id: 'LdapRealm', name: 'LDAP' },
];

describe('realm ordering', () => {
  it('moves a realm to a position and keeps the others in order', () => {
    expect(moveTo(['a', 'b', 'c', 'd'], 3, 0)).toEqual(['d', 'a', 'b', 'c']);
    expect(moveTo(['a', 'b', 'c'], 0, 2)).toEqual(['b', 'c', 'a']);
    expect(moveTo(['a', 'b', 'c'], 1, 9)).toEqual(['a', 'c', 'b']);
    expect(moveTo(['a', 'b'], 5, 0)).toEqual(['a', 'b']);
  });

  it('moves up and down one step and stops at the ends', () => {
    expect(moveBy(['a', 'b', 'c'], 'c', -1)).toEqual(['a', 'c', 'b']);
    expect(moveBy(['a', 'b', 'c'], 'a', -1)).toEqual(['a', 'b', 'c']);
    expect(moveBy(['a', 'b', 'c'], 'c', 1)).toEqual(['a', 'b', 'c']);
    expect(moveBy(['a', 'b'], 'x', 1)).toEqual(['a', 'b']);
  });

  it('activates at the end and deactivates without touching the order', () => {
    expect(activate(['a'], 'b')).toEqual(['a', 'b']);
    expect(activate(['a', 'b'], 'a')).toEqual(['a', 'b']);
    expect(deactivate(['a', 'b', 'c'], 'b')).toEqual(['a', 'c']);
    expect(sameOrder(['a', 'b'], ['a', 'b'])).toBe(true);
    expect(sameOrder(['a', 'b'], ['b', 'a'])).toBe(false);
  });

  it('splits the realms into active, in order, and inactive', () => {
    expect(activeRealms(AVAILABLE, ['LdapRealm', 'Unknown']).map((realm) => realm.id)).toEqual([
      'LdapRealm',
      'Unknown',
    ]);
    expect(inactiveRealms(AVAILABLE, ['LdapRealm']).map((realm) => realm.id)).toEqual([
      'AccessTokenRealm',
      'AstrumAuthenticatingRealm',
    ]);
  });

  it('names a realm as the server does', () => {
    expect(realmLabel({ id: 'AstrumAuthenticatingRealm', name: 'Local Authenticating Realm' })).toBe(
      'Local Authenticating Realm',
    );
    expect(realmLabel({ id: 'Custom', name: '' })).toBe('Custom');
  });
});
