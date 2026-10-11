

import React, { useState, useRef } from 'react';
import { Cloud, Upload, Info } from 'lucide-react';
import {
  SettingsFormSection,
  SettingsTextInput,
  SettingsSelect,
  SettingsCheckbox,
  SettingsAlert
} from '../../../../shared/form';
import type { BlobStoreFormData, GoogleBlobStoreConfig } from './types';
import './GoogleBlobStoreSettings.scss';
import BlobStoresStrings from '../../../../constants/pages/settings/repository/BlobStoresStrings';

interface GoogleBlobStoreSettingsProps {
  data: BlobStoreFormData;
  onChange: (path: string, value: unknown) => void;
  disabled?: boolean;
  isEdit?: boolean;
  errors?: Record<string, string | null>;
}

const STRINGS = {
  BASIC: {
    title: BlobStoresStrings.GOOGLE.GOOGLE_CLOUD_STORAGE_CONFIGURATION,
    description: BlobStoresStrings.GOOGLE.CONFIGURE_GCP_BUCKET_BLOB_STORAGE,
    REGION: {
      label: BlobStoresStrings.GOOGLE.REGION,
      helpText: BlobStoresStrings.GOOGLE.REGION_AUTOMATICALLY_SET_BASED_WHERE
    },
    PROJECT_ID: {
      label: BlobStoresStrings.GOOGLE.PROJECT_ID,
      helpText: BlobStoresStrings.GOOGLE.GCP_PROJECT_ID_UNIQUE_IDENTIFIER,
      placeholder: 'my-gcp-project'
    },
    BUCKET: {
      label: BlobStoresStrings.GOOGLE.BUCKET,
      helpText: BlobStoresStrings.GOOGLE.GOOGLE_CLOUD_PLATFORM_BUCKET_NAME,
      placeholder: 'my-astrum-bucket'
    },
    PREFIX: {
      label: BlobStoresStrings.GOOGLE.PREFIX,
      helpText: BlobStoresStrings.GOOGLE.GOOGLE_CLOUD_STORAGE_PATH_PREFIX,
      placeholder: 'astrum/'
    }
  },
  AUTH: {
    title: BlobStoresStrings.GOOGLE.AUTHENTICATION,
    label: BlobStoresStrings.GOOGLE.AUTHENTICATION_METHOD,
    APPLICATION_DEFAULT: {
      value: 'applicationDefault',
      label: BlobStoresStrings.GOOGLE.USE_GOOGLE_APPLICATION_DEFAULT_CREDENTIALS
    },
    ACCOUNT_KEY: {
      value: 'accountKey',
      label: BlobStoresStrings.GOOGLE.USE_SEPARATE_CREDENTIAL_JSON_FILE
    },
    FILE: {
      label: BlobStoresStrings.GOOGLE.JSON_CREDENTIAL_FILE,
      helpText: BlobStoresStrings.GOOGLE.UPLOAD_JSON_FILE_MAXIMUM_SIZE,
      button: BlobStoresStrings.GOOGLE.CHOOSE_FILE,
      selected: (name: string) => BlobStoresStrings.GOOGLE.SELECTED(name)
    }
  },
  ENCRYPTION: {
    title: BlobStoresStrings.GOOGLE.ENCRYPTION,
    description: BlobStoresStrings.GOOGLE.DATA_ENCRYPTED_DEFAULT_ENABLE_KMS,
    note: BlobStoresStrings.GOOGLE.ENCRYPTION_SETTINGS_CANNOT_CHANGED_ONCE,
    KMS_ENABLED: {
      label: BlobStoresStrings.GOOGLE.ENABLE_KMS_MANAGED_ENCRYPTION
    },
    KEY_NAME: {
      label: BlobStoresStrings.GOOGLE.KMS_KEY_ID,
      helpText: BlobStoresStrings.GOOGLE.ENTER_KMS_KEY_ID_USE,
      placeholder: 'projects/PROJECT_ID/locations/LOCATION/keyRings/KEY_RING_NAME/cryptoKeys/KEY_NAME'
    }
  }
};

const AUTH_OPTIONS = [
  { value: STRINGS.AUTH.APPLICATION_DEFAULT.value, label: STRINGS.AUTH.APPLICATION_DEFAULT.label },
  { value: STRINGS.AUTH.ACCOUNT_KEY.value, label: STRINGS.AUTH.ACCOUNT_KEY.label }
];

