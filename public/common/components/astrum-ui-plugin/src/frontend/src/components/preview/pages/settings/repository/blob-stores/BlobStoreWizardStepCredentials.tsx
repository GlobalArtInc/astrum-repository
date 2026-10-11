

import React, { useState, useRef } from 'react';
import { Box, Flex, Text } from '@radix-ui/themes';
import { Shield, Upload, CheckCircle, XCircle } from 'lucide-react';
import {
  SettingsFormSection,
  SettingsTextInput,
  SettingsPasswordInput,
  SettingsSelect,
  SettingsCheckbox,
  SettingsAlert,
  SettingsButton,
} from '../../../../shared/form';
import { useS3DropdownValues, useAzureConnectionTest } from './useBlobStores';
import type { BlobStoreFormData, S3BlobStoreConfig, AzureBlobStoreConfig } from './types';
import { BLOB_STORE_TYPE_IDS } from './blobStoreFormMachine';
import type { BlobStoreTypeId } from './BlobStoreTypeSelector';
import BlobStoresStrings from '../../../../constants/pages/settings/repository/BlobStoresStrings';

const AUTH_OPTIONS_AZURE = [
  { value: 'ENVIRONMENTVARIABLE', label: BlobStoresStrings.WIZARD_CREDENTIALS.USE_ENVIRONMENT_VARIABLES },
  { value: 'MANAGEDIDENTITY', label: BlobStoresStrings.WIZARD_CREDENTIALS.MANAGED_IDENTITY_SYSTEM },
  { value: 'ACCOUNTKEY', label: BlobStoresStrings.WIZARD_CREDENTIALS.ACCOUNT_KEY },
];

const AUTH_OPTIONS_GOOGLE = [
  { value: 'applicationDefault', label: BlobStoresStrings.WIZARD_CREDENTIALS.APPLICATION_DEFAULT_CREDENTIALS },
  { value: 'accountKey', label: BlobStoresStrings.WIZARD_CREDENTIALS.CREDENTIAL_JSON_FILE },
];

export interface BlobStoreWizardStepCredentialsProps {
  data: BlobStoreFormData;
  selectedType: BlobStoreTypeId;
  onChange: (path: string, value: unknown) => void;
}

