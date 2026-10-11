

import { SETTINGS_SECTIONS, type SettingCard } from '../settingsConfig';

const cardById = (id: string): SettingCard | undefined =>
  SETTINGS_SECTIONS.flatMap((section) => section.cards).find((card) => card.id === id);

// SettingsHubPage.test.tsx mocks ../settingsConfig, so it exercises the filtering
// mechanism but never asserts the real config's gates. These tests run against the
// REAL config so a fix that removes a card's visibilityRequirements fails here
// instead of silently regressing (ASTRUM-54048).
describe('settingsConfig without the Classic UI', () => {
  it('has no Astrum One UI card, since the Classic UI can no longer be chosen', () => {
    expect(cardById('preview-ui')).toBeUndefined();
    const paths = SETTINGS_SECTIONS.flatMap((section) => section.cards).map((card) => card.path);
    expect(paths).not.toContain('system/previewui');
  });
});

describe('settingsConfig permission gating (ASTRUM-54048)', () => {
  it('gates the API card on astrum:settings:read', () => {
    // The API page has no dedicated privilege; the Default UI and Classic UI gate it on
    // astrum:settings:read, so the Settings Hub card must carry the same requirement or an
    // unauthorized user sees a card that then fails with a 403.
    const api = cardById('api');
    expect(api).toBeDefined();
    expect(api?.visibilityRequirements).toBeDefined();
    expect(api?.visibilityRequirements?.requiresPermission).toBe('astrum:settings:read');
  });

  // The Settings Hub cards must carry the SAME gate as their corresponding route, which in turn
  // mirrors the Default UI (adminRoutes.js). A card that is more permissive than its route shows
  // a tile that then dead-ends in a 403. These lock each card's gate to the Default UI value.
  it('gates the repositories card on the astrum:repository-admin prefix', () => {
    const card = cardById('repositories');
    expect(card?.visibilityRequirements?.permissionPrefix).toBe('astrum:repository-admin');
    expect(card?.visibilityRequirements?.requiresPermission).toBeUndefined();
  });

  it('gates the cleanup-policies card on astrum:*', () => {
    const card = cardById('cleanup-policies');
    expect(card?.visibilityRequirements?.requiresPermission).toBe('astrum:*');
  });

  it('gates the routing-rules card on astrum:*', () => {
    const card = cardById('routing-rules');
    expect(card?.visibilityRequirements?.requiresPermission).toBe('astrum:*');
  });

  // ASTRUM-54048: datastore, saml, oauth2, and nodes are admin-only screens. Their Default UI
  // routes (adminRoutes.js DATASTORE/SAML/OAUTH2/NODES.ROOT) all gate on astrum:*, so a
  // settings:read-only user must not see the tile and then hit a 403 opening the page.
  it.each(['data-store', 'saml', 'oauth2', 'nodes'])(
    'gates the %s card on astrum:*',
    (cardId) => {
      const card = cardById(cardId);
      expect(card).toBeDefined();
      expect(card?.visibilityRequirements?.requiresPermission).toBe('astrum:*');
    },
  );

  it('requires both roles:read and privileges:read for the roles card', () => {
    const card = cardById('roles');
    expect(card?.visibilityRequirements?.permissions).toEqual([
      'astrum:roles:read',
      'astrum:privileges:read',
    ]);
  });

  it('requires both users:read and roles:read for the users card', () => {
    const card = cardById('users');
    expect(card?.visibilityRequirements?.permissions).toEqual([
      'astrum:users:read',
      'astrum:roles:read',
    ]);
  });

  it('gates the user-tokens card on usertoken-settings:read for PRO only', () => {
    const card = cardById('user-tokens');
    expect(card?.visibilityRequirements?.requiresPermission).toBe('astrum:usertoken-settings:read');
    expect(card?.visibilityRequirements?.editions).toEqual(['PRO']);
  });

  it('gates the usage card on astrum:settings:read (matches cloud Classic)', () => {
    const card = cardById('usage');
    expect(card?.visibilityRequirements?.requiresPermission).toBe('astrum:settings:read');
  });

  // ASTRUM-54019: align Settings Hub cards with Classic/Default UI (adminRoutes.js) edition/bundle/
  // license/capability gates and correct three permission-string mismatches. Each card must carry
  // the SAME gate as its route or a tile dead-ends in a 403 / shows where the feature is absent.
  describe('ASTRUM-54019 alignment with Default UI gates', () => {
    it('gates data-store on the pro-datastore bundle + PRO/COMMUNITY editions', () => {
      const card = cardById('data-store');
      expect(card?.visibilityRequirements?.bundle).toBe('astrum-pro-datastore-plugin');
      expect(card?.visibilityRequirements?.editions).toEqual(['PRO', 'COMMUNITY']);
    });

    it('gates saml on the saml bundle + PRO edition', () => {
      const card = cardById('saml');
      expect(card?.visibilityRequirements?.bundle).toBe('astrum-saml-plugin');
      expect(card?.visibilityRequirements?.editions).toEqual(['PRO']);
    });

    it('gates oauth2 only on the oauth2Available state, with no bundle or edition requirement', () => {
      const card = cardById('oauth2');
      expect(card?.visibilityRequirements?.bundle).toBeUndefined();
      expect(card?.visibilityRequirements?.editions).toBeUndefined();
      expect(card?.visibilityRequirements?.statesEnabled).toContainEqual({
        key: 'oauth2Available',
        defaultValue: false,
      });
    });

    it('gates recovery mode on astrum:* and the recoveryModeAvailable state', () => {
      const card = cardById('recovery-mode');
      expect(card?.visibilityRequirements?.permissions).toEqual(['astrum:*']);
      expect(card?.visibilityRequirements?.statesEnabled).toContainEqual({
        key: 'recoveryModeAvailable',
        defaultValue: false,
      });
    });

    it('gates crowd on the crowd bundle + a valid crowd license', () => {
      const card = cardById('crowd');
      expect(card?.visibilityRequirements?.bundle).toBe('astrum-crowd-plugin');
      expect(card?.visibilityRequirements?.licenseValid).toContainEqual({
        key: 'crowd',
        defaultValue: false,
      });
    });

    it('gates user-tokens on a valid usertoken license', () => {
      const card = cardById('user-tokens');
      expect(card?.visibilityRequirements?.licenseValid).toContainEqual({
        key: 'usertoken',
        defaultValue: false,
      });
    });

    it('gates service-account-tokens on PRO edition + serviceAccountEnabled state', () => {
      const card = cardById('service-account-tokens');
      expect(card?.visibilityRequirements?.editions).toEqual(['PRO']);
      expect(card?.visibilityRequirements?.statesEnabled).toContainEqual({
        key: 'serviceAccountEnabled',
        defaultValue: false,
      });
    });

    it('gates ldap on ldap:read and the ldap plugin bundle with NO edition restriction', () => {
      const card = cardById('ldap');
      expect(card?.visibilityRequirements?.requiresPermission).toBe('astrum:ldap:read');
      expect(card?.visibilityRequirements?.bundle).toBe('astrum-ldap-plugin');
      expect(card?.visibilityRequirements?.editions).toBeUndefined();
    });

    it('gates support-request on atlas:create + PRO edition', () => {
      const card = cardById('support-request');
      expect(card?.visibilityRequirements?.requiresPermission).toBe('astrum:atlas:create');
      expect(card?.visibilityRequirements?.editions).toEqual(['PRO']);
    });

    it('gates support-zip on atlas:read', () => {
      const card = cardById('support-zip');
      expect(card?.visibilityRequirements?.requiresPermission).toBe('astrum:atlas:read');
    });

    // ASTRUM-54237: informational page — no migration capability or migration:read requirement.
  });

});
