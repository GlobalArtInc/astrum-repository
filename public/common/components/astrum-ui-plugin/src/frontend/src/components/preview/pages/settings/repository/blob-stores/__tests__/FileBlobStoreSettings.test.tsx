

import React from 'react';
import { render, screen, fireEvent } from '@testing-library/react';
import FileBlobStoreSettings from '../FileBlobStoreSettings';
import { ExtJS } from '../../../../../../../interface/ExtJS';

jest.mock('../../../../../../../interface/ExtJS', () => ({
  ExtJS: {
    useState: jest.fn((fn) => fn()),
    state: jest.fn(() => ({
      getValue: jest.fn()
    }))
  }
}));

jest.mock('../../../../../shared/form', () => ({
  SettingsFormSection: ({ children, title }) => (
    <div data-testid="settings-form-section">
      <h2>{title}</h2>
      {children}
    </div>
  ),
  SettingsTextInput: ({ label, value, onChange, helpText, placeholder, required, disabled }) => (
    <div>
      <label>{label}{required && ' *'}</label>
      <input
        value={value || ''}
        onChange={(e) => onChange(e.target.value)}
        placeholder={placeholder}
        disabled={disabled}
        data-testid={`input-${label}`}
      />
      {helpText && <span className="help">{helpText}</span>}
    </div>
  ),
  SettingsAlert: ({ children, type }) => (
    <div data-testid="alert" data-type={type}>{children}</div>
  )
}));

const HA_WARNING_TITLE = 'High Availability Path Warning';
const EDIT_WARNING_TEXT = 'Changing the path will not migrate existing data.';
const WORK_DIRECTORY = '/astrum-data';

function mockExtJSState({ isClustered = false, workDirectory = WORK_DIRECTORY } = {}) {
  (ExtJS.useState as jest.Mock).mockImplementation((fn) => fn());
  (ExtJS.state as jest.Mock).mockReturnValue({
    getValue: jest.fn((key, defaultValue) => {
      if (key === 'astrum.datastore.clustered.enabled') return isClustered;
      if (key === 'astrum.application.workDirectory') return workDirectory;
      return defaultValue;
    })
  });
}

describe('FileBlobStoreSettings', () => {
  const defaultProps = {
    data: { name: 'test-store', path: '/mnt/shared/blobs' },
    onChange: jest.fn(),
    disabled: false,
    isEdit: false
  };

  beforeEach(() => {
    jest.clearAllMocks();
    mockExtJSState({ isClustered: false });
  });

  it('renders the component', () => {
    render(<FileBlobStoreSettings {...defaultProps} />);
    expect(screen.getByText('File Storage Configuration')).toBeInTheDocument();
  });

  it('displays path input with current value', () => {
    render(<FileBlobStoreSettings {...defaultProps} />);
    const input = screen.getByTestId('input-Path');
    expect(input).toHaveValue('/mnt/shared/blobs');
  });

  it('calls onChange when path is updated', () => {
    render(<FileBlobStoreSettings {...defaultProps} />);
    const input = screen.getByTestId('input-Path');

    fireEvent.change(input, { target: { value: '/new/path' } });

    expect(defaultProps.onChange).toHaveBeenCalledWith('path', '/new/path');
  });

  it('shows edit warning alert in edit mode', () => {
    render(<FileBlobStoreSettings {...defaultProps} isEdit={true} />);
    expect(screen.getByText(EDIT_WARNING_TEXT, { exact: false })).toBeInTheDocument();
  });

  it('does not show edit warning alert in create mode', () => {
    render(<FileBlobStoreSettings {...defaultProps} isEdit={false} />);
    expect(screen.queryByText(EDIT_WARNING_TEXT, { exact: false })).not.toBeInTheDocument();
  });

  it('disables input when disabled prop is true', () => {
    render(<FileBlobStoreSettings {...defaultProps} disabled={true} />);
    const input = screen.getByTestId('input-Path');
    expect(input).toBeDisabled();
  });

  it('displays help text', () => {
    render(<FileBlobStoreSettings {...defaultProps} />);
    expect(screen.getByText(/An absolute path or a path relative/)).toBeInTheDocument();
  });

  describe('HA Path Warning', () => {
    it('does not show HA warning when not clustered', () => {
      mockExtJSState({ isClustered: false });
      render(<FileBlobStoreSettings {...defaultProps} data={{ ...defaultProps.data, path: 'relative/path' }} />);
      expect(screen.queryByText(HA_WARNING_TITLE)).not.toBeInTheDocument();
    });

    it('shows HA warning when clustered and path is relative', () => {
      mockExtJSState({ isClustered: true });
      render(<FileBlobStoreSettings {...defaultProps} data={{ ...defaultProps.data, path: 'blobs/default' }} />);
      expect(screen.getByText(HA_WARNING_TITLE)).toBeInTheDocument();
    });

    it('shows HA warning when clustered and path is under work directory', () => {
      mockExtJSState({ isClustered: true, workDirectory: '/astrum-data' });
      render(<FileBlobStoreSettings {...defaultProps} data={{ ...defaultProps.data, path: '/astrum-data/blobs' }} />);
      expect(screen.getByText(HA_WARNING_TITLE)).toBeInTheDocument();
    });

    it('does not show HA warning when clustered and path is safe', () => {
      mockExtJSState({ isClustered: true, workDirectory: '/astrum-data' });
      render(<FileBlobStoreSettings {...defaultProps} data={{ ...defaultProps.data, path: '/mnt/shared/blobs' }} />);
      expect(screen.queryByText(HA_WARNING_TITLE)).not.toBeInTheDocument();
    });

    it('shows HA warning in create mode with relative path', () => {
      mockExtJSState({ isClustered: true });
      render(<FileBlobStoreSettings {...defaultProps} isEdit={false} data={{ ...defaultProps.data, path: 'default' }} />);
      expect(screen.getByText(HA_WARNING_TITLE)).toBeInTheDocument();
    });

    it('shows HA warning in edit mode with relative path', () => {
      mockExtJSState({ isClustered: true });
      render(<FileBlobStoreSettings {...defaultProps} isEdit={true} data={{ ...defaultProps.data, path: 'default' }} />);
      expect(screen.getByText(HA_WARNING_TITLE)).toBeInTheDocument();
    });

    it('does not show HA warning when path is empty', () => {
      mockExtJSState({ isClustered: true });
      render(<FileBlobStoreSettings {...defaultProps} data={{ ...defaultProps.data, path: '' }} />);
      expect(screen.queryByText(HA_WARNING_TITLE)).not.toBeInTheDocument();
    });

    it('shows warning message explaining risks', () => {
      mockExtJSState({ isClustered: true });
      render(<FileBlobStoreSettings {...defaultProps} data={{ ...defaultProps.data, path: 'blobs/default' }} />);
      expect(screen.getByText(/severe performance issues and data inconsistency/)).toBeInTheDocument();
    });
  });
});
