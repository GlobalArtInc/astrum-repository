

import React from 'react';
import { render, screen, fireEvent } from '@testing-library/react';
import '@testing-library/jest-dom';
import { SettingsButton } from '../SettingsButton';

// Mock lucide-react
jest.mock('lucide-react', () => ({
  Loader2: ({ className }) => <span data-testid="spinner" className={className}>⟳</span>,
  Save: ({ className }) => <span data-testid="save-icon" className={className}>💾</span>,
}));

describe('SettingsButton', () => {
  it('renders button with children', () => {
    render(<SettingsButton>Click Me</SettingsButton>);
    
    expect(screen.getByRole('button')).toHaveTextContent('Click Me');
  });

  it('defaults to button type', () => {
    render(<SettingsButton>Test</SettingsButton>);
    
    expect(screen.getByRole('button')).toHaveAttribute('type', 'button');
  });

  it('supports submit type', () => {
    render(<SettingsButton type="submit">Submit</SettingsButton>);
    
    expect(screen.getByRole('button')).toHaveAttribute('type', 'submit');
  });

  it('calls onClick when clicked', () => {
    const onClick = jest.fn();
    render(<SettingsButton onClick={onClick}>Click</SettingsButton>);
    
    fireEvent.click(screen.getByRole('button'));
    
    expect(onClick).toHaveBeenCalledTimes(1);
  });

  it('is disabled when disabled prop is true', () => {
    render(<SettingsButton disabled>Disabled</SettingsButton>);
    
    expect(screen.getByRole('button')).toBeDisabled();
  });

  it('is disabled when loading prop is true', () => {
    render(<SettingsButton loading>Loading</SettingsButton>);
    
    expect(screen.getByRole('button')).toBeDisabled();
  });

  it('shows spinner when loading', () => {
    render(<SettingsButton loading>Loading</SettingsButton>);
    
    expect(screen.getByTestId('spinner')).toBeInTheDocument();
  });

  it('hides spinner when not loading', () => {
    render(<SettingsButton>Not Loading</SettingsButton>);
    
    expect(screen.queryByTestId('spinner')).not.toBeInTheDocument();
  });

  it('renders icon when provided', () => {
    const SaveIcon = () => <span data-testid="save-icon">💾</span>;
    render(<SettingsButton icon={SaveIcon}>Save</SettingsButton>);
    
    expect(screen.getByTestId('save-icon')).toBeInTheDocument();
  });

  it('hides icon when loading (shows spinner instead)', () => {
    const SaveIcon = () => <span data-testid="save-icon">💾</span>;
    render(<SettingsButton icon={SaveIcon} loading>Save</SettingsButton>);
    
    expect(screen.queryByTestId('save-icon')).not.toBeInTheDocument();
    expect(screen.getByTestId('spinner')).toBeInTheDocument();
  });

  it('applies primary variant class', () => {
    render(<SettingsButton variant="primary">Primary</SettingsButton>);
    
    expect(screen.getByRole('button')).toHaveClass('settings-button--primary');
  });

  it('applies secondary variant class by default', () => {
    render(<SettingsButton>Secondary</SettingsButton>);
    
    expect(screen.getByRole('button')).toHaveClass('settings-button--secondary');
  });

  it('applies danger variant class', () => {
    render(<SettingsButton variant="danger">Delete</SettingsButton>);
    
    expect(screen.getByRole('button')).toHaveClass('settings-button--danger');
  });

  it('applies ghost variant class', () => {
    render(<SettingsButton variant="ghost">Ghost</SettingsButton>);
    
    expect(screen.getByRole('button')).toHaveClass('settings-button--ghost');
  });

  it('applies small size class', () => {
    render(<SettingsButton size="small">Small</SettingsButton>);
    
    expect(screen.getByRole('button')).toHaveClass('settings-button--small');
  });

  it('applies medium size class by default', () => {
    render(<SettingsButton>Medium</SettingsButton>);
    
    expect(screen.getByRole('button')).toHaveClass('settings-button--medium');
  });

  it('applies large size class', () => {
    render(<SettingsButton size="large">Large</SettingsButton>);
    
    expect(screen.getByRole('button')).toHaveClass('settings-button--large');
  });

  it('applies custom className', () => {
    render(<SettingsButton className="custom">Custom</SettingsButton>);
    
    expect(screen.getByRole('button')).toHaveClass('custom');
  });

  it('does not call onClick when disabled', () => {
    const onClick = jest.fn();
    render(<SettingsButton onClick={onClick} disabled>Click</SettingsButton>);
    
    fireEvent.click(screen.getByRole('button'));
    
    expect(onClick).not.toHaveBeenCalled();
  });
});

