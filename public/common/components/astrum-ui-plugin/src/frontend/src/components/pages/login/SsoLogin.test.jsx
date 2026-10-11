

import React from 'react';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { ExtJS } from '../../../interface/ExtJS';
import { useRouter } from '@uirouter/react';
import SsoLogin from './SsoLogin';

jest.mock('../../../interface/ExtJS', () => ({
  ExtJS: {
    useState: jest.fn(),
    state: jest.fn(),
  },
}));
jest.mock('@uirouter/react');

describe('SsoLogin', () => {
  const mockUseState = jest.fn();
  const mockState = jest.fn();
  const mockRouterParams = {};

  beforeEach(() => {
    ExtJS.useState = mockUseState;
    ExtJS.state = mockState;
    mockState.mockReturnValue({
      getValue: jest.fn()
    });

    useRouter.mockReturnValue({
      globals: {
        params: mockRouterParams
      }
    });

    Object.defineProperty(window, 'location', {
      value: {
        assign: jest.fn()
      },
      writable: true
    });

    mockRouterParams.returnTo = undefined;
    sessionStorage.clear();
    jest.clearAllMocks();
  });

  describe('visibility', () => {
    it('renders button when SAML is enabled', () => {
      mockUseState
        .mockReturnValueOnce(true)  // samlEnabled
        .mockReturnValueOnce(false) // oauth2Enabled
        .mockReturnValueOnce('/')   // contextPath
        .mockReturnValueOnce(false); // isCloud

      render(<SsoLogin />);
      expect(screen.getByRole('button', { name: /continue with sso/i })).toBeInTheDocument();
    });

    it('renders button when OAuth2 is enabled', () => {
      mockUseState
        .mockReturnValueOnce(false) // samlEnabled
        .mockReturnValueOnce(true)  // oauth2Enabled
        .mockReturnValueOnce('/')   // contextPath
        .mockReturnValueOnce(false); // isCloud

      render(<SsoLogin />);
      expect(screen.getByRole('button', { name: /continue with sso/i })).toBeInTheDocument();
    });

    it('renders button when both SAML and OAuth2 are enabled', () => {
      mockUseState
        .mockReturnValueOnce(true)  // samlEnabled
        .mockReturnValueOnce(true)  // oauth2Enabled
        .mockReturnValueOnce('/')   // contextPath
        .mockReturnValueOnce(false); // isCloud

      render(<SsoLogin />);
      expect(screen.getByRole('button', { name: /continue with sso/i })).toBeInTheDocument();
    });
  });

  describe('button properties', () => {
    beforeEach(() => {
      mockUseState
        .mockReturnValueOnce(true)  // samlEnabled
        .mockReturnValueOnce(false) // oauth2Enabled
        .mockReturnValueOnce('/')   // contextPath
        .mockReturnValueOnce(false); // isCloud
    });

    it('has correct attributes', () => {
      render(<SsoLogin />);
      const button = screen.getByRole('button');

      expect(button).toHaveAttribute('type', 'button');
      expect(button).toHaveAttribute('data-analytics-id', 'as-login-sso');
      expect(button).toHaveClass('login-submit');
    });

    it('displays correct text when not loading (self-hosted)', () => {
      render(<SsoLogin />);
      expect(screen.getByText('Continue with SSO')).toBeInTheDocument();
    });

    it('displays "Login" text in cloud environment', () => {
      mockUseState.mockReset();
      mockUseState
        .mockReturnValueOnce(true)  // samlEnabled
        .mockReturnValueOnce(false) // oauth2Enabled
        .mockReturnValueOnce('/')   // contextPath
        .mockReturnValueOnce(true); // isCloud

      render(<SsoLogin />);
      expect(screen.getByText('Login')).toBeInTheDocument();
    });
  });

  describe('SAML redirect', () => {
    it('redirects to /saml with empty hash parameter', async () => {
      mockUseState
        .mockReturnValueOnce(true)  // samlEnabled
        .mockReturnValueOnce(false) // oauth2Enabled
        .mockReturnValueOnce('/')   // contextPath
        .mockReturnValueOnce(false); // isCloud
      mockRouterParams.returnTo = undefined;

      render(<SsoLogin />);
      const button = screen.getByRole('button');

      fireEvent.click(button);

      await waitFor(() => {
        expect(window.location.assign).toHaveBeenCalledWith('/saml?');
      });
    });

    it('redirects to /saml with hash parameter', async () => {
      mockUseState
        .mockReturnValueOnce(true)  // samlEnabled
        .mockReturnValueOnce(false) // oauth2Enabled
        .mockReturnValueOnce('/')   // contextPath
        .mockReturnValueOnce(false); // isCloud
      mockRouterParams.returnTo = '#admin/repository/repositories';

      render(<SsoLogin />);
      const button = screen.getByRole('button');

      fireEvent.click(button);

      await waitFor(() => {
        expect(window.location.assign).toHaveBeenCalledWith(`/saml?hash=${encodeURIComponent(btoa('#preview/admin/repository/repositories'))}`);
      });
    });

    it('redirects to context path + /saml when context path is set', async () => {
      mockUseState
        .mockReturnValueOnce(true)      // samlEnabled
        .mockReturnValueOnce(false)     // oauth2Enabled
        .mockReturnValueOnce('/astrum')  // contextPath
        .mockReturnValueOnce(false);    // isCloud
      mockRouterParams.returnTo = undefined;

      render(<SsoLogin />);
      const button = screen.getByRole('button');

      fireEvent.click(button);

      await waitFor(() => {
        expect(window.location.assign).toHaveBeenCalledWith('/astrum/saml?');
      });
    });

    it('redirects to context path + /saml with hash parameter when context path is set', async () => {
      mockUseState
        .mockReturnValueOnce(true)      // samlEnabled
        .mockReturnValueOnce(false)     // oauth2Enabled
        .mockReturnValueOnce('/astrum')  // contextPath
        .mockReturnValueOnce(false);    // isCloud
      mockRouterParams.returnTo = '#admin/repository/repositories';

      render(<SsoLogin />);
      const button = screen.getByRole('button');

      fireEvent.click(button);

      await waitFor(() => {
        expect(window.location.assign).toHaveBeenCalledWith(`/astrum/saml?hash=${encodeURIComponent(btoa('#preview/admin/repository/repositories'))}`);
      });
    });
  });

  describe('OAuth2 redirect', () => {
    it('redirects to /oidc/login with empty hash parameter', async () => {
      mockUseState
        .mockReturnValueOnce(false) // samlEnabled
        .mockReturnValueOnce(true)  // oauth2Enabled
        .mockReturnValueOnce('/')   // contextPath
        .mockReturnValueOnce(false); // isCloud
      mockRouterParams.returnTo = undefined;

      render(<SsoLogin />);
      const button = screen.getByRole('button');

      fireEvent.click(button);

      await waitFor(() => {
        expect(window.location.assign).toHaveBeenCalledWith('/oidc/login?');
      });
    });

    it('redirects to /oidc/login with hash parameter', async () => {
      mockUseState
        .mockReturnValueOnce(false) // samlEnabled
        .mockReturnValueOnce(true)  // oauth2Enabled
        .mockReturnValueOnce('/')   // contextPath
        .mockReturnValueOnce(false); // isCloud
      mockRouterParams.returnTo = '#admin/repository/repositories';

      render(<SsoLogin />);
      const button = screen.getByRole('button');

      fireEvent.click(button);

      await waitFor(() => {
        expect(window.location.assign).toHaveBeenCalledWith(`/oidc/login?hash=${encodeURIComponent(btoa('#preview/admin/repository/repositories'))}`);
      });
    });

    it('redirects to context path + /oidc/login when context path is set', async () => {
      mockUseState
        .mockReturnValueOnce(false)     // samlEnabled
        .mockReturnValueOnce(true)      // oauth2Enabled
        .mockReturnValueOnce('/astrum')  // contextPath
        .mockReturnValueOnce(false);    // isCloud
      mockRouterParams.returnTo = undefined;

      render(<SsoLogin />);
      const button = screen.getByRole('button');

      fireEvent.click(button);

      await waitFor(() => {
        expect(window.location.assign).toHaveBeenCalledWith('/astrum/oidc/login?');
      });
    });

    it('redirects to context path + /oidc/login with hash parameter when context path is set', async () => {
      mockUseState
        .mockReturnValueOnce(false)     // samlEnabled
        .mockReturnValueOnce(true)      // oauth2Enabled
        .mockReturnValueOnce('/astrum')  // contextPath
        .mockReturnValueOnce(false);    // isCloud
      mockRouterParams.returnTo = '#admin/repository/repositories';

      render(<SsoLogin />);
      const button = screen.getByRole('button');

      fireEvent.click(button);

      await waitFor(() => {
        expect(window.location.assign).toHaveBeenCalledWith(`/astrum/oidc/login?hash=${encodeURIComponent(btoa('#preview/admin/repository/repositories'))}`);
      });
    });
  });

  describe('SAML priority', () => {
    it('prefers SAML when both SAML and OAuth2 are enabled', async () => {
      mockUseState
        .mockReturnValueOnce(true)  // samlEnabled
        .mockReturnValueOnce(true)  // oauth2Enabled
        .mockReturnValueOnce('/')   // contextPath
        .mockReturnValueOnce(false); // isCloud
      mockRouterParams.returnTo = undefined;

      render(<SsoLogin />);
      const button = screen.getByRole('button');

      fireEvent.click(button);

      await waitFor(() => {
        expect(window.location.assign).toHaveBeenCalledWith('/saml?');
      });
    });
  });

  describe('Astrum One return target', () => {
    it('sends no hash when there is no returnTo, since the application root opens Astrum One', async () => {
      mockUseState
        .mockReturnValueOnce(true)   // samlEnabled
        .mockReturnValueOnce(false)  // oauth2Enabled
        .mockReturnValueOnce('/')    // contextPath
        .mockReturnValueOnce(false); // isCloud
      mockRouterParams.returnTo = undefined;
      mockState.mockReturnValue({
        getValue: (key, def) => ({ defaultToPreviewUi: true, loggedInEnabled: true }[key] ?? def)
      });

      render(<SsoLogin />);
      fireEvent.click(screen.getByRole('button'));

      await waitFor(() => {
        expect(window.location.assign).toHaveBeenCalledWith('/saml?');
      });
    });

    it('clears a stale user_requested_legacy flag', async () => {
      sessionStorage.setItem('user_requested_legacy', 'true');
      mockUseState
        .mockReturnValueOnce(true)   // samlEnabled
        .mockReturnValueOnce(false)  // oauth2Enabled
        .mockReturnValueOnce('/')    // contextPath
        .mockReturnValueOnce(false); // isCloud
      mockRouterParams.returnTo = undefined;
      mockState.mockReturnValue({
        getValue: (key, def) => ({ defaultToPreviewUi: true, loggedInEnabled: true }[key] ?? def)
      });

      render(<SsoLogin />);
      fireEvent.click(screen.getByRole('button'));

      await waitFor(() => {
        expect(window.location.assign).toHaveBeenCalledWith('/saml?');
      });
      expect(sessionStorage.getItem('user_requested_legacy')).toBeNull();
    });

    it('opens a Classic dashboard returnTo as the Astrum One dashboard', async () => {
      mockUseState
        .mockReturnValueOnce(true)   // samlEnabled
        .mockReturnValueOnce(false)  // oauth2Enabled
        .mockReturnValueOnce('/')    // contextPath
        .mockReturnValueOnce(false); // isCloud
      mockRouterParams.returnTo = btoa('#browse/welcome');
      mockState.mockReturnValue({
        getValue: (key, def) => ({ defaultToPreviewUi: true, loggedInEnabled: true }[key] ?? def)
      });

      render(<SsoLogin />);
      fireEvent.click(screen.getByRole('button'));

      const expectedHash = encodeURIComponent(btoa('#preview/browse/welcome'));
      await waitFor(() => {
        expect(window.location.assign).toHaveBeenCalledWith(`/saml?hash=${expectedHash}`);
      });
    });

    it('opens a Classic deep-link returnTo as the matching Astrum One page', async () => {
      mockUseState
        .mockReturnValueOnce(true)   // samlEnabled
        .mockReturnValueOnce(false)  // oauth2Enabled
        .mockReturnValueOnce('/')    // contextPath
        .mockReturnValueOnce(false); // isCloud
      const deepLink = btoa('#admin/security/users');
      mockRouterParams.returnTo = deepLink;
      mockState.mockReturnValue({
        getValue: (key, def) => ({ defaultToPreviewUi: true, loggedInEnabled: true }[key] ?? def)
      });

      render(<SsoLogin />);
      fireEvent.click(screen.getByRole('button'));

      const expectedHash = encodeURIComponent(btoa('#preview/admin/security/users'));
      await waitFor(() => {
        expect(window.location.assign).toHaveBeenCalledWith(`/saml?hash=${expectedHash}`);
      });
    });

    it('honors a preview returnTo instead of overriding it', async () => {
      mockUseState
        .mockReturnValueOnce(true)   // samlEnabled
        .mockReturnValueOnce(false)  // oauth2Enabled
        .mockReturnValueOnce('/')    // contextPath
        .mockReturnValueOnce(false); // isCloud
      const previewReturnTo = btoa('#preview/admin/repository/repositories');
      mockRouterParams.returnTo = previewReturnTo;
      mockState.mockReturnValue({
        getValue: (key, def) => ({ defaultToPreviewUi: true, loggedInEnabled: true }[key] ?? def)
      });

      render(<SsoLogin />);
      fireEvent.click(screen.getByRole('button'));

      const expectedHash = encodeURIComponent(previewReturnTo);
      await waitFor(() => {
        expect(window.location.assign).toHaveBeenCalledWith(`/saml?hash=${expectedHash}`);
      });
    });

    it('sends no hash without a returnTo when loggedInEnabled is off', async () => {
      mockUseState
        .mockReturnValueOnce(true)   // samlEnabled
        .mockReturnValueOnce(false)  // oauth2Enabled
        .mockReturnValueOnce('/')    // contextPath
        .mockReturnValueOnce(false); // isCloud
      mockRouterParams.returnTo = undefined;
      mockState.mockReturnValue({
        getValue: (key, def) => ({ defaultToPreviewUi: true, loggedInEnabled: false }[key] ?? def)
      });

      render(<SsoLogin />);
      fireEvent.click(screen.getByRole('button'));

      await waitFor(() => {
        expect(window.location.assign).toHaveBeenCalledWith('/saml?');
      });
    });

    it('maps a raw Classic returnTo whatever defaultToPreviewUi says', async () => {
      mockUseState
        .mockReturnValueOnce(true)   // samlEnabled
        .mockReturnValueOnce(false)  // oauth2Enabled
        .mockReturnValueOnce('/')    // contextPath
        .mockReturnValueOnce(false); // isCloud
      mockRouterParams.returnTo = '#admin/repository/repositories';
      mockState.mockReturnValue({
        getValue: (key, def) => ({ defaultToPreviewUi: false, loggedInEnabled: true }[key] ?? def)
      });

      render(<SsoLogin />);
      fireEvent.click(screen.getByRole('button'));

      await waitFor(() => {
        expect(window.location.assign).toHaveBeenCalledWith(`/saml?hash=${encodeURIComponent(btoa('#preview/admin/repository/repositories'))}`);
      });
    });

  });

  describe('accessibility', () => {
    beforeEach(() => {
      mockUseState
        .mockReturnValueOnce(true)  // samlEnabled
        .mockReturnValueOnce(false) // oauth2Enabled
        .mockReturnValueOnce('/')   // contextPath
        .mockReturnValueOnce(false); // isCloud
    });

    it('is keyboard accessible', () => {
      render(<SsoLogin />);
      const button = screen.getByRole('button');

      // Check that the button can receive focus and has proper attributes
      expect(button).toHaveAttribute('type', 'button');
      expect(button).not.toHaveAttribute('tabindex', '-1');
    });
  });

  describe('analytics', () => {
    beforeEach(() => {
      mockUseState
        .mockReturnValueOnce(true)  // samlEnabled
        .mockReturnValueOnce(false) // oauth2Enabled
        .mockReturnValueOnce('/')   // contextPath
        .mockReturnValueOnce(false); // isCloud
    });

    it('has analytics attribute', () => {
      render(<SsoLogin />);
      const button = screen.getByRole('button');

      expect(button).toHaveAttribute('data-analytics-id', 'as-login-sso');
    });
  });


  describe('button behavior on click', () => {
    it('calls window.location.assign when clicked', () => {
      mockUseState
        .mockReturnValueOnce(true)  // samlEnabled
        .mockReturnValueOnce(false) // oauth2Enabled
        .mockReturnValueOnce('/')   // contextPath
        .mockReturnValueOnce(false); // isCloud

      render(<SsoLogin />);
      const button = screen.getByRole('button');

      fireEvent.click(button);

      expect(window.location.assign).toHaveBeenCalledWith('/saml?');
      expect(window.location.assign).toHaveBeenCalledTimes(1);
    });

    it('prevents multiple redirects on rapid clicks', () => {
      mockUseState
        .mockReturnValueOnce(true)  // samlEnabled
        .mockReturnValueOnce(false) // oauth2Enabled
        .mockReturnValueOnce('/')   // contextPath
        .mockReturnValueOnce(false); // isCloud

      render(<SsoLogin />);
      const button = screen.getByRole('button');

      // Simulate rapid clicks
      fireEvent.click(button);
      fireEvent.click(button);
      fireEvent.click(button);

      // Only the first click should trigger redirect (button gets disabled)
      expect(window.location.assign).toHaveBeenCalledTimes(1);
    });
  });

  describe('edge cases', () => {
    it('handles returnTo with special characters', async () => {
      mockUseState
        .mockReturnValueOnce(true)  // samlEnabled
        .mockReturnValueOnce(false) // oauth2Enabled
        .mockReturnValueOnce('/')   // contextPath
        .mockReturnValueOnce(false); // isCloud

      useRouter.mockReturnValue({
        globals: {
          params: {
            returnTo: '%23browse%2Fsearch%3Fq%3Dtest%2520package'
          }
        }
      });

      render(<SsoLogin />);
      const button = screen.getByRole('button');

      fireEvent.click(button);

      await waitFor(() => {
        expect(window.location.assign).toHaveBeenCalledWith('/saml?hash=%2523browse%252Fsearch%253Fq%253Dtest%252520package');
      });
    });
  });

  describe('autoFocus behavior', () => {
    beforeEach(() => {
      mockUseState
        .mockReturnValueOnce(true)  // samlEnabled
        .mockReturnValueOnce(false) // oauth2Enabled
        .mockReturnValueOnce('/')   // contextPath
        .mockReturnValueOnce(false); // isCloud
    });

    it('always focuses button on mount', async () => {
      render(<SsoLogin />);
      const button = screen.getByRole('button');

      // Wait for autofocus to take effect
      await waitFor(() => {
        expect(document.activeElement).toBe(button);
      }, { timeout: 1000 });
    });
  });
});
