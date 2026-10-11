

import React from 'react';
import { render, screen } from '@testing-library/react';
import { Theme } from '@radix-ui/themes';
import SettingsCard from '../SettingsCard';
import { SettingCard } from '../types';

// Mock previewFeatureFlags — repositories is enabled, users is disabled
jest.mock('../../../config/featureFlags', () => ({
  isFeatureEnabled: (key: string) => key === 'repository.repositories',
}));

const renderWithTheme = (ui: React.ReactElement) => {
  return render(<Theme>{ui}</Theme>);
};

describe('SettingsCard', () => {
  const mockCard: SettingCard = {
    id: 'users',
    path: 'security/users',
    label: 'Users',
    description: 'Manage user accounts and access',
  };

  it('renders card with label and description', () => {
    renderWithTheme(<SettingsCard card={mockCard} />);

    expect(screen.getByText('Users')).toBeInTheDocument();
    expect(screen.getByText('Manage user accounts and access')).toBeInTheDocument();
  });

  it('renders a tile with the icon of the page and no button on wider screens', () => {
    const { container } = renderWithTheme(<SettingsCard card={mockCard} />);

    expect(container.querySelector('.settings-card--tile')).toBeInTheDocument();
    expect(container.querySelector('.settings-card__icon svg')).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'View' })).not.toBeInTheDocument();
  });

  it('shows a list row with a chevron instead of a tile on phones', () => {
    const originalMatchMedia = window.matchMedia;
    window.matchMedia = jest.fn((query: string) => ({
      matches: query.includes('639px'),
      media: query,
      addEventListener: jest.fn(),
      removeEventListener: jest.fn(),
    })) as unknown as typeof window.matchMedia;

    try {
      renderWithTheme(<SettingsCard card={mockCard} />);

      expect(document.querySelector('.settings-card--tile')).not.toBeInTheDocument();
      expect(screen.getByRole('link')).toHaveTextContent('Users');
    } finally {
      window.matchMedia = originalMatchMedia;
    }
  });

  it('renders as a link with correct href', () => {
    renderWithTheme(<SettingsCard card={mockCard} />);

    const link = screen.getByRole('link');
    expect(link).toHaveAttribute('href', '#preview/admin/security/users');
  });

  it('uses fullHash when provided', () => {
    const cardWithHash: SettingCard = {
      ...mockCard,
      fullHash: '#browse/welcome',
    };
    renderWithTheme(<SettingsCard card={cardWithHash} />);

    const link = screen.getByRole('link');
    expect(link).toHaveAttribute('href', '#browse/welcome');
  });

  it('marks the card as the settings card link', () => {
    const { container } = renderWithTheme(<SettingsCard card={mockCard} />);

    expect(container.querySelector('a.settings-card')).toBeInTheDocument();
  });

  it('does not truncate description text', () => {
    renderWithTheme(<SettingsCard card={mockCard} />);

    const description = screen.getByText('Manage user accounts and access');
    expect(description).not.toHaveStyle({ whiteSpace: 'nowrap' });
  });

  it('renders with searchTerms in card data', () => {
    const cardWithSearchTerms: SettingCard = {
      ...mockCard,
      searchTerms: ['accounts', 'people'],
    };

    renderWithTheme(<SettingsCard card={cardWithSearchTerms} />);

    // Card should still render normally (searchTerms don't affect visual display)
    expect(screen.getByText('Users')).toBeInTheDocument();
    expect(screen.getByText('Manage user accounts and access')).toBeInTheDocument();
  });

  it('renders card with long description without truncation', () => {
    const cardWithLongDescription: SettingCard = {
      ...mockCard,
      description: 'This is a very long description that should wrap naturally when it exceeds the available width',
    };

    renderWithTheme(<SettingsCard card={cardWithLongDescription} />);

    expect(screen.getByText(cardWithLongDescription.description)).toBeInTheDocument();
  });

  it('renders card with special characters in label and description', () => {
    const cardWithSpecialChars: SettingCard = {
      id: 'test',
      path: 'test/path',
      label: 'SSL/TLS & Certificates',
      description: 'Configure SSL/TLS certificates & security settings',
    };

    renderWithTheme(<SettingsCard card={cardWithSpecialChars} />);

    expect(screen.getByText('SSL/TLS & Certificates')).toBeInTheDocument();
    expect(screen.getByText('Configure SSL/TLS certificates & security settings')).toBeInTheDocument();
  });

  describe('Coming Soon Badge', () => {
    it('renders badge when featureKey is disabled', () => {
      const comingSoonCard: SettingCard = {
        ...mockCard,
        featureKey: 'security.users', // disabled in mock
      };
      renderWithTheme(<SettingsCard card={comingSoonCard} />);

      expect(screen.getByText('Coming Soon')).toBeInTheDocument();
    });

    it('does not render badge when featureKey is enabled', () => {
      const enabledCard: SettingCard = {
        id: 'repositories',
        path: 'repository/repositories',
        label: 'Repositories',
        description: 'Manage repository configurations',
        featureKey: 'repository.repositories', // enabled in mock
      };
      renderWithTheme(<SettingsCard card={enabledCard} />);

      expect(screen.queryByText('Coming Soon')).not.toBeInTheDocument();
    });

    it('does not render badge when featureKey is absent', () => {
      renderWithTheme(<SettingsCard card={mockCard} />);

      expect(screen.queryByText('Coming Soon')).not.toBeInTheDocument();
    });

    it('badge is visible to screen readers via text content', () => {
      const comingSoonCard: SettingCard = {
        ...mockCard,
        featureKey: 'security.users',
      };
      renderWithTheme(<SettingsCard card={comingSoonCard} />);

      expect(screen.getByText('Coming Soon')).toBeInTheDocument();
    });
  });
});