export default function GoogleBlobStoreSettings({
  data,
  onChange,
  disabled = false,
  isEdit = false,
  errors = {},
}: GoogleBlobStoreSettingsProps) {
  const fileInputRef = useRef<HTMLInputElement>(null);
  const [selectedFileName, setSelectedFileName] = useState<string | null>(null);

  const config: GoogleBlobStoreConfig = (data.bucketConfiguration as GoogleBlobStoreConfig) || {
    bucket: { name: '' },
    bucketSecurity: { authenticationMethod: 'applicationDefault' },
    encryption: { encryptionType: 'default' }
  };

  const updateBucket = (field: string, value: unknown) => {
    onChange(`bucketConfiguration.bucket.${field}`, value);
  };

  const updateSecurity = (field: string, value: unknown) => {
    onChange(`bucketConfiguration.bucketSecurity.${field}`, value);
  };

  const updateEncryption = (field: string, value: unknown) => {
    onChange(`bucketConfiguration.encryption.${field}`, value);
  };

  const handleFileSelect = (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (file) {
      setSelectedFileName(file.name);
      onChange('bucketConfiguration.bucketSecurity.file', file);
    }
  };

  const handleKmsToggle = (enabled: boolean) => {
    updateEncryption('encryptionType', enabled ? 'kmsManagedEncryption' : 'default');
  };

  const showFileUpload = config.bucketSecurity?.authenticationMethod === 'accountKey';
  const isKmsEnabled = config.encryption?.encryptionType === 'kmsManagedEncryption';

  return (
    <div className="google-blob-store-settings">
      {/* Basic Configuration */}
      <SettingsFormSection
        title={STRINGS.BASIC.title}
        description={STRINGS.BASIC.description}
        icon={<Cloud size={20} />}
      >
        <SettingsAlert variant="info" icon={<Info size={16} />}>
          {STRINGS.BASIC.REGION.helpText}
        </SettingsAlert>

        <SettingsTextInput
          name="google-project-id"
          label={STRINGS.BASIC.PROJECT_ID.label}
          value={config.bucket?.projectId || ''}
          onChange={(value) => updateBucket('projectId', value)}
          helpText={STRINGS.BASIC.PROJECT_ID.helpText}
          placeholder={STRINGS.BASIC.PROJECT_ID.placeholder}
          disabled={disabled}
        />

        <SettingsTextInput
          name="google-bucket"
          label={STRINGS.BASIC.BUCKET.label}
          value={config.bucket?.name || ''}
          onChange={(value) => updateBucket('name', value)}
          helpText={STRINGS.BASIC.BUCKET.helpText}
          placeholder={STRINGS.BASIC.BUCKET.placeholder}
          error={errors['bucketConfiguration.bucket.name'] ?? undefined}
          required
          disabled={disabled}
        />

        <SettingsTextInput
          name="google-prefix"
          label={STRINGS.BASIC.PREFIX.label}
          value={config.bucket?.prefix || ''}
          onChange={(value) => updateBucket('prefix', value)}
          helpText={STRINGS.BASIC.PREFIX.helpText}
          placeholder={STRINGS.BASIC.PREFIX.placeholder}
          disabled={disabled}
        />
      </SettingsFormSection>

      {/* Authentication */}
      <SettingsFormSection title={STRINGS.AUTH.title}>
        <SettingsSelect
          name="google-auth-method"
          label={STRINGS.AUTH.label}
          value={config.bucketSecurity?.authenticationMethod || 'applicationDefault'}
          onChange={(value) => updateSecurity('authenticationMethod', value)}
          options={AUTH_OPTIONS}
          disabled={disabled}
        />

        {showFileUpload && (
          <div className="google-blob-store-settings__file-upload">
            <label className="google-blob-store-settings__file-label">
              {STRINGS.AUTH.FILE.label}
              {!isEdit && <span className="google-blob-store-settings__required">*</span>}
            </label>
            <p className="google-blob-store-settings__file-help">{STRINGS.AUTH.FILE.helpText}</p>
            
            <input
              ref={fileInputRef}
              type="file"
              accept=".json"
              onChange={handleFileSelect}
              className="google-blob-store-settings__file-input"
              disabled={disabled}
            />
            
            <button
              type="button"
              onClick={() => fileInputRef.current?.click()}
              className="google-blob-store-settings__file-button"
              disabled={disabled}
            >
              <Upload size={16} />
              {STRINGS.AUTH.FILE.button}
            </button>

            {selectedFileName && (
              <p className="google-blob-store-settings__file-selected">
                {STRINGS.AUTH.FILE.selected(selectedFileName)}
              </p>
            )}
          </div>
        )}
      </SettingsFormSection>

      {/* Encryption */}
      <SettingsFormSection
        title={STRINGS.ENCRYPTION.title}
        description={STRINGS.ENCRYPTION.description}
      >
        <SettingsAlert variant="info" icon={<Info size={16} />}>
          {STRINGS.ENCRYPTION.note}
        </SettingsAlert>

        <SettingsCheckbox
          name="google-kms-enabled"
          label={STRINGS.ENCRYPTION.KMS_ENABLED.label}
          checked={isKmsEnabled}
          onChange={handleKmsToggle}
          disabled={disabled}
        />

        {isKmsEnabled && (
          <SettingsTextInput
            name="google-kms-key"
            label={STRINGS.ENCRYPTION.KEY_NAME.label}
            value={config.encryption?.encryptionKey || ''}
            onChange={(value) => updateEncryption('encryptionKey', value)}
            helpText={STRINGS.ENCRYPTION.KEY_NAME.helpText}
            placeholder={STRINGS.ENCRYPTION.KEY_NAME.placeholder}
            required={!isEdit}
            disabled={disabled}
            monospace
          />
        )}
      </SettingsFormSection>
    </div>
  );
}

