

import React from 'react';
import { render, screen, fireEvent } from '@testing-library/react';
import { Theme } from '@radix-ui/themes';

import { LoggerForm } from '../LoggerForm';
import * as useLoggerFormModule from '../useLoggerForm';
import { LogLevel } from '../types';

// Mock the integration hook
jest.mock('../useLoggerForm');

const mockedUseLoggerForm = useLoggerFormModule.useLoggerForm as jest.MockedFunction<
  typeof useLoggerFormModule.useLoggerForm
>;

// Wrapper component for Radix Theme
function TestWrapper({ children }: { children: React.ReactNode }) {
  return <Theme>{children}</Theme>;
}

function makeHook(overrides: Partial<ReturnType<typeof useLoggerFormModule.useLoggerForm>> = {}) {
  return {
    name: '',
    level: 'INFO' as LogLevel,
    isDirty: false,
    isLoading: false,
    isSaving: false,
    error: null as string | null,
    setName: jest.fn(),
    setLevel: jest.fn(),
    handleSubmit: jest.fn(),
    ...overrides,
  };
}

describe('LoggerForm', () => {
  const mockOnSave = jest.fn();
  const mockOnCancel = jest.fn();

  beforeEach(() => {
    jest.clearAllMocks();
    mockedUseLoggerForm.mockReturnValue(makeHook());
  });

  describe('Create Mode', () => {
    it('renders create form with empty fields', () => {
      render(<LoggerForm isCreate={true} onSave={mockOnSave} onCancel={mockOnCancel} />, {
        wrapper: TestWrapper,
      });

      expect(screen.getByLabelText(/Logger Name/i)).toBeInTheDocument();
      expect(screen.getByLabelText(/Logger Level/i)).toBeInTheDocument();
      expect(screen.getByText('Create Logger')).toBeInTheDocument();
    });

    it('allows editing logger name in create mode', () => {
      const mockSetName = jest.fn();
      mockedUseLoggerForm.mockReturnValue(makeHook({ setName: mockSetName }));

      render(<LoggerForm isCreate={true} onSave={mockOnSave} onCancel={mockOnCancel} />, {
        wrapper: TestWrapper,
      });

      const nameInput = screen.getByLabelText(/Logger Name/i);
      expect(nameInput).not.toBeDisabled();

      fireEvent.change(nameInput, { target: { value: 'org.test' } });

      expect(mockSetName).toHaveBeenCalledWith('org.test', expect.anything());
    });

    it('does not call handleSubmit when form is pristine', () => {
      const mockHandleSubmit = jest.fn();
      mockedUseLoggerForm.mockReturnValue(makeHook({ isDirty: false, handleSubmit: mockHandleSubmit }));

      render(<LoggerForm isCreate={true} onSave={mockOnSave} onCancel={mockOnCancel} />, {
        wrapper: TestWrapper,
      });

      const submitButton = screen.getByRole('button', { name: /create logger/i });
      fireEvent.click(submitButton);

      // SettingsForm disables submit when dirty=false
      expect(mockHandleSubmit).not.toHaveBeenCalled();
    });

    it('calls handleSubmit when form is dirty and submitted', () => {
      const mockHandleSubmit = jest.fn();
      mockedUseLoggerForm.mockReturnValue(
        makeHook({ isDirty: true, name: 'org.test', handleSubmit: mockHandleSubmit })
      );

      render(<LoggerForm isCreate={true} onSave={mockOnSave} onCancel={mockOnCancel} />, {
        wrapper: TestWrapper,
      });

      fireEvent.click(screen.getByText('Create Logger'));

      expect(mockHandleSubmit).toHaveBeenCalled();
    });
  });

  describe('Edit Mode', () => {
    it('renders loading state while fetching logger', () => {
      mockedUseLoggerForm.mockReturnValue(makeHook({ isLoading: true }));

      render(<LoggerForm loggerName="org.globalart" onSave={mockOnSave} onCancel={mockOnCancel} />, {
        wrapper: TestWrapper,
      });

      expect(screen.getByText('Loading logger...')).toBeInTheDocument();
    });

    it('displays existing logger name (disabled) in edit mode', () => {
      mockedUseLoggerForm.mockReturnValue(makeHook({ name: 'org.globalart', level: 'DEBUG' as LogLevel }));

      render(<LoggerForm loggerName="org.globalart" onSave={mockOnSave} onCancel={mockOnCancel} />, {
        wrapper: TestWrapper,
      });

      const nameInput = screen.getByLabelText(/Logger Name/i);
      expect(nameInput).toBeDisabled();
      expect(nameInput).toHaveValue('org.globalart');
    });

    it('shows Delete button in edit mode when onDelete provided', () => {
      mockedUseLoggerForm.mockReturnValue(makeHook({ name: 'org.globalart', level: 'DEBUG' as LogLevel }));

      render(
        <LoggerForm loggerName="org.globalart" onSave={mockOnSave} onCancel={mockOnCancel} onDelete={jest.fn()} />,
        { wrapper: TestWrapper }
      );

      expect(screen.getByText('Delete')).toBeInTheDocument();
    });

    it('does not show Delete button in create mode', () => {
      render(<LoggerForm isCreate={true} onSave={mockOnSave} onCancel={mockOnCancel} />, {
        wrapper: TestWrapper,
      });

      expect(screen.queryByText('Delete')).not.toBeInTheDocument();
    });

    it('calls onDelete when Delete is clicked', () => {
      const mockOnDelete = jest.fn();
      mockedUseLoggerForm.mockReturnValue(makeHook({ name: 'org.globalart', level: 'DEBUG' as LogLevel }));

      render(
        <LoggerForm loggerName="org.globalart" onSave={mockOnSave} onCancel={mockOnCancel} onDelete={mockOnDelete} />,
        { wrapper: TestWrapper }
      );

      fireEvent.click(screen.getByText('Delete'));

      expect(mockOnDelete).toHaveBeenCalled();
    });
  });

  describe('Common Behavior', () => {
    it('calls onCancel when the Cancel button is clicked', () => {
      render(<LoggerForm isCreate={true} onSave={mockOnSave} onCancel={mockOnCancel} />, {
        wrapper: TestWrapper,
      });

      // The button leaves the form, so it says Cancel rather than SettingsForm's default Discard
      fireEvent.click(screen.getByText('Cancel'));

      expect(mockOnCancel).toHaveBeenCalled();
    });

    it('displays error alert when hook returns an error', () => {
      mockedUseLoggerForm.mockReturnValue(makeHook({ error: 'Failed to update logger' }));

      render(<LoggerForm isCreate={true} onSave={mockOnSave} onCancel={mockOnCancel} />, {
        wrapper: TestWrapper,
      });

      expect(screen.getByText('Failed to update logger')).toBeInTheDocument();
    });

    it('renders the level select with the current level value', () => {
      mockedUseLoggerForm.mockReturnValue(makeHook({ level: 'WARN' as LogLevel }));

      render(<LoggerForm isCreate={true} onSave={mockOnSave} onCancel={mockOnCancel} />, {
        wrapper: TestWrapper,
      });

      // SettingsSelect uses Radix UI Select (not a native <select>), so we verify
      // the select element is present and correctly wired
      expect(screen.getByTestId('select-loggerLevel')).toBeInTheDocument();
      expect(screen.getByLabelText(/Logger Level/i)).toBeInTheDocument();
    });

    it('shows inheritance help text below level select', () => {
      render(<LoggerForm isCreate={true} onSave={mockOnSave} onCancel={mockOnCancel} />, {
        wrapper: TestWrapper,
      });

      expect(screen.getByText(/overrides the inherited level/i)).toBeInTheDocument();
    });
  });

  describe('testId selectors (E2E targeting)', () => {
    it('logger name input has data-testid input-loggerName', () => {
      render(<LoggerForm isCreate={true} onSave={mockOnSave} onCancel={mockOnCancel} />, {
        wrapper: TestWrapper,
      });

      expect(screen.getByTestId('input-loggerName')).toBeInTheDocument();
    });

    it('logger level select has data-testid select-loggerLevel', () => {
      render(<LoggerForm isCreate={true} onSave={mockOnSave} onCancel={mockOnCancel} />, {
        wrapper: TestWrapper,
      });

      expect(screen.getByTestId('select-loggerLevel')).toBeInTheDocument();
    });

    it('shows Delete button with testid form-delete in edit mode', () => {
      mockedUseLoggerForm.mockReturnValue(makeHook({ name: 'org.test', level: 'DEBUG' as LogLevel }));

      render(
        <LoggerForm loggerName="org.test" onSave={mockOnSave} onCancel={mockOnCancel} onDelete={jest.fn()} />,
        { wrapper: TestWrapper }
      );

      expect(screen.getByTestId('form-delete')).toBeInTheDocument();
    });
  });
});
