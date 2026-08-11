

import React from 'react';
import { render, screen } from '@testing-library/react';
import { Theme } from '@radix-ui/themes';
import { LoadingState } from '../LoadingState';

const renderWithTheme = (ui: React.ReactElement) => {
  return render(<Theme>{ui}</Theme>);
};

describe('LoadingState', () => {
  it('renders without message', () => {
    renderWithTheme(<LoadingState />);

    expect(screen.getByTestId('loading-state')).toBeInTheDocument();
    expect(screen.getByRole('status')).toBeInTheDocument();
  });

  it('renders with message', () => {
    renderWithTheme(<LoadingState message="Loading repositories..." />);

    expect(screen.getByText('Loading repositories...')).toBeInTheDocument();
  });

  it('has aria-busy attribute', () => {
    renderWithTheme(<LoadingState />);

    expect(screen.getByRole('status')).toHaveAttribute('aria-busy', 'true');
  });

  it('has aria-live attribute', () => {
    renderWithTheme(<LoadingState />);

    expect(screen.getByRole('status')).toHaveAttribute('aria-live', 'polite');
  });

  it('applies small size class', () => {
    const { container } = renderWithTheme(<LoadingState size="small" />);

    expect(container.querySelector('.loading-state--small')).toBeInTheDocument();
  });

  it('applies large size class', () => {
    const { container } = renderWithTheme(<LoadingState size="large" />);

    expect(container.querySelector('.loading-state--large')).toBeInTheDocument();
  });

  it('applies inline class when inline prop is true', () => {
    const { container } = renderWithTheme(<LoadingState inline />);

    expect(container.querySelector('.loading-state--inline')).toBeInTheDocument();
  });

  it('applies custom className', () => {
    const { container } = renderWithTheme(
      <LoadingState className="custom-class" />
    );

    expect(container.querySelector('.custom-class')).toBeInTheDocument();
  });

  it('includes screen reader text', () => {
    renderWithTheme(<LoadingState message="Loading data" />);

    expect(screen.getByText('Loading: Loading data')).toBeInTheDocument();
  });
});


