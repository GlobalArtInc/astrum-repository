

import React from 'react';
import { render, screen } from '@testing-library/react';
import { Theme } from '@radix-ui/themes';
import {
  FirewallNotSupportedEmptyState,
  FirewallNotSupportedBadge,
} from '../FirewallNotSupportedEmptyState';

function renderWithTheme(ui: React.ReactElement) {
  return render(<Theme>{ui}</Theme>);
}

describe('FirewallNotSupportedEmptyState', () => {
  describe('context=cell (default)', () => {
    it('renders inline AlertCircle + Not supported text', () => {
      renderWithTheme(<FirewallNotSupportedEmptyState format="terraform" />);
      expect(screen.getByTestId('firewall-not-supported-badge')).toBeInTheDocument();
      expect(screen.getByText('Not supported')).toBeInTheDocument();
    });

    it('has correct aria-label for the format', () => {
      renderWithTheme(<FirewallNotSupportedEmptyState format="helm" />);
      expect(
        screen.getByLabelText('Firewall: Not supported for helm')
      ).toBeInTheDocument();
    });
  });

  describe('context=tab', () => {
    it('renders the full empty state with ShieldOff heading and body', () => {
      renderWithTheme(<FirewallNotSupportedEmptyState format="terraform" context="tab" />);
      expect(screen.getByTestId('firewall-not-supported-empty-state')).toBeInTheDocument();
      expect(screen.getByText('Firewall Not Available')).toBeInTheDocument();
      expect(screen.getByText(/does not support the terraform format/i)).toBeInTheDocument();
    });

    it('renders Learn about supported formats link', () => {
      renderWithTheme(<FirewallNotSupportedEmptyState format="apt" context="tab" />);
      const link = screen.getByRole('link', { name: /learn about supported formats/i });
      expect(link).toBeInTheDocument();
      expect(link).toHaveAttribute('href', 'https://help.globalart.dev/en/repository-firewall.html');
    });
  });
});

describe('FirewallNotSupportedBadge', () => {
  it('renders Not supported badge for the given format', () => {
    renderWithTheme(<FirewallNotSupportedBadge format="swift" />);
    expect(screen.getByTestId('firewall-not-supported-badge')).toBeInTheDocument();
    expect(screen.getByText('Not supported')).toBeInTheDocument();
  });

  it('has correct aria-label', () => {
    renderWithTheme(<FirewallNotSupportedBadge format="gitlfs" />);
    expect(
      screen.getByLabelText('Firewall: Not supported for gitlfs')
    ).toBeInTheDocument();
  });
});
