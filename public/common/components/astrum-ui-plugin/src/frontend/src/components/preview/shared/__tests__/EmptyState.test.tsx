

import React from 'react';
import { render, screen, fireEvent } from '@testing-library/react';
import { Theme } from '@radix-ui/themes';
import { Package, Plus } from 'lucide-react';
import { EmptyState } from '../EmptyState';

const renderWithTheme = (ui: React.ReactElement) => {
  return render(<Theme>{ui}</Theme>);
};

describe('EmptyState', () => {
  it('renders title and description', () => {
    renderWithTheme(
      <EmptyState
        icon={Package}
        title="No Repositories"
        description="Create your first repository to get started."
      />
    );

    expect(screen.getByTestId('empty-state')).toBeInTheDocument();
    expect(screen.getByText('No Repositories')).toBeInTheDocument();
    expect(screen.getByText('Create your first repository to get started.')).toBeInTheDocument();
  });

  it('renders primary action button', () => {
    const mockOnClick = jest.fn();

    renderWithTheme(
      <EmptyState
        icon={Package}
        title="No Repositories"
        description="Create your first repository."
        action={{
          label: 'Create Repository',
          onClick: mockOnClick,
          icon: Plus,
        }}
      />
    );

    const button = screen.getByRole('button', { name: /create repository/i });
    expect(button).toBeInTheDocument();

    fireEvent.click(button);
    expect(mockOnClick).toHaveBeenCalled();
  });

  it('renders secondary action link', () => {
    renderWithTheme(
      <EmptyState
        icon={Package}
        title="No Repositories"
        description="Create your first repository."
        secondaryAction={{
          label: 'Learn more',
          href: 'https://help.globalart.dev',
        }}
      />
    );

    const link = screen.getByRole('link', { name: /learn more/i });
    expect(link).toBeInTheDocument();
    expect(link).toHaveAttribute('href', 'https://help.globalart.dev');
    expect(link).toHaveAttribute('target', '_blank');
    expect(link).toHaveAttribute('rel', 'noopener noreferrer');
  });

  it('renders tip text', () => {
    renderWithTheme(
      <EmptyState
        icon={Package}
        title="No Repositories"
        description="Create your first repository."
        tip="Start with a proxy repository to cache Maven Central."
      />
    );

    expect(screen.getByText(/Start with a proxy repository/)).toBeInTheDocument();
  });

  it('applies small size class', () => {
    const { container } = renderWithTheme(
      <EmptyState
        icon={Package}
        title="No Items"
        description="No items found."
        size="small"
      />
    );

    expect(container.querySelector('.empty-state--small')).toBeInTheDocument();
  });

  it('applies large size class', () => {
    const { container } = renderWithTheme(
      <EmptyState
        icon={Package}
        title="No Items"
        description="No items found."
        size="large"
      />
    );

    expect(container.querySelector('.empty-state--large')).toBeInTheDocument();
  });

  it('applies custom className', () => {
    const { container } = renderWithTheme(
      <EmptyState
        icon={Package}
        title="No Items"
        description="No items found."
        className="custom-class"
      />
    );

    expect(container.querySelector('.custom-class')).toBeInTheDocument();
  });
});


