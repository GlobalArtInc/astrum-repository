

import React from 'react';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { Theme } from '@radix-ui/themes';
import { ToastProvider } from '../../../../shared';

import { UserAccountPage } from '../UserAccountPage';

// Mock ExtJS - note: jest.mock is hoisted, so we use a factory function
jest.mock('../../../../../../interface/ExtJS', () => ({
  ExtJS: {
    useUser: jest.fn(),
  },
}));

const mockVisibleRoutes = new Set(['preview.user.usertoken', 'preview.user.nugetapitoken']);

jest.mock('../../../../shared/Navigation/useRouteVisibility', () => ({
  useRouteVisibility: (name: string) => mockVisibleRoutes.has(name),
}));

// Mock restClient
jest.mock('../../../../../../interface/api', () => ({
  restClient: {
    get: jest.fn(),
    put: jest.fn(),
  },
  ENDPOINTS: {
    USER_ACCOUNT: '/service/rest/internal/ui/user',
  },
}));

const mockApiUser = {
  userId: 'testuser',
  firstName: 'Test',
  lastName: 'User',
  email: 'test@example.com',
  external: false,
};

const mockExtJsUser = {
  id: 'testuser',
  authenticated: true,
  administrator: true,
  authenticatedRealms: ['AstrumAuthenticatingRealm'],
};

// Wrapper component for Radix Theme
function TestWrapper({ children }: { children: React.ReactNode }) {
  return <Theme><ToastProvider>{children}</ToastProvider></Theme>;
}

