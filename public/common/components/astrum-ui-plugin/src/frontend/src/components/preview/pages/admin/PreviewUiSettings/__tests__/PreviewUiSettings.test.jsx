
/* global require */
import React from 'react';
import { render, screen, fireEvent } from '@testing-library/react';
import '@testing-library/jest-dom';

import PreviewUiSettings from '../PreviewUiSettings';
import * as usePreviewUiSettingsFormModule from '../../../settings/system/preview-ui/usePreviewUiSettingsForm';

jest.mock('../../../settings/system/preview-ui/usePreviewUiSettingsForm');

const mockedUseForm = usePreviewUiSettingsFormModule.usePreviewUiSettingsForm;

// Mock layout components (PreviewUiSettings.jsx imports from ../../../../layout relative to the component)
jest.mock('../../../../../layout', () => ({
  ContentBody: ({ children, className }) => <div className={className}>{children}</div>,
  Page: ({ children }) => <div>{children}</div>,
  PageHeader: ({ children }) => <header>{children}</header>,
  PageTitle: ({ text }) => <h1>{text}</h1>,
  Section: ({ children }) => <section>{children}</section>,
}));

// Mock ExtJS and FormUtils
jest.mock('../../../../../../interface/ExtJS', () => ({
  ExtJS: {
    checkPermission: jest.fn().mockReturnValue(true),
  },
}));

jest.mock('../../../../../../interface/FormUtils', () => ({
  FormUtils: {
    discardTooltip: jest.fn(() => ''),
  },
}));

// Provide UIStrings.SETTINGS so that constants/UIStrings.jsx (which re-exports from this module) works
jest.mock('@globalart/astrum-ui-plugin', () => ({
  ContentBody: ({ children, className }) => <div className={className}>{children}</div>,
  Page: ({ children }) => <div>{children}</div>,
  PageHeader: ({ children }) => <header>{children}</header>,
  PageTitle: ({ text }) => <h1>{text}</h1>,
  Section: ({ children }) => <section>{children}</section>,
  ExtJS: {
    checkPermission: jest.fn().mockReturnValue(true),
  },
  FormUtils: {
    discardTooltip: jest.fn(() => ''),
  },
  // UIStrings is re-exported by constants/UIStrings.jsx — provide minimum needed keys
  UIStrings: {
    SETTINGS: {
      DISCARD_BUTTON_LABEL: 'Discard',
      SAVE_BUTTON_LABEL: 'Save',
    },
  },
  APIConstants: {
    REST: {
      INTERNAL: {
        PREVIEW_UI_SETTINGS: '/service/rest/internal/ui/preview-ui-settings',
      },
    },
  },
}));

jest.mock('@globalart/astrum-react-shared-components', () => ({
  AsLoadWrapper: ({ loading, error, children }) => {
    if (loading) return <div>Loading...</div>;
    if (error) return <div>Error: {error}</div>;
    return <div>{children}</div>;
  },
  AsStatefulForm: ({ onSubmit, loading, submitError, additionalFooterBtns, children }) => (
    <form data-loading={loading}>
      {submitError && <div role="alert">{submitError}</div>}
      {typeof children === 'function' ? children() : children}
      {additionalFooterBtns}
      <button type="button" disabled={!!loading} onClick={() => onSubmit()}>Save</button>
    </form>
  ),
  AsCheckbox: ({ checkboxId, isChecked, onChange, disabled, children }) => (
    <label>
      <input
        type="checkbox"
        id={checkboxId}
        checked={isChecked}
        onChange={onChange}
        disabled={disabled}
        data-testid={checkboxId}
      />
      {children}
    </label>
  ),
  AsButton: ({ children, onClick, className, type }) => (
    <button type={type} onClick={onClick} className={className}>{children}</button>
  ),
  AsFieldset: ({ label, children }) => (
    <fieldset>
      <legend>{label}</legend>
      {children}
    </fieldset>
  ),
  AsTooltip: ({ children }) => <>{children}</>,
}));

const DEFAULT_SETTINGS = {
  anonymousEnabled: false,
  loggedInEnabled: true,
  defaultToPreviewUi: false,
  disableLegacyUi: false,
  disableSwitchFeedback: false,
};

function createFormMock(data = DEFAULT_SETTINGS, overrides = {}) {
  const checkboxOnChange = jest.fn();
  return {
    data,
    isPristine: true,
    isLoading: false,
    isSaving: false,
    loadError: null,
    saveError: null,
    checkbox: jest.fn((name) => ({
      checked: Boolean(data[name]),
      onChange: checkboxOnChange,
    })),
    submit: jest.fn(),
    reset: jest.fn(),
    field: jest.fn(),
    send: jest.fn(),
    ...overrides,
  };
}

