

import React from 'react';
import { render, screen, fireEvent } from '@testing-library/react';
import S3BlobStoreSettings from '../S3BlobStoreSettings';
import * as useBlobStoresModule from '../useBlobStores';

jest.mock('../useBlobStores', () => ({
  useS3DropdownValues: jest.fn(),
}));

// Mock ExtJS at the path the source uses
jest.mock('../../../../../../../interface/ExtJS', () => ({
  ExtJS: {
    isProEdition: () => true,
    state: () => ({
      getValue: () => false
    })
  }
}));

// Mock shared form components
jest.mock('../../../../../shared/form', () => ({
  SettingsFormSection: ({ children, title }) => (
    <div data-testid="settings-form-section">
      <h2>{title}</h2>
      {children}
    </div>
  ),
  SettingsTextInput: ({ label, value, onChange, error }) => (
    <div>
      <label>{label}</label>
      <input
        value={value || ''}
        onChange={(e) => onChange(e.target.value)}
        data-testid={`input-${label}`}
      />
      {error && <p data-testid={`error-${label}`}>{error}</p>}
    </div>
  ),
  SettingsPasswordInput: ({ label, value, onChange }) => (
    <div>
      <label>{label}</label>
      <input
        type="password"
        value={value || ''}
        onChange={(e) => onChange(e.target.value)}
        data-testid={`password-${label}`}
      />
    </div>
  ),
  SettingsSelect: ({ label, value, onChange, options }) => (
    <div>
      <label>{label}</label>
      <select value={value} onChange={(e) => onChange(e.target.value)} data-testid={`select-${label}`}>
        {options.map((opt) => (
          <option key={opt.value} value={opt.value}>{opt.label}</option>
        ))}
      </select>
    </div>
  ),
  SettingsCheckbox: ({ label, checked, onChange }) => (
    <div>
      <label>
        <input
          type="checkbox"
          checked={checked}
          onChange={(e) => onChange(e.target.checked)}
          data-testid={`checkbox-${label}`}
        />
        {label}
      </label>
    </div>
  ),
  SettingsAlert: ({ children }) => <div data-testid="alert">{children}</div>,
  SettingsButton: ({ children, onClick, disabled }) => (
    <button onClick={onClick} disabled={disabled}>{children}</button>
  )
}));

describe('S3BlobStoreSettings', () => {
  const mockUseS3DropdownValues = useBlobStoresModule.useS3DropdownValues as jest.Mock;

  const defaultProps = {
    data: {
      name: 'test-s3',
      bucketConfiguration: {
        bucket: {
          region: 'us-east-1',
          name: 'my-bucket',
          prefix: 'astrum/'
        },
        bucketSecurity: {},
        encryption: {},
        advancedBucketConnection: {},
        failoverBuckets: []
      }
    },
    onChange: jest.fn(),
    disabled: false,
    isEdit: false
  };

  beforeEach(() => {
    jest.clearAllMocks();

    mockUseS3DropdownValues.mockReturnValue({
      values: {
        regions: [
          { id: 'us-east-1', name: 'US East (N. Virginia)' },
          { id: 'us-west-2', name: 'US West (Oregon)' }
        ],
        encryptionTypes: [
          { id: 's3', name: 'S3 Managed' },
          { id: 'kms', name: 'KMS Managed' }
        ]
      },
      loading: false
    });
  });

  it('renders basic configuration section', () => {
    render(<S3BlobStoreSettings {...defaultProps} />);
    expect(screen.getByText('S3 Bucket Configuration')).toBeInTheDocument();
  });

  it('displays bucket name input', () => {
    render(<S3BlobStoreSettings {...defaultProps} />);
    const input = screen.getByTestId('input-Bucket');
    expect(input).toHaveValue('my-bucket');
  });

  it('displays region selector', () => {
    render(<S3BlobStoreSettings {...defaultProps} />);
    expect(screen.getByTestId('select-Region')).toBeInTheDocument();
  });

  it('calls onChange when bucket name is updated', () => {
    render(<S3BlobStoreSettings {...defaultProps} />);
    const input = screen.getByTestId('input-Bucket');
    
    fireEvent.change(input, { target: { value: 'new-bucket' } });
    
    expect(defaultProps.onChange).toHaveBeenCalledWith('bucketConfiguration.bucket.name', 'new-bucket');
  });

  it('renders authentication section', () => {
    render(<S3BlobStoreSettings {...defaultProps} />);
    expect(screen.getByText('Authentication (Optional)')).toBeInTheDocument();
  });

  it('renders encryption section', () => {
    render(<S3BlobStoreSettings {...defaultProps} />);
    expect(screen.getByText('Encryption (Optional)')).toBeInTheDocument();
  });

  it('renders advanced settings section', () => {
    render(<S3BlobStoreSettings {...defaultProps} />);
    expect(screen.getByText('Advanced Connection Settings (Optional)')).toBeInTheDocument();
  });

  it('displays pre-signed URL checkbox for Pro edition', () => {
    render(<S3BlobStoreSettings {...defaultProps} />);
    expect(screen.getByTestId('checkbox-Pre-Signed URL')).toBeInTheDocument();
  });

  it('shows documentation link', () => {
    render(<S3BlobStoreSettings {...defaultProps} />);
    expect(screen.getByText('View Documentation →')).toBeInTheDocument();
  });

  describe('field-level validation messages', () => {
    it('shouldDisplayBucketNameErrorWhenProvided', () => {
      render(
        <S3BlobStoreSettings
          {...defaultProps}
          errors={{ 'bucketConfiguration.bucket.name': 'Bucket name must be between 3 and 63 characters' }}
        />
      );

      expect(screen.getByTestId('error-Bucket')).toHaveTextContent(
        'Bucket name must be between 3 and 63 characters'
      );
    });

    it('shouldNotDisplayBucketNameErrorWhenAbsent', () => {
      render(<S3BlobStoreSettings {...defaultProps} />);

      expect(screen.queryByTestId('error-Bucket')).not.toBeInTheDocument();
    });
  });
});

