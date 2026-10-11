

import givenUserLoggedIn from '../../../testUtils/givenUserLoggedIn';
import givenBundleActiveStates from '../../../testUtils/givenBundleActiveStates';
import givenExtJSState from '../../../testUtils/givenExtJSState';
import givenPermissions from '../../../testUtils/givenPermissions';
import { Permissions } from '@globalart/astrum-ui-plugin';
import UIStrings from '../../../constants/UIStrings';
import { runLinkNotVisibleTest, runLinkVisiblityTest, runDirectoryPage404Test } from '../../../testUtils/directoryPageTestUtils';
import { ROUTE_NAMES } from '../../../routerConfig/routeNames/routeNames';
import {screen} from "@testing-library/react";
import {renderComponentRoute} from "../../../testUtils/renderUtils";

describe('AdminSystemDirectoryPage', () => {
  beforeEach(() => {
    givenUserLoggedIn();

    givenBundleActiveStates({
      'astrum-coreui-plugin': true
    });

    givenExtJSState(defaultExtState());
  });

  it('redirects to 404 when user has no permissions for any child routes', async () => {
    givenPermissions({});

    await runDirectoryPage404Test(ROUTE_NAMES.ADMIN.SYSTEM.DIRECTORY);
  });

  describe('API Link', () => {
    it('shows given permissions', async () => {
      givenPermissions({ [Permissions.SETTINGS.READ]: true });
      await runLinkVisiblityTestForSystemPage(UIStrings.API.MENU);
    });

    it('does not show without permissions', async () => {
      givenPermissions({
        [Permissions.SETTINGS.READ]: false,
        [Permissions.TASKS.READ]: true  // Grant at least one permission so page renders
      });
      await runLinkNotVisibleTestForSystemPage(UIStrings.API.MENU);
    });
  });

  describe('Astrum One UI Link', () => {
    it('is gone now that the Classic UI cannot be chosen', async () => {
      givenExtJSState({ ...defaultExtState(), 'astrum.previewui.enabled': true });
      givenPermissions({ [Permissions.SETTINGS.READ]: true });
      await runLinkNotVisibleTestForSystemPage(UIStrings.PREVIEW_UI_SETTINGS.MENU);
    });
  });

  describe('Capabilities Link', () => {
    it('renders both extjs and react links given sufficient permissions and feature flags enabled', async () => {
      givenPermissions({ 'astrum:capabilities:read': true });

      await renderComponentRoute(ROUTE_NAMES.ADMIN.SYSTEM.DIRECTORY);

      const extJsLink = screen.getAllByRole('link', { name: /^Capabilities\b/ })[0];
      expect(extJsLink).toBeVisible();
      expect(extJsLink.href).toContain('/#admin/system/capabilities-extjs');

      const reactLink = screen.getAllByRole('link', { name: /^Capabilities\b/ })[1];
      expect(reactLink).toBeVisible();
      expect(reactLink.href).toContain('/#admin/system/capabilities');
    });

    it('does not render any link when user does not have permissions', async () => {
      givenPermissions({
        ['astrum:capabilities:read']: false,
        [Permissions.TASKS.READ]: true  // Grant at least one permission so page renders
      });
      await runLinkNotVisibleTestForSystemPage(UIStrings.CAPABILITIES.MENU);
    })

    it('does not render react link when react feature flag is disabled', async () => {
      givenUserLoggedIn();
      givenPermissions({ 'astrum:capabilities:read': true });
      givenExtJSState({
        ...defaultExtState(),
        'astrum.react.capabilities.enabled': false
      })

      await renderComponentRoute(ROUTE_NAMES.ADMIN.SYSTEM.DIRECTORY);

      const links = screen.getAllByRole('link', { name: /^Capabilities\b/ });
      // Expect nav sidebar link + content list link (both extjs since react is disabled)
      expect(links).toHaveLength(2);
      links.forEach(link => {
        expect(link).toBeVisible();
        expect(link.href).toContain('/#admin/system/capabilities-extjs');
      });
    });

    it('does not render ExtJs link when ExtJs feature flag is disabled', async () => {
      givenUserLoggedIn();
      givenPermissions({ 'astrum:capabilities:read': true });
      givenExtJSState({
        ...defaultExtState(),
        'astrum.extjs.capabilities.enabled': false
      });

      await renderComponentRoute(ROUTE_NAMES.ADMIN.SYSTEM.DIRECTORY);

      const links = screen.getAllByRole('link', { name: /^Capabilities\b/ });
      // Expect nav sidebar link + content list link (both react since extjs is disabled)
      expect(links).toHaveLength(2);
      links.forEach(link => {
        expect(link).toBeVisible();
        expect(link.href).toContain('/#admin/system/capabilities');
        expect(link.href).not.toContain('capabilities-extjs');
      });
    });

    it('does not render any link when both ExtJs and React feature flag is disabled', async () => {
      givenPermissions({
        ['astrum:capabilities:read']: true,
        [Permissions.TASKS.READ]: true  // Grant at least one permission so page renders
      });
      givenExtJSState({
        ...defaultExtState(),
        'astrum.extjs.capabilities.enabled': false,
        'astrum.react.capabilities.enabled': false
      });
      await runLinkNotVisibleTestForSystemPage(UIStrings.CAPABILITIES.MENU);
    })
  });

  describe('Email Server Link', () => {
    it('shows given permissions', async () => {
      givenPermissions({ [Permissions.SETTINGS.READ]: true });
      await runLinkVisiblityTestForSystemPage(UIStrings.EMAIL_SERVER.MENU);
    });

    it('does not show without permissions', async () => {
      givenPermissions({
        [Permissions.SETTINGS.READ]: false,
        [Permissions.TASKS.READ]: true  // Grant at least one permission so page renders
      });
      await runLinkNotVisibleTestForSystemPage(UIStrings.EMAIL_SERVER.MENU);
    });
  });

  describe('HTTP Link', () => {
    it('shows given permissions', async () => {
      givenPermissions({ [Permissions.SETTINGS.READ]: true });
      await runLinkVisiblityTestForSystemPage(UIStrings.HTTP.MENU);
    });

    it('does not show without permissions', async () => {
      givenPermissions({
        [Permissions.SETTINGS.READ]: false,
        [Permissions.TASKS.READ]: true  // Grant at least one permission so page renders
      });
      await runLinkNotVisibleTestForSystemPage(UIStrings.HTTP.MENU);
    });
  });

  describe('Nodes Link', () => {
    it('shows given permissions', async () => {
      givenPermissions({ [Permissions.ADMIN]: true });
      await runLinkVisiblityTestForSystemPage(UIStrings.NODES.MENU);
    });

    it('does not show without permissions', async () => {
      givenPermissions({
        [Permissions.ADMIN]: false,
        [Permissions.TASKS.READ]: true  // Grant at least one permission so page renders
      });
      await runLinkNotVisibleTestForSystemPage(UIStrings.NODES.MENU);
    });
  });

  describe('Tasks Link', () => {
    it('shows given permissions', async () => {
      givenPermissions({ [Permissions.TASKS.READ]: true });
      await runLinkVisiblityTestForSystemPage(UIStrings.TASKS.MENU);
    });

    it('does not show without permissions', async () => {
      givenPermissions({
        [Permissions.TASKS.READ]: false,
        [Permissions.SETTINGS.READ]: true  // Grant at least one permission so page renders
      });
      await runLinkNotVisibleTestForSystemPage(UIStrings.TASKS.MENU);
    });
  });

  function defaultExtState() {
    return {
      'astrum.extjs.capabilities.enabled': true,
      'astrum.react.capabilities.enabled': true,
      usertoken: { licenseValid: true }
    };
  }

  async function runLinkVisiblityTestForSystemPage(menu) {
    await runLinkVisiblityTest(ROUTE_NAMES.ADMIN.SYSTEM.DIRECTORY, 'System', menu);
  }

  async function runLinkNotVisibleTestForSystemPage(menu) {
    await runLinkNotVisibleTest(ROUTE_NAMES.ADMIN.SYSTEM.DIRECTORY, 'System', menu);
  }
});