describe('UserAccountPage', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    const { ExtJS } = require('../../../../../../interface/ExtJS');
    ExtJS.useUser.mockReturnValue(mockExtJsUser);

    const { restClient } = require('../../../../../../interface/api');
    restClient.get.mockResolvedValue(mockApiUser);
  });

  it('renders the page header', async () => {
    render(<UserAccountPage />, { wrapper: TestWrapper });

    await waitFor(() => {
      expect(screen.getByText('User Account')).toBeInTheDocument();
    });

    expect(screen.getByText('Manage your account settings')).toBeInTheDocument();
  });

  it('fetches user account data from the internal API on mount', async () => {
    const { restClient, ENDPOINTS } = require('../../../../../../interface/api');

    render(<UserAccountPage />, { wrapper: TestWrapper });

    await waitFor(() => {
      expect(restClient.get).toHaveBeenCalledWith(ENDPOINTS.USER_ACCOUNT);
    });
  });

  it('lets a local user edit the name and email from the API', async () => {
    render(<UserAccountPage />, { wrapper: TestWrapper });

    expect(await screen.findByLabelText(/First name/)).toHaveValue('Test');
    expect(screen.getByLabelText(/Last name/)).toHaveValue('User');
    expect(screen.getByLabelText(/^Email/)).toHaveValue('test@example.com');
    expect(screen.getByText('Local')).toBeInTheDocument();
  });

  it('saves the changed name and email', async () => {
    const { restClient } = require('../../../../../../interface/api');
    restClient.put.mockResolvedValue(undefined);
    render(<UserAccountPage />, { wrapper: TestWrapper });

    const lastName = await screen.findByLabelText(/Last name/);
    expect(screen.getByTestId('account-details-submit')).toBeDisabled();
    fireEvent.change(lastName, { target: { value: 'Tester' } });
    fireEvent.click(screen.getByTestId('account-details-submit'));

    await waitFor(() => {
      expect(restClient.put).toHaveBeenCalledWith('/service/rest/internal/ui/user', {
        userId: 'testuser',
        firstName: 'Test',
        lastName: 'Tester',
        email: 'test@example.com',
        external: false,
      });
    });
  });

  it('shows the details of an external user read-only', async () => {
    const { restClient } = require('../../../../../../interface/api');
    restClient.get.mockResolvedValue({ ...mockApiUser, external: true });

    render(<UserAccountPage />, { wrapper: TestWrapper });

    await waitFor(() => {
      expect(screen.getByText('Test User')).toBeInTheDocument();
    });
    expect(screen.getByText('test@example.com')).toBeInTheDocument();
    expect(screen.getByText('External')).toBeInTheDocument();
    expect(screen.queryByLabelText(/First name/)).not.toBeInTheDocument();
  });

  it('shows "Not set" for an external user without name and email', async () => {
    const { restClient } = require('../../../../../../interface/api');
    restClient.get.mockResolvedValue({ ...mockApiUser, firstName: '', lastName: '', email: '', external: true });

    render(<UserAccountPage />, { wrapper: TestWrapper });

    await waitFor(() => {
      expect(screen.getAllByText('Not set')).toHaveLength(2);
    });
  });

  it('hides password form when API fetch fails', async () => {
    const { restClient } = require('../../../../../../interface/api');
    restClient.get.mockRejectedValue(new Error('Network error'));

    render(<UserAccountPage />, { wrapper: TestWrapper });

    await waitFor(() => {
      expect(screen.getByText(/Failed to load account information/)).toBeInTheDocument();
    });

    expect(screen.queryByRole('heading', { name: 'Change Password' })).not.toBeInTheDocument();
  });

  it('shows password change form for internal (non-external) users', async () => {
    render(<UserAccountPage />, { wrapper: TestWrapper });

    await waitFor(() => {
      expect(screen.getByRole('heading', { name: 'Change Password' })).toBeInTheDocument();
    });

    expect(screen.getByLabelText(/Current Password/)).toBeInTheDocument();
    expect(screen.getByTestId('password-newPassword')).toBeInTheDocument();
    expect(screen.getByTestId('password-confirmPassword')).toBeInTheDocument();
  });

  it('validates password length', async () => {
    render(<UserAccountPage />, { wrapper: TestWrapper });

    await waitFor(() => {
      expect(screen.getByTestId('password-newPassword')).toBeInTheDocument();
    });

    const newPasswordInput = screen.getByTestId('password-newPassword');
    fireEvent.change(newPasswordInput, { target: { value: 'short' } });
    fireEvent.blur(newPasswordInput);

    await waitFor(() => {
      const errorEl = document.getElementById('settings-error-newPassword');
      expect(errorEl).toBeInTheDocument();
      expect(errorEl).toHaveTextContent(/at least.*characters/i);
    });
  });

  it('validates password confirmation', async () => {
    render(<UserAccountPage />, { wrapper: TestWrapper });

    await waitFor(() => {
      expect(screen.getByTestId('password-newPassword')).toBeInTheDocument();
    });

    const newPasswordInput = screen.getByTestId('password-newPassword');
    const confirmPasswordInput = screen.getByTestId('password-confirmPassword');

    fireEvent.change(newPasswordInput, { target: { value: 'password123' } });
    fireEvent.change(confirmPasswordInput, { target: { value: 'different123' } });
    fireEvent.blur(confirmPasswordInput);

    await waitFor(() => {
      expect(screen.getByText(/passwords.*match/i)).toBeInTheDocument();
    });
  });

  it('shows external user message when API returns external=true', async () => {
    const { restClient } = require('../../../../../../interface/api');
    restClient.get.mockResolvedValue({ ...mockApiUser, external: true });

    render(<UserAccountPage />, { wrapper: TestWrapper });

    await waitFor(() => {
      expect(screen.getByText(/Your account is managed externally/)).toBeInTheDocument();
    });
  });

  it('shows warning when not logged in (no ExtJS user)', async () => {
    const { ExtJS } = require('../../../../../../interface/ExtJS');
    ExtJS.useUser.mockReturnValue(null);

    render(<UserAccountPage />, { wrapper: TestWrapper });

    await waitFor(() => {
      expect(screen.getByText(/must be logged in/i)).toBeInTheDocument();
    });
  });

  it('clears form on discard', async () => {
    render(<UserAccountPage />, { wrapper: TestWrapper });

    await waitFor(() => {
      expect(screen.getByLabelText(/Current Password/)).toBeInTheDocument();
    });

    const discardButton = screen.getByTestId('change-password-cancel');
    expect(discardButton).toBeDisabled();

    const currentPasswordInput = screen.getByLabelText(/Current Password/);
    fireEvent.change(currentPasswordInput, { target: { value: 'oldpassword' } });

    expect(discardButton).toBeEnabled();
    fireEvent.click(discardButton);

    expect(currentPasswordInput).toHaveValue('');
    expect(discardButton).toBeDisabled();
  });

  describe('Security section (spec xhpg)', () => {
    it('renders Security section with User Token and NuGet API Key links', async () => {
      render(<UserAccountPage />, { wrapper: TestWrapper });

      await waitFor(() => {
        expect(screen.getByTestId('security-section')).toBeInTheDocument();
      });

      expect(screen.getByTestId('user-token-link-card')).toBeInTheDocument();
      expect(screen.getByTestId('nuget-key-link-card')).toBeInTheDocument();
    });

    it('leaves out the links to pages the server does not offer', async () => {
      mockVisibleRoutes.delete('preview.user.usertoken');
      render(<UserAccountPage />, { wrapper: TestWrapper });

      await waitFor(() => {
        expect(screen.getByTestId('nuget-key-link-card')).toBeInTheDocument();
      });
      expect(screen.queryByTestId('user-token-link-card')).not.toBeInTheDocument();
      expect(screen.queryByText('Pro')).not.toBeInTheDocument();
      mockVisibleRoutes.add('preview.user.usertoken');
    });

    it('leaves out the Security section when no such page is offered', async () => {
      mockVisibleRoutes.clear();
      render(<UserAccountPage />, { wrapper: TestWrapper });

      await waitFor(() => {
        expect(screen.getByText('Account Information')).toBeInTheDocument();
      });
      expect(screen.queryByTestId('security-section')).not.toBeInTheDocument();
      mockVisibleRoutes.add('preview.user.usertoken');
      mockVisibleRoutes.add('preview.user.nugetapitoken');
    });

    it('Manage User Token button navigates to user token hash', async () => {
      render(<UserAccountPage />, { wrapper: TestWrapper });

      await waitFor(() => {
        expect(screen.getByTestId('manage-user-token-btn')).toBeInTheDocument();
      });

      fireEvent.click(screen.getByTestId('manage-user-token-btn'));
      expect(window.location.hash).toBe('#user/usertoken');
    });

    it('Manage NuGet API Key button navigates to nuget api token hash', async () => {
      render(<UserAccountPage />, { wrapper: TestWrapper });

      await waitFor(() => {
        expect(screen.getByTestId('manage-nuget-key-btn')).toBeInTheDocument();
      });

      fireEvent.click(screen.getByTestId('manage-nuget-key-btn'));
      expect(window.location.hash).toBe('#user/nugetapitoken');
    });
  });
});
