

import React from 'react';
import { Cloud, CheckCircle, XCircle } from 'lucide-react';
import { ExtJS } from '../../../../../../interface/ExtJS';
import {
  SettingsFormSection,
  SettingsTextInput,
  SettingsPasswordInput,
  SettingsSelect,
  SettingsCheckbox,
  SettingsButton,
  SettingsAlert
} from '../../../../shared/form';
import { useAzureConnectionTest } from './useBlobStores';
import type { BlobStoreFormData, AzureBlobStoreConfig } from './types';
import './AzureBlobStoreSettings.scss';
import BlobStoresStrings from '../../../../constants/pages/settings/repository/BlobStoresStrings';

interface AzureBlobStoreSettingsProps {
  data: BlobStoreFormData;
  onChange: (path: string, value: unknown) => void;
  disabled?: boolean;
  isEdit?: boolean;
  errors?: Record<string, string | null>;
}

const STRINGS = {
  TITLE: BlobStoresStrings.AZURE.AZURE_BLOB_STORAGE_CONFIGURATION,
  DESCRIPTION: BlobStoresStrings.AZURE.CONFIGURE_AZURE_STORAGE_ACCOUNT_BLOB,
  ACCOUNT_NAME: {
    label: BlobStoresStrings.AZURE.ACCOUNT_NAME,
    helpText: BlobStoresStrings.AZURE.NAME_AZURE_STORAGE_ACCOUNT,
    placeholder: 'mystorageaccount'
  },
  CONTAINER_NAME: {
    label: BlobStoresStrings.AZURE.CONTAINER_NAME,
    helpText: BlobStoresStrings.AZURE.NAME_CONTAINER_USED_STORAGE_CONTAINER,
    placeholder: 'astrum-blobs'
  },
  AUTH: {
    label: BlobStoresStrings.AZURE.AUTHENTICATION_METHOD,
    ENVIRONMENT: {
      value: 'ENVIRONMENTVARIABLE',
      label: BlobStoresStrings.AZURE.USE_ENVIRONMENT_VARIABLES
    },
    MANAGED: {
      value: 'MANAGEDIDENTITY',
      label: BlobStoresStrings.AZURE.MANAGED_IDENTITY_SYSTEM
    },
    ACCOUNT_KEY: {
      value: 'ACCOUNTKEY',
      label: BlobStoresStrings.AZURE.ACCOUNT_KEY,
      field: {
        label: BlobStoresStrings.AZURE.ACCOUNT_KEY,
        helpText: BlobStoresStrings.AZURE.ACCOUNT_KEY_FOUND_UNDER_ACCESS,
        placeholder: BlobStoresStrings.AZURE.AZURE_STORAGE_ACCOUNT_KEY
      }
    }
  },
  DIRECT_DOWNLOAD: {
    label: BlobStoresStrings.AZURE.DIRECT_DOWNLOAD_SAS_URLS,
    description: BlobStoresStrings.AZURE.REDIRECT_DOWNLOADS_DIRECTLY_AZURE_BLOB,
    rbacNote: BlobStoresStrings.AZURE.IDENTITY_NEEDS_AZURE_ROLE_CAN
  },
  TEST_CONNECTION: {
    button: BlobStoresStrings.AZURE.TEST_CONNECTION,
    testing: BlobStoresStrings.AZURE.TESTING_CONNECTION,
    success: BlobStoresStrings.AZURE.CONNECTION_SUCCEEDED,
    error: BlobStoresStrings.AZURE.CONNECTION_FAILED_CHECK_LOGS_MORE
  }
};

const AUTH_OPTIONS = [
  { value: STRINGS.AUTH.ENVIRONMENT.value, label: STRINGS.AUTH.ENVIRONMENT.label },
  { value: STRINGS.AUTH.MANAGED.value, label: STRINGS.AUTH.MANAGED.label },
  { value: STRINGS.AUTH.ACCOUNT_KEY.value, label: STRINGS.AUTH.ACCOUNT_KEY.label }
];

