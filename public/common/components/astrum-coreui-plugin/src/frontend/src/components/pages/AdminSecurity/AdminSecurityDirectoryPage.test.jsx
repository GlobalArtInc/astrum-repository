

import givenUserLoggedIn from '../../../testUtils/givenUserLoggedIn';
import givenBundleActiveStates from '../../../testUtils/givenBundleActiveStates';
import givenExtJSState from '../../../testUtils/givenExtJSState';
import { runLinkNotVisibleTest, runLinkVisiblityTest, runDirectoryPage404Test } from '../../../testUtils/directoryPageTestUtils';
import { ROUTE_NAMES } from '../../../routerConfig/routeNames/routeNames';
import UIStrings from '../../../constants/UIStrings';
import givenPermissions from '../../../testUtils/givenPermissions';
import { Permissions } from '@globalart/astrum-ui-plugin';

describe('AdminSecurityDirectoryPage', () => {
  beforeEach(() => {
    givenUserLoggedIn();

    givenBundleActiveStates({
      'astrum-coreui-plugin': true
    });

    givenExtJSState(defaultExtState());
  });

  it('redirects to 404 when user has no permissions for any child routes', async () => {
    givenPermissions({});

    await runDirectoryPage404Test(ROUTE_NAMES.ADMIN.SECURITY.DIRECTORY);
  });

  describe('Privileges Link', () => {
    it('shows given permissions', async () => {
      givenPermissions({ [Permissions.PRIVILEGES.READ]: true });
      await runLinkVisiblityTestForSecurityPage(UIStrings.PRIVILEGES.MENU);
    });

    it('does not show without permissions', async () => {
      givenPermissions({
        [Permissions.PRIVILEGES.READ]: false,
        [Permissions.SETTINGS.READ]: true  // Grant at least one permission so page renders
      });
      await runLinkNotVisibleTestForSecurityPage(UIStrings.PRIVILEGES.MENU);
    });
  });

  describe('Roles Link', () => {
    it('shows given permissions', async () => {
      givenPermissions({
        'astrum:roles:read': true,
        'astrum:privileges:read': true
      });
      await runLinkVisiblityTestForSecurityPage(UIStrings.ROLES.MENU);
    });

    it('does not show without permissions', async () => {
      givenPermissions({
        'astrum:roles:read': true,
        'astrum:privileges:read': false,
        [Permissions.SETTINGS.READ]: true  // Grant at least one permission so page renders
      });
      await runLinkNotVisibleTestForSecurityPage(UIStrings.ROLES.MENU);
    });
  });

  describe('Users Link', () => {
    it('shows given permissions', async () => {
      givenPermissions({
        [Permissions.USERS.READ]: true,
        [Permissions.ROLES.READ]: true
      });
      await runLinkVisiblityTestForSecurityPage(UIStrings.USERS.MENU)
    });

    it('does not show without permissions', async () => {
      givenPermissions({
        [Permissions.USERS.READ]: true,
        [Permissions.ROLES.READ]: false,
        [Permissions.SETTINGS.READ]: true  // Grant at least one permission so page renders
      });
      await runLinkNotVisibleTestForSecurityPage(UIStrings.USERS.MENU);
    });
  });

  describe('Anonymous Settings Link', () => {
    it('shows given permissions', async () => {
      givenPermissions({ [Permissions.SETTINGS.READ]: true });
      await runLinkVisiblityTestForSecurityPage(UIStrings.ANONYMOUS_SETTINGS.MENU)
    });

    it('does not show without permissions', async () => {
      givenPermissions({
        [Permissions.SETTINGS.READ]: false,
        [Permissions.PRIVILEGES.READ]: true  // Grant at least one permission so page renders
      });
      await runLinkNotVisibleTestForSecurityPage(UIStrings.ANONYMOUS_SETTINGS.MENU);
    });
  });

  describe('Crowd Settings Link', () => {
    beforeEach(() => {
      givenBundleActiveStates({
        'astrum-crowd-plugin': true
      });

      givenExtJSState({ ...defaultExtState(), crowd: { licenseValid: true }  });
    })

    it('shows given permissions', async () => {
      givenPermissions({ 'astrum:crowd:read': true });
      await runLinkVisiblityTestForSecurityPage(UIStrings.CROWD_SETTINGS.MENU)
    });

    it('does not show without permissions', async () => {
      givenPermissions({
        'astrum:crowd:read': false,
        [Permissions.SETTINGS.READ]: true  // Grant at least one permission so page renders
      });
      await runLinkNotVisibleTestForSecurityPage(UIStrings.CROWD_SETTINGS.MENU);
    });
  });

  describe('LDAP Servers Link', () => {
    it('shows given permissions and the ldap plugin bundle', async () => {
      givenBundleActiveStates({ 'astrum-ldap-plugin': true });
      givenPermissions({ [Permissions.LDAP.READ]: true });
      await runLinkVisiblityTestForSecurityPage(UIStrings.LDAP_SERVERS.MENU)
    });

    it('does not show without the ldap plugin bundle', async () => {
      givenBundleActiveStates({ 'astrum-ldap-plugin': false });
      givenPermissions({ [Permissions.LDAP.READ]: true, [Permissions.SETTINGS.READ]: true });
      await runLinkNotVisibleTestForSecurityPage(UIStrings.LDAP_SERVERS.MENU);
    });

    it('does not show without permissions', async () => {
      givenPermissions({
        [Permissions.LDAP.READ]: false,
        [Permissions.SETTINGS.READ]: true  // Grant at least one permission so page renders
      });
      await runLinkNotVisibleTestForSecurityPage(UIStrings.LDAP_SERVERS.MENU);
    });
  });

  describe('Realms Link', () => {
    it('shows given permissions', async () => {
      givenPermissions({ [Permissions.SETTINGS.READ]: true });
      await runLinkVisiblityTestForSecurityPage(UIStrings.REALMS.MENU)
    });

    it('does not show without permissions', async () => {
      givenPermissions({
        [Permissions.SETTINGS.READ]: false,
        [Permissions.PRIVILEGES.READ]: true  // Grant at least one permission so page renders
      });
      await runLinkNotVisibleTestForSecurityPage(UIStrings.REALMS.MENU);
    });
  });

  describe('SAML Configuration Link', () => {
    beforeEach(() => {
      givenBundleActiveStates({ 'astrum-saml-plugin': true })
      givenExtJSState(defaultExtState(), 'PRO')
    })
    it('shows given permissions', async () => {
      givenPermissions({ 'astrum:*': true });
      await runLinkVisiblityTestForSecurityPage(UIStrings.SAML_CONFIGURATION.MENU)
    });

    it('does not show without permissions', async () => {
      givenPermissions({
        'astrum:*': false,
        [Permissions.SETTINGS.READ]: true  // Grant at least one permission so page renders
      });
      await runLinkNotVisibleTestForSecurityPage(UIStrings.SAML_CONFIGURATION.MENU);
    });
  });

  describe('SSL Certificates Link', () => {
    it('shows given permissions', async () => {
      givenPermissions({ [Permissions.SSL_TRUSTSTORE.READ]: true });
      await runLinkVisiblityTestForSecurityPage(UIStrings.SSL_CERTIFICATES.MENU);
    });

    it('does not show without permissions', async () => {
      givenPermissions({
        [Permissions.SSL_TRUSTSTORE.READ]: false,
        [Permissions.SETTINGS.READ]: true  // Grant at least one permission so page renders
      });
      await runLinkNotVisibleTestForSecurityPage(UIStrings.SSL_CERTIFICATES.MENU);
    });
  });

  describe('User Token Configuration Link', () => {
    beforeEach(() => {
      givenExtJSState(defaultExtState(),  'PRO');
    });

    it('shows given permissions', async () => {
      givenPermissions({ [Permissions.USER_TOKENS_SETTINGS.READ]: true});
      await runLinkVisiblityTestForSecurityPage(UIStrings.USER_TOKEN_CONFIGURATION.MENU)
    });

    it('does not show without permissions', async () => {
      givenPermissions({
        [Permissions.USER_TOKENS_SETTINGS.READ]: false,
        [Permissions.SETTINGS.READ]: true  // Grant at least one permission so page renders
      });
      await runLinkNotVisibleTestForSecurityPage(UIStrings.USER_TOKEN_CONFIGURATION.MENU);
    });
  });

  describe('Service Account Tokens Link', () => {
    it('shows when serviceAccountEnabled flag is true', async () => {
      givenPermissions({ [Permissions.SERVICE_ACCOUNTS.READ]: true });
      givenExtJSState({ ...defaultExtState(), serviceAccountEnabled: true }, 'PRO');

      await runLinkVisiblityTestForSecurityPage(UIStrings.SERVICE_ACCOUNT_TOKENS.MENU);
    });

    it('does not show when serviceAccountEnabled flag is false', async () => {
      givenPermissions({ [Permissions.SERVICE_ACCOUNTS.READ]: true, [Permissions.SETTINGS.READ]: true });
      givenExtJSState({ ...defaultExtState(), serviceAccountEnabled: false }, 'PRO');

      await runLinkNotVisibleTestForSecurityPage(UIStrings.SERVICE_ACCOUNT_TOKENS.MENU);
    });

    it('does not show without service accounts read permission', async () => {
      givenPermissions({ [Permissions.SERVICE_ACCOUNTS.READ]: false, [Permissions.SETTINGS.READ]: true });
      givenExtJSState({ ...defaultExtState(), serviceAccountEnabled: true }, 'PRO');

      await runLinkNotVisibleTestForSecurityPage(UIStrings.SERVICE_ACCOUNT_TOKENS.MENU);
    });
  });

  async function runLinkVisiblityTestForSecurityPage(menu) {
    await runLinkVisiblityTest(ROUTE_NAMES.ADMIN.SECURITY.DIRECTORY, 'Security', menu);
  }

  async function runLinkNotVisibleTestForSecurityPage(menu) {
    await runLinkNotVisibleTest(ROUTE_NAMES.ADMIN.SECURITY.DIRECTORY, 'Security', menu);
  }

  function defaultExtState() {
    return { 
        usertoken: { licenseValid: true }
    };
  }
});