export function BlobStoreWizardStepCredentials({
  data,
  selectedType,
  onChange,
}: BlobStoreWizardStepCredentialsProps) {
  const { values: s3Dropdown, loading: s3Loading } = useS3DropdownValues();
  const { testing: azureTesting, result: azureResult, testConnection, reset } = useAzureConnectionTest();
  const [googleFileName, setGoogleFileName] = useState<string | null>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const sec = (data.bucketConfiguration as S3BlobStoreConfig)?.bucketSecurity || {};
  const enc = (data.bucketConfiguration as S3BlobStoreConfig)?.encryption;
  const azureConfig = (data.bucketConfiguration || {}) as AzureBlobStoreConfig;
  const azureAuth = azureConfig.authentication || { authenticationMethod: 'ENVIRONMENTVARIABLE' };
  const gConfig = data.bucketConfiguration as Record<string, unknown>;
  const gSecurity = (gConfig?.bucketSecurity || {}) as Record<string, string>;
  const gEnc = (gConfig?.encryption || {}) as Record<string, string>;
  const encryptionTypes = s3Dropdown?.encryptionTypes || [];

  if (selectedType === BLOB_STORE_TYPE_IDS.FILE || selectedType === BLOB_STORE_TYPE_IDS.GROUP) {
    return (
      <Box
        p="6"
        style={{
          background: 'var(--gray-2)',
          borderRadius: 'var(--radius-4)',
          border: '1px solid var(--gray-4)',
        }}
      >
        <Flex align="center" gap="4">
          <Shield size={32} color="var(--green-9)" />
          <Box>
            <Text weight="bold" size="3" as="p">
              {BlobStoresStrings.WIZARD_CREDENTIALS.NO_CREDENTIALS_REQUIRED}
            </Text>
            <Text color="gray" size="2" as="p">
              {selectedType === BLOB_STORE_TYPE_IDS.FILE
                ? BlobStoresStrings.WIZARD_CREDENTIALS.FILE_BLOB_STORES_USE_LOCAL
                : BlobStoresStrings.WIZARD_CREDENTIALS.GROUP_BLOB_STORES_AGGREGATE_EXISTING}{' '}
              {BlobStoresStrings.WIZARD_CREDENTIALS.CLICK_NEXT_CONTINUE_ADVANCED_OPTIONS}
            </Text>
          </Box>
        </Flex>
      </Box>
    );
  }

  if (selectedType === BLOB_STORE_TYPE_IDS.S3) {
    return (
      <>
        <SettingsFormSection
          title={BlobStoresStrings.WIZARD_CREDENTIALS.AUTHENTICATION}
          description={BlobStoresStrings.WIZARD_CREDENTIALS.AWS_CREDENTIALS_LEAVE_BLANK_IAM}
        >
          <SettingsTextInput
            name="s3-access-key"
            label={BlobStoresStrings.WIZARD_CREDENTIALS.ACCESS_KEY_ID}
            value={sec.accessKeyId || ''}
            onChange={(v) => onChange('bucketConfiguration.bucketSecurity.accessKeyId', v)}
            placeholder={BlobStoresStrings.WIZARD_CREDENTIALS.AKIAIOSFODNN7EXAMPLE_PLACEHOLDER}
          />
          <SettingsPasswordInput
            name="s3-secret-key"
            label={BlobStoresStrings.WIZARD_CREDENTIALS.SECRET_ACCESS_KEY}
            value={sec.secretAccessKey || ''}
            onChange={(v) => onChange('bucketConfiguration.bucketSecurity.secretAccessKey', v)}
            placeholder="••••••••••••••••••••"
            autoComplete="new-password"
          />
          <SettingsTextInput
            name="s3-role-arn"
            label={BlobStoresStrings.WIZARD_CREDENTIALS.ASSUME_ROLE_ARN}
            value={sec.role || ''}
            onChange={(v) => onChange('bucketConfiguration.bucketSecurity.role', v)}
            placeholder="arn:aws:iam::123456789012:role/S3Access"
          />
          <SettingsPasswordInput
            name="s3-session-token"
            label={BlobStoresStrings.WIZARD_CREDENTIALS.SESSION_TOKEN}
            value={sec.sessionToken || ''}
            onChange={(v) => onChange('bucketConfiguration.bucketSecurity.sessionToken', v)}
            placeholder={BlobStoresStrings.WIZARD_CREDENTIALS.TEMPORARY_TOKEN_OPTIONAL_PLACEHOLDER}
          />
        </SettingsFormSection>
        <SettingsFormSection title={BlobStoresStrings.WIZARD_CREDENTIALS.ENCRYPTION} description={BlobStoresStrings.WIZARD_CREDENTIALS.OPTIONAL_SERVER_SIDE_ENCRYPTION_S3}>
          <SettingsSelect
            name="s3-encryption-type"
            label={BlobStoresStrings.WIZARD_CREDENTIALS.ENCRYPTION_TYPE}
            value={enc?.encryptionType || ''}
            onChange={(v) => onChange('bucketConfiguration.encryption.encryptionType', v)}
            options={[
              { value: '', label: BlobStoresStrings.WIZARD_CREDENTIALS.NONE },
              ...encryptionTypes.map((e) => ({ value: e.id, label: e.name })),
            ]}
            disabled={s3Loading}
          />
          <SettingsTextInput
            name="s3-kms-key"
            label={BlobStoresStrings.WIZARD_CREDENTIALS.KMS_KEY_ID_OPTIONAL}
            value={enc?.encryptionKey || ''}
            onChange={(v) => onChange('bucketConfiguration.encryption.encryptionKey', v)}
            placeholder="arn:aws:kms:us-east-1:123456789012:key/..."
          />
        </SettingsFormSection>
      </>
    );
  }

  if (selectedType === BLOB_STORE_TYPE_IDS.AZURE) {
    const showKey = azureAuth.authenticationMethod === 'ACCOUNTKEY';
    return (
      <SettingsFormSection
        title={BlobStoresStrings.WIZARD_CREDENTIALS.AUTHENTICATION}
        description={BlobStoresStrings.WIZARD_CREDENTIALS.CONFIGURE_HOW_ASTRUM_AUTHENTICATES_AZURE}
      >
        <SettingsSelect
          name="azure-auth"
          label={BlobStoresStrings.WIZARD_CREDENTIALS.AUTHENTICATION_METHOD}
          value={azureAuth.authenticationMethod || 'ENVIRONMENTVARIABLE'}
          onChange={(v) => {
            onChange('bucketConfiguration.authentication.authenticationMethod', v);
            reset();
          }}
          options={AUTH_OPTIONS_AZURE}
        />
        {showKey && (
          <SettingsPasswordInput
            name="azure-account-key"
            label={BlobStoresStrings.WIZARD_CREDENTIALS.ACCOUNT_KEY}
            value={azureAuth.accountKey || ''}
            onChange={(v) => {
              onChange('bucketConfiguration.authentication.accountKey', v);
              reset();
            }}
            placeholder={BlobStoresStrings.WIZARD_CREDENTIALS.AZURE_STORAGE_ACCOUNT_KEY_PLACEHOLDER}
            required
            autoComplete="new-password"
          />
        )}
        <Flex gap="2" mt="4" align="center">
          <SettingsButton
            variant="secondary"
            onClick={() =>
              testConnection({
                accountName: azureConfig.accountName,
                containerName: azureConfig.containerName,
                authenticationMethod: azureAuth.authenticationMethod || 'ENVIRONMENTVARIABLE',
                accountKey: azureAuth.accountKey,
              })
            }
            disabled={azureTesting || !azureConfig.accountName || !azureConfig.containerName}
            loading={azureTesting}
          >
            {azureTesting ? BlobStoresStrings.WIZARD_CREDENTIALS.TESTING : BlobStoresStrings.WIZARD_CREDENTIALS.TEST_CONNECTION}
          </SettingsButton>
          {azureResult === 'success' && (
            <SettingsAlert variant="success" icon={<CheckCircle size={16} />}>
              {BlobStoresStrings.WIZARD_CREDENTIALS.CONNECTION_SUCCEEDED}
            </SettingsAlert>
          )}
          {azureResult === 'error' && (
            <SettingsAlert variant="error" icon={<XCircle size={16} />}>
              {BlobStoresStrings.WIZARD_CREDENTIALS.CONNECTION_FAILED}
            </SettingsAlert>
          )}
        </Flex>
      </SettingsFormSection>
    );
  }

  if (selectedType === BLOB_STORE_TYPE_IDS.GOOGLE) {
    const showFile = gSecurity.authenticationMethod === 'accountKey';
    const isKms = gEnc.encryptionType === 'kmsManagedEncryption';
    return (
      <>
        <SettingsFormSection title={BlobStoresStrings.WIZARD_CREDENTIALS.AUTHENTICATION} description={BlobStoresStrings.WIZARD_CREDENTIALS.HOW_ASTRUM_AUTHENTICATES_GCP}>
          <SettingsSelect
            name="google-auth"
            label={BlobStoresStrings.WIZARD_CREDENTIALS.METHOD}
            value={gSecurity.authenticationMethod || 'applicationDefault'}
            onChange={(v) => onChange('bucketConfiguration.bucketSecurity.authenticationMethod', v)}
            options={AUTH_OPTIONS_GOOGLE}
          />
          {showFile && (
            <Box mt="2">
              <input
                ref={fileInputRef}
                type="file"
                accept=".json"
                onChange={(e) => {
                  const f = e.target.files?.[0];
                  if (f) {
                    setGoogleFileName(f.name);
                    onChange('bucketConfiguration.bucketSecurity.file', f);
                  }
                }}
                style={{ display: 'none' }}
              />
              <SettingsButton
                variant="secondary"
                onClick={() => fileInputRef.current?.click()}
                icon={Upload}
              >
                {BlobStoresStrings.WIZARD_CREDENTIALS.CHOOSE_JSON_CREDENTIAL_FILE}
              </SettingsButton>
              {googleFileName && (
                <Text size="1" color="gray" ml="2" as="span">
                  {BlobStoresStrings.WIZARD_CREDENTIALS.SELECTED(googleFileName)}
                </Text>
              )}
            </Box>
          )}
        </SettingsFormSection>
        <SettingsFormSection title={BlobStoresStrings.WIZARD_CREDENTIALS.ENCRYPTION} description={BlobStoresStrings.WIZARD_CREDENTIALS.KMS_ENCRYPTION_OPTIONAL}>
          <SettingsCheckbox
            name="google-kms"
            label={BlobStoresStrings.WIZARD_CREDENTIALS.ENABLE_KMS_MANAGED_ENCRYPTION}
            checked={isKms}
            onChange={(v) => onChange('bucketConfiguration.encryption.encryptionType', v ? 'kmsManagedEncryption' : 'default')}
          />
          {isKms && (
            <SettingsTextInput
              name="google-kms-key"
              label={BlobStoresStrings.WIZARD_CREDENTIALS.KMS_KEY_ID}
              value={gEnc.encryptionKey || ''}
              onChange={(v) => onChange('bucketConfiguration.encryption.encryptionKey', v)}
              placeholder="projects/PROJECT/locations/LOC/keyRings/RING/cryptoKeys/KEY"
              required
              monospace
            />
          )}
        </SettingsFormSection>
      </>
    );
  }

  return null;
}
