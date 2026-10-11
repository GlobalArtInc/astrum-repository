

import React from 'react';
import { render, screen } from '@testing-library/react';
import { Theme } from '@radix-ui/themes';
import SettingsNotAvailablePage from '../SettingsNotAvailablePage';

const renderWithTheme = (ui: React.ReactElement) => {
  return render(<Theme>{ui}</Theme>);
};

describe('SettingsNotAvailablePage', () => {
  it('renders the heading', () => {
    renderWithTheme(<SettingsNotAvailablePage featureName="Users" />);

    expect(screen.getByRole('heading', { name: 'Not available in preview' })).toBeInTheDocument();
  });

  it('renders featureName in the message', () => {
    renderWithTheme(<SettingsNotAvailablePage featureName="Users" />);

    expect(screen.getByText(/Users is still being prepared for the Astrum One UI/)).toBeInTheDocument();
  });

  it('renders back link to settings hub', () => {
    renderWithTheme(<SettingsNotAvailablePage featureName="Users" />);

    const backLink = screen.getByRole('link', { name: /Back to Settings/ });
    expect(backLink).toHaveAttribute('href', '#admin/settings');
  });

  it('renders with different featureName', () => {
    renderWithTheme(<SettingsNotAvailablePage featureName="LDAP" />);

    expect(screen.getByText(/LDAP is still being prepared for the Astrum One UI/)).toBeInTheDocument();
  });

  it('instructs user to use classic admin panel', () => {
    renderWithTheme(<SettingsNotAvailablePage featureName="Realms" />);

    expect(screen.getByText(/classic UI administration panel/)).toBeInTheDocument();
  });
});