function renderPage() {
  return render(<PreviewUiSettings />);
}

describe('PreviewUiSettings (heritage)', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    const { ExtJS } = require('../../../../../../interface/ExtJS');
    ExtJS.checkPermission.mockReturnValue(true);
    mockedUseForm.mockReturnValue(createFormMock());
  });

  it('shows loading state when form is loading', () => {
    mockedUseForm.mockReturnValue(createFormMock(DEFAULT_SETTINGS, { isLoading: true }));
    renderPage();
    expect(screen.getByText('Loading...')).toBeInTheDocument();
  });

  it('shows load error with retry option', () => {
    mockedUseForm.mockReturnValue(createFormMock(DEFAULT_SETTINGS, { loadError: 'Failed to load settings' }));
    renderPage();
    expect(screen.getByText(/Failed to load settings/)).toBeInTheDocument();
  });

  it('renders page title', () => {
    renderPage();
    expect(screen.getByText('Astrum One UI')).toBeInTheDocument();
  });

  it('renders all fieldsets', () => {
    renderPage();
    expect(screen.getByText('Anonymous Users')).toBeInTheDocument();
    expect(screen.getByText('Logged-in Users')).toBeInTheDocument();
    expect(screen.getByText('Rollout Control')).toBeInTheDocument();
  });

  it('Discard button has disabled class when form is pristine', () => {
    renderPage();
    expect(screen.getByRole('button', { name: /discard/i })).toHaveClass('disabled');
  });

  it('Discard button does not have disabled class when form is dirty', () => {
    mockedUseForm.mockReturnValue(createFormMock(DEFAULT_SETTINGS, { isPristine: false }));
    renderPage();
    expect(screen.getByRole('button', { name: /discard/i })).not.toHaveClass('disabled');
  });

  it('shows save error when saveError is set', () => {
    mockedUseForm.mockReturnValue(createFormMock(DEFAULT_SETTINGS, { saveError: 'Failed to save' }));
    renderPage();
    expect(screen.getByText('Failed to save')).toBeInTheDocument();
  });

  it('calls form.submit() when Save is clicked', () => {
    const mockSubmit = jest.fn();
    mockedUseForm.mockReturnValue(createFormMock(DEFAULT_SETTINGS, { isPristine: false, submit: mockSubmit }));
    renderPage();
    fireEvent.click(screen.getByRole('button', { name: /save/i }));
    expect(mockSubmit).toHaveBeenCalledTimes(1);
  });

  it('calls form.reset() when Discard is clicked', () => {
    const mockReset = jest.fn();
    mockedUseForm.mockReturnValue(createFormMock(DEFAULT_SETTINGS, { isPristine: false, reset: mockReset }));
    renderPage();
    fireEvent.click(screen.getByRole('button', { name: /discard/i }));
    expect(mockReset).toHaveBeenCalledTimes(1);
  });

  it('calls form.checkbox for each field', () => {
    const mock = createFormMock();
    mockedUseForm.mockReturnValue(mock);
    renderPage();
    expect(mock.checkbox).toHaveBeenCalledWith('anonymousEnabled');
    expect(mock.checkbox).toHaveBeenCalledWith('loggedInEnabled');
    expect(mock.checkbox).toHaveBeenCalledWith('defaultToPreviewUi');
    expect(mock.checkbox).toHaveBeenCalledWith('disableSwitchFeedback');
  });

  it('disables all checkboxes when user lacks update permission', () => {
    const { ExtJS } = require('../../../../../../interface/ExtJS');
    ExtJS.checkPermission.mockReturnValue(false);
    renderPage();
    expect(screen.getByTestId('anonymousEnabled')).toBeDisabled();
    expect(screen.getByTestId('loggedInEnabled')).toBeDisabled();
    expect(screen.getByTestId('defaultToPreviewUi')).toBeDisabled();
    expect(screen.getByTestId('disableSwitchFeedback')).toBeDisabled();
  });

  it('renders the switch feedback checkbox', () => {
    renderPage();
    expect(screen.getByTestId('disableSwitchFeedback')).toBeInTheDocument();
    expect(screen.getByText(/Hide the feedback prompt and prevent Astrum One UI from sending feedback/i))
        .toBeInTheDocument();
  });
});