export default function AzureBlobStoreSettings({
  data,
  onChange,
  disabled = false,
  isEdit = false,
  errors = {},
}: AzureBlobStoreSettingsProps) {
  const { testing, result, testConnection, reset } = useAzureConnectionTest();

  const isProEdition = ExtJS.isProEdition();

  const config: AzureBlobStoreConfig = (data.bucketConfiguration as AzureBlobStoreConfig) || {
    accountName: '',
    containerName: '',
    authentication: { authenticationMethod: 'ENVIRONMENTVARIABLE' }
  };

  const updateConfig = (field: string, value: unknown) => {
    onChange(`bucketConfiguration.${field}`, value);
    reset(); // Clear test result when config changes
  };

  const updateAuth = (field: string, value: unknown) => {
    onChange(`bucketConfiguration.authentication.${field}`, value);
    reset();
  };

  const handleTestConnection = () => {
    testConnection({
      blobStoreName: isEdit ? data.name : undefined,
      accountName: config.accountName,
      containerName: config.containerName,
      authenticationMethod: config.authentication?.authenticationMethod || 'ENVIRONMENTVARIABLE',
      accountKey: config.authentication?.accountKey
    });
  };

  const showAccountKey = config.authentication?.authenticationMethod === 'ACCOUNTKEY';
  const isTokenCredentialAuth =
      config.authentication?.authenticationMethod === 'MANAGEDIDENTITY' ||
      config.authentication?.authenticationMethod === 'ENVIRONMENTVARIABLE';
  const showRbacNote =
      isProEdition && !!config.preSignedUrlEnabled && isTokenCredentialAuth;

  return (
    <div className="azure-blob-store-settings">
      <SettingsFormSection
        title={STRINGS.TITLE}
        description={STRINGS.DESCRIPTION}
        icon={<Cloud size={20} />}
      >
        <SettingsTextInput
          name="azure-account-name"
          label={STRINGS.ACCOUNT_NAME.label}
          value={config.accountName || ''}
          onChange={(value) => updateConfig('accountName', value)}
          helpText={STRINGS.ACCOUNT_NAME.helpText}
          placeholder={STRINGS.ACCOUNT_NAME.placeholder}
          error={errors['bucketConfiguration.accountName'] ?? undefined}
          required
          disabled={disabled}
        />

        <SettingsTextInput
          name="azure-container-name"
          label={STRINGS.CONTAINER_NAME.label}
          value={config.containerName || ''}
          onChange={(value) => updateConfig('containerName', value)}
          helpText={STRINGS.CONTAINER_NAME.helpText}
          placeholder={STRINGS.CONTAINER_NAME.placeholder}
          error={errors['bucketConfiguration.containerName'] ?? undefined}
          required
          disabled={disabled}
        />

        <SettingsSelect
          name="azure-auth-method"
          label={STRINGS.AUTH.label}
          value={config.authentication?.authenticationMethod || 'ENVIRONMENTVARIABLE'}
          onChange={(value) => updateAuth('authenticationMethod', value)}
          options={AUTH_OPTIONS}
          disabled={disabled}
        />

        {showAccountKey && (
          <SettingsPasswordInput
            name="azure-account-key"
            label={STRINGS.AUTH.ACCOUNT_KEY.field.label}
            value={config.authentication?.accountKey || ''}
            onChange={(value) => updateAuth('accountKey', value)}
            helpText={STRINGS.AUTH.ACCOUNT_KEY.field.helpText}
            placeholder={STRINGS.AUTH.ACCOUNT_KEY.field.placeholder}
            required
            disabled={disabled}
            autoComplete="new-password"
          />
        )}

        {isProEdition && (
          <div className="azure-blob-store-settings__sas-section">
            <SettingsCheckbox
              name="azure-direct-download"
              label={STRINGS.DIRECT_DOWNLOAD.label}
              description={STRINGS.DIRECT_DOWNLOAD.description}
              checked={config.preSignedUrlEnabled || false}
              onChange={(checked) => onChange('bucketConfiguration.preSignedUrlEnabled', checked)}
              disabled={disabled}
            />

            {showRbacNote && (
              <SettingsAlert type="info">
                {STRINGS.DIRECT_DOWNLOAD.rbacNote}
              </SettingsAlert>
            )}
          </div>
        )}

        <div className="azure-blob-store-settings__test-section">
          <SettingsButton
            variant="secondary"
            onClick={handleTestConnection}
            disabled={disabled || testing || !config.accountName || !config.containerName}
            testId="azure-test-connection"
            loading={testing}
          >
            {testing ? STRINGS.TEST_CONNECTION.testing : STRINGS.TEST_CONNECTION.button}
          </SettingsButton>

          {result === 'success' && (
            <SettingsAlert variant="success" icon={<CheckCircle size={16} />}>
              {STRINGS.TEST_CONNECTION.success}
            </SettingsAlert>
          )}

          {result === 'error' && (
            <SettingsAlert variant="error" icon={<XCircle size={16} />}>
              {STRINGS.TEST_CONNECTION.error}
            </SettingsAlert>
          )}
        </div>
      </SettingsFormSection>
    </div>
  );
}

