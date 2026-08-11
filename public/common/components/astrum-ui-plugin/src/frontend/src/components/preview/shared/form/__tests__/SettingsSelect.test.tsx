

import React from 'react';
import { render, screen } from '@testing-library/react';
import '@testing-library/jest-dom';
import { SettingsSelect } from '../SettingsSelect';

// Mock Radix UI Select to avoid portal issues in tests
jest.mock('@radix-ui/themes', () => {
  const actual = jest.requireActual('@radix-ui/themes');
  return {
    ...actual,
    Select: {
      Root: ({ children, value, onValueChange, disabled }: any) => (
        <div data-testid="select-root" data-value={value} data-disabled={disabled}>
          {children}
        </div>
      ),
      Trigger: ({ children, id, placeholder, className, ...props }: any) => (
        <button
          id={id}
          role="combobox"
          className={className}
          disabled={props.disabled}
          aria-invalid={props['aria-invalid']}
          {...props}
        >
          {children || placeholder}
        </button>
      ),
      Content: ({ children }: any) => <div>{children}</div>,
      Item: ({ children, value }: any) => <div data-value={value}>{children}</div>,
    },
  };
});

describe('SettingsSelect', () => {
  const defaultOptions = [
    { value: 'option1', label: 'Option 1' },
    { value: 'option2', label: 'Option 2' },
    { value: 'option3', label: 'Option 3' },
  ];

  const defaultProps = {
    name: 'test-select',
    label: 'Test Select',
    value: '',
    onChange: jest.fn(),
    options: defaultOptions,
  };

  beforeEach(() => {
    jest.clearAllMocks();
  });

  describe('rendering', () => {
    it('renders with label', () => {
      render(<SettingsSelect {...defaultProps} />);
      expect(screen.getByText('Test Select')).toBeInTheDocument();
    });

    it('renders select trigger', () => {
      render(<SettingsSelect {...defaultProps} />);
      expect(screen.getByRole('combobox')).toBeInTheDocument();
    });

    it('shows placeholder when no value is selected', () => {
      render(<SettingsSelect {...defaultProps} placeholder="Select an option" />);
      expect(screen.getByText('Select an option')).toBeInTheDocument();
    });

    it('renders with help text', () => {
      render(<SettingsSelect {...defaultProps} helpText="This is helpful" />);
      expect(screen.getByText('This is helpful')).toBeInTheDocument();
    });

    it('renders with error message', () => {
      render(<SettingsSelect {...defaultProps} error="Selection is required" />);
      expect(screen.getByText('Selection is required')).toBeInTheDocument();
    });

    it('hides help text when error is present', () => {
      render(
        <SettingsSelect
          {...defaultProps}
          helpText="Help text"
          error="Error message"
        />
      );
      expect(screen.queryByText('Help text')).not.toBeInTheDocument();
      expect(screen.getByText('Error message')).toBeInTheDocument();
    });

    it('shows required indicator when required', () => {
      render(<SettingsSelect {...defaultProps} required />);
      expect(screen.getByText('*')).toBeInTheDocument();
    });

    it('applies error styling when error is present', () => {
      const { container } = render(
        <SettingsSelect {...defaultProps} error="Error" />
      );
      expect(container.querySelector('.settings-select--error')).toBeInTheDocument();
    });

    it('applies custom className', () => {
      const { container } = render(
        <SettingsSelect {...defaultProps} className="custom-class" />
      );
      expect(container.querySelector('.custom-class')).toBeInTheDocument();
    });
  });

  describe('accessibility', () => {
    it('has combobox role', () => {
      render(<SettingsSelect {...defaultProps} />);
      expect(screen.getByRole('combobox')).toBeInTheDocument();
    });

    it('sets aria-invalid when error is present', () => {
      render(<SettingsSelect {...defaultProps} error="Error" />);
      expect(screen.getByRole('combobox')).toHaveAttribute('aria-invalid', 'true');
    });

    it('sets aria-required when required (ASTRUM-53625 A7)', () => {
      render(<SettingsSelect {...defaultProps} required />);
      expect(screen.getByRole('combobox')).toHaveAttribute('aria-required', 'true');
    });

    it('sets aria-required to false when not required (ASTRUM-53625 A7)', () => {
      render(<SettingsSelect {...defaultProps} />);
      expect(screen.getByRole('combobox')).toHaveAttribute('aria-required', 'false');
    });

    it('can be focused', () => {
      render(<SettingsSelect {...defaultProps} />);
      const select = screen.getByRole('combobox');
      select.focus();
      expect(select).toHaveFocus();
    });
  });
});

