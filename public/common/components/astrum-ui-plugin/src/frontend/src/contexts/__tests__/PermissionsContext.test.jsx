

import React from 'react';
import { act, render, screen } from '@testing-library/react';
import { PermissionsProvider, usePermissions, usePermission } from '../PermissionsContext';
import * as extJsLoader from '../../utils/extJsLoader';

jest.mock('../../utils/extJsLoader');

const mockedExtJsLoader = extJsLoader;

// Simple test component to access permissions context
function TestConsumer({ permission }) {
  const { checkPermission, hasPermission } = usePermissions();

  return (
    <div>
      <span data-testid="check">{String(checkPermission(permission || 'astrum:*'))}</span>
      <span data-testid="has">{String(hasPermission(permission || 'astrum:*'))}</span>
    </div>
  );
}

describe('PermissionsContext', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    mockedExtJsLoader.isExtJSLoaded.mockReturnValue(false);
    mockedExtJsLoader.onExtJSLoad.mockImplementation(() => {});
    delete window.NX;
    delete window.Ext;
  });

  describe('PermissionsProvider', () => {
    it('renders children', () => {
      render(
        <PermissionsProvider>
          <div data-testid="child">Child Content</div>
        </PermissionsProvider>
      );

      expect(screen.getByTestId('child')).toHaveTextContent('Child Content');
    });

    it('returns false for permissions when ExtJS is not loaded', () => {
      render(
        <PermissionsProvider>
          <TestConsumer permission="astrum:repository-admin:*:*:read" />
        </PermissionsProvider>
      );

      expect(screen.getByTestId('check')).toHaveTextContent('false');
      expect(screen.getByTestId('has')).toHaveTextContent('false');
    });

    it('checks permissions via ExtJS when loaded', () => {
      mockedExtJsLoader.isExtJSLoaded.mockReturnValue(true);
      window.NX = {
        Permissions: {
          check: jest.fn().mockReturnValue(true),
        },
      };

      render(
        <PermissionsProvider>
          <TestConsumer permission="astrum:repository-admin:*:*:read" />
        </PermissionsProvider>
      );

      expect(screen.getByTestId('check')).toHaveTextContent('true');
      expect(window.NX.Permissions.check).toHaveBeenCalledWith('astrum:repository-admin:*:*:read');
    });

    it('returns false for denied permissions via ExtJS', () => {
      mockedExtJsLoader.isExtJSLoaded.mockReturnValue(true);
      window.NX = {
        Permissions: {
          check: jest.fn().mockReturnValue(false),
        },
      };

      render(
        <PermissionsProvider>
          <TestConsumer permission="astrum:settings:delete" />
        </PermissionsProvider>
      );

      expect(screen.getByTestId('check')).toHaveTextContent('false');
    });

    it('registers callback for ExtJS load', () => {
      render(
        <PermissionsProvider>
          <TestConsumer permission="astrum:*" />
        </PermissionsProvider>
      );

      expect(mockedExtJsLoader.onExtJSLoad).toHaveBeenCalledWith(expect.any(Function));
    });
  });

  describe('checkPermission', () => {
    it('delegates to NX.Permissions.check when ExtJS is loaded', () => {
      mockedExtJsLoader.isExtJSLoaded.mockReturnValue(true);
      window.NX = {
        Permissions: {
          check: jest.fn().mockImplementation((perm) => perm === 'astrum:admin'),
        },
      };

      render(
        <PermissionsProvider>
          <TestConsumer permission="astrum:admin" />
        </PermissionsProvider>
      );

      expect(screen.getByTestId('check')).toHaveTextContent('true');
    });

    it('returns false when NX.Permissions is not available', () => {
      mockedExtJsLoader.isExtJSLoaded.mockReturnValue(true);
      window.NX = {}; // No Permissions object

      render(
        <PermissionsProvider>
          <TestConsumer permission="astrum:*" />
        </PermissionsProvider>
      );

      expect(screen.getByTestId('check')).toHaveTextContent('false');
    });

    it('returns false when NX is undefined', () => {
      mockedExtJsLoader.isExtJSLoaded.mockReturnValue(true);
      // window.NX is undefined

      render(
        <PermissionsProvider>
          <TestConsumer permission="astrum:*" />
        </PermissionsProvider>
      );

      expect(screen.getByTestId('check')).toHaveTextContent('false');
    });
  });

  describe('hasPermission', () => {
    it('is an alias for checkPermission', () => {
      mockedExtJsLoader.isExtJSLoaded.mockReturnValue(true);
      window.NX = {
        Permissions: {
          check: jest.fn().mockReturnValue(true),
        },
      };

      render(
        <PermissionsProvider>
          <TestConsumer permission="astrum:*" />
        </PermissionsProvider>
      );

      expect(screen.getByTestId('check')).toHaveTextContent('true');
      expect(screen.getByTestId('has')).toHaveTextContent('true');
    });
  });

  describe('usePermissions', () => {
    it('returns default context values when used outside provider', () => {
      // Default context has restrictive defaults
      render(<TestConsumer permission="astrum:*" />);

      expect(screen.getByTestId('check')).toHaveTextContent('false');
      expect(screen.getByTestId('has')).toHaveTextContent('false');
    });
  });

  describe('usePermission hook', () => {
    function PermissionConsumer({ permission }) {
      const value = usePermission(permission);
      return <span data-testid="value">{String(value)}</span>;
    }

    function Wrapper({ children }) {
      return <PermissionsProvider>{children}</PermissionsProvider>;
    }

    it('returns false when ExtJS is not loaded', () => {
      render(<PermissionConsumer permission="astrum:admin" />, { wrapper: Wrapper });
      expect(screen.getByTestId('value')).toHaveTextContent('false');
    });

    it('returns the permission value from NX.Permissions.check when ExtJS is loaded', () => {
      mockedExtJsLoader.isExtJSLoaded.mockReturnValue(true);
      window.NX = { Permissions: { check: jest.fn().mockReturnValue(true) } };

      render(<PermissionConsumer permission="astrum:admin" />, { wrapper: Wrapper });

      expect(screen.getByTestId('value')).toHaveTextContent('true');
    });

    it('re-evaluates when the ExtJS changed event fires', async () => {
      mockedExtJsLoader.isExtJSLoaded.mockReturnValue(true);
      const checkMock = jest.fn().mockReturnValue(false);
      window.NX = { Permissions: { check: checkMock } };

      const changedListeners = [];
      window.Ext = {
        getApplication: jest.fn().mockReturnValue({
          getController: jest.fn().mockReturnValue({
            on: jest.fn((event, cb) => { if (event === 'changed') changedListeners.push(cb); }),
            un: jest.fn()
          })
        })
      };

      render(<PermissionConsumer permission="astrum:admin" />, { wrapper: Wrapper });
      expect(screen.getByTestId('value')).toHaveTextContent('false');

      // Simulate permissions becoming available and firing 'changed'
      checkMock.mockReturnValue(true);
      act(() => { changedListeners.forEach(fn => fn()); });

      expect(screen.getByTestId('value')).toHaveTextContent('true');
    });

    it('cleans up ExtJS listeners on unmount', () => {
      mockedExtJsLoader.isExtJSLoaded.mockReturnValue(true);
      window.NX = { Permissions: { check: jest.fn().mockReturnValue(false) } };

      const unMock = jest.fn();
      window.Ext = {
        getApplication: jest.fn().mockReturnValue({
          getController: jest.fn().mockReturnValue({ on: jest.fn(), un: unMock })
        })
      };

      const { unmount } = render(<PermissionConsumer permission="astrum:admin" />, { wrapper: Wrapper });
      unmount();

      expect(unMock).toHaveBeenCalled();
    });

    it('registers listeners and re-evaluates when ExtJS loads after component mount', async () => {
      // Component mounts while ExtJS is not yet loaded
      mockedExtJsLoader.isExtJSLoaded.mockReturnValue(false);
      // Collect ALL onExtJSLoad callbacks — both PermissionsProvider and usePermission register one
      const extJsLoadCallbacks = [];
      mockedExtJsLoader.onExtJSLoad.mockImplementation((cb) => { extJsLoadCallbacks.push(cb); });

      window.NX = { Permissions: { check: jest.fn().mockReturnValue(false) } };

      const changedListeners = [];
      window.Ext = {
        getApplication: jest.fn().mockReturnValue({
          getController: jest.fn().mockReturnValue({
            on: jest.fn((event, cb) => { if (event === 'changed') changedListeners.push(cb); }),
            un: jest.fn()
          })
        })
      };

      render(<PermissionConsumer permission="astrum:admin" />, { wrapper: Wrapper });
      expect(screen.getByTestId('value')).toHaveTextContent('false');

      // ExtJS finishes loading — fire all queued onExtJSLoad callbacks
      mockedExtJsLoader.isExtJSLoaded.mockReturnValue(true);
      act(() => { extJsLoadCallbacks.forEach(cb => cb()); });

      // Listeners are now registered; simulate a subsequent login event
      window.NX.Permissions.check.mockReturnValue(true);
      act(() => { changedListeners.forEach(fn => fn()); });

      expect(screen.getByTestId('value')).toHaveTextContent('true');
    });
  });

  describe('permission patterns', () => {
    it('handles wildcard permissions', () => {
      mockedExtJsLoader.isExtJSLoaded.mockReturnValue(true);
      window.NX = {
        Permissions: {
          check: jest.fn().mockImplementation((perm) => {
            return perm === 'astrum:*' || perm.startsWith('astrum:read');
          }),
        },
      };

      render(
        <PermissionsProvider>
          <TestConsumer permission="astrum:*" />
        </PermissionsProvider>
      );

      expect(screen.getByTestId('check')).toHaveTextContent('true');
    });

    it('handles repository-specific permissions', () => {
      mockedExtJsLoader.isExtJSLoaded.mockReturnValue(true);
      window.NX = {
        Permissions: {
          check: jest.fn().mockImplementation((perm) => {
            return perm === 'astrum:repository-admin:maven2:maven-central:read';
          }),
        },
      };

      render(
        <PermissionsProvider>
          <TestConsumer permission="astrum:repository-admin:maven2:maven-central:read" />
        </PermissionsProvider>
      );

      expect(screen.getByTestId('check')).toHaveTextContent('true');
    });

    it('handles multiple permission checks', () => {
      mockedExtJsLoader.isExtJSLoaded.mockReturnValue(true);
      const checkMock = jest.fn().mockImplementation((perm) => {
        return perm === 'astrum:admin' || perm === 'astrum:browse:read';
      });
      window.NX = {
        Permissions: {
          check: checkMock,
        },
      };

      render(
        <PermissionsProvider>
          <TestConsumer permission="astrum:admin" />
        </PermissionsProvider>
      );

      expect(screen.getByTestId('check')).toHaveTextContent('true');

      render(
        <PermissionsProvider>
          <TestConsumer permission="astrum:browse:read" />
        </PermissionsProvider>
      );

      expect(screen.getAllByTestId('check')[1]).toHaveTextContent('true');

      render(
        <PermissionsProvider>
          <TestConsumer permission="astrum:delete" />
        </PermissionsProvider>
      );

      expect(screen.getAllByTestId('check')[2]).toHaveTextContent('false');
    });
  });
});
