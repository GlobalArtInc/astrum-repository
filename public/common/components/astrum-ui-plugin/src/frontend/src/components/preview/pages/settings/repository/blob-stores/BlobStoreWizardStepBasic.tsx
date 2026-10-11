

import React from 'react';
import {
  SettingsFormSection,
  SettingsTextInput,
  SettingsSelect,
  SettingsCheckbox,
  SettingsTransferList,
  SettingsAlert,
} from '../../../../shared/form';
import { useIsClustered } from '../../../../shared/hooks';
import { ExtJS } from '../../../../../../interface/ExtJS';
import { useS3DropdownValues, useGroupableBlobStores } from './useBlobStores';
import { shouldShowHaPathWarning, HA_PATH_WARNING } from './haPathWarning';
import type { BlobStoreFormData, S3BlobStoreConfig } from './types';
import { BLOB_STORE_TYPE_IDS } from './blobStoreFormMachine';
import type { BlobStoreTypeId } from './BlobStoreTypeSelector';
import './AzureBlobStoreSettings.scss';
import BlobStoresStrings from '../../../../constants/pages/settings/repository/BlobStoresStrings';
import CommonStrings from '../../../../constants/CommonStrings';

const FILL_POLICY_OPTIONS = [
  { value: '', label: BlobStoresStrings.WIZARD_BASIC.SELECT_FILL_POLICY },
  { value: 'writeToFirst', label: BlobStoresStrings.WIZARD_BASIC.WRITE_FIRST },
  { value: 'roundRobin', label: BlobStoresStrings.WIZARD_BASIC.ROUND_ROBIN },
];

export interface BlobStoreWizardStepBasicProps {
  data: BlobStoreFormData;
  selectedType: BlobStoreTypeId;
  onChange: (path: string, value: unknown) => void;
  updateField: (field: string, value: unknown) => void;
  validationErrors: Record<string, string>;
}

export function BlobStoreWizardStepBasic({
  data,
  selectedType,
  onChange,
  updateField,
  validationErrors,
}: BlobStoreWizardStepBasicProps) {
  const { values: s3Dropdown, loading: s3Loading } = useS3DropdownValues();
  const { blobStores, loading: groupLoading } = useGroupableBlobStores();
  const isProEdition = ExtJS.isProEdition();
  const isClustered = useIsClustered();
  const workDirectory = ExtJS.state()?.getValue?.('astrum.application.workDirectory') || '';

  const config = (data.bucketConfiguration || {}) as S3BlobStoreConfig;
  const bucket = config.bucket || { region: '', name: '', prefix: '' };
  const regions = s3Dropdown?.regions || [];

  const members: string[] = data.members || [];
  const availableOptions = blobStores
    .filter((s) => !members.includes(s))
    .map((s) => ({ value: s, label: s }));
  const selectedOptions = members.map((s) => ({ value: s, label: s }));

  return (
    <SettingsFormSection title={BlobStoresStrings.WIZARD_BASIC.BASIC_CONFIGURATION} description={BlobStoresStrings.WIZARD_BASIC.NAME_REQUIRED_SETTINGS_BLOB_STORE}>
        <SettingsTextInput
          name="blobstore-name"
          label={CommonStrings.LABELS.NAME}
          value={data.name || ''}
          onChange={(v) => updateField('name', v)}
          placeholder="my-blob-store"
          helpText={BlobStoresStrings.WIZARD_BASIC.UNIQUE_IDENTIFIER_USE_LETTERS_NUMBERS}
          required
          error={validationErrors.name}
        />

        {selectedType === BLOB_STORE_TYPE_IDS.FILE && (
          <>
            <SettingsTextInput
              name="file-path"
              label={BlobStoresStrings.WIZARD_BASIC.PATH}
              value={data.path || ''}
              onChange={(v) => onChange('path', v)}
              placeholder="/path/to/blob/storage"
              helpText={BlobStoresStrings.WIZARD_BASIC.PATH_HELP}
              required
              error={validationErrors.path}
              monospace
            />
            {isClustered && shouldShowHaPathWarning(data.path, workDirectory) && (
              <SettingsAlert type="warning">
                <strong>{HA_PATH_WARNING.TITLE}</strong>
                <br />
                {HA_PATH_WARNING.MESSAGE}
              </SettingsAlert>
            )}
          </>
        )}

        {selectedType === BLOB_STORE_TYPE_IDS.S3 && (
          <>
            <SettingsSelect
              name="s3-region"
              label={BlobStoresStrings.WIZARD_BASIC.REGION}
              value={bucket.region || ''}
              onChange={(v) => onChange('bucketConfiguration.bucket.region', v)}
              options={[
                { value: '', label: BlobStoresStrings.WIZARD_BASIC.SELECT_REGION },
                ...regions.map((r) => ({ value: r.id, label: r.name })),
              ]}
              helpText={BlobStoresStrings.WIZARD_BASIC.AWS_REGION_WHERE_BUCKET_RESIDES}
              disabled={s3Loading}
            />
            <SettingsTextInput
              name="s3-bucket"
              label={BlobStoresStrings.WIZARD_BASIC.BUCKET}
              value={bucket.name || ''}
              onChange={(v) => onChange('bucketConfiguration.bucket.name', v)}
              placeholder="my-blob-store-bucket"
              helpText={BlobStoresStrings.WIZARD_BASIC.S3_BUCKET_NAME_3_63}
              required
              error={validationErrors['bucketConfiguration.bucket.name']}
            />
            <SettingsTextInput
              name="s3-prefix"
              label={BlobStoresStrings.WIZARD_BASIC.PREFIX}
              value={bucket.prefix || ''}
              onChange={(v) => onChange('bucketConfiguration.bucket.prefix', v)}
              placeholder="astrum/"
              helpText={BlobStoresStrings.WIZARD_BASIC.OPTIONAL_PATH_PREFIX_WITHIN_BUCKET}
            />
            {isProEdition && (
              <SettingsCheckbox
                name="s3-presigned"
                label={BlobStoresStrings.WIZARD_BASIC.PRE_SIGNED_URL}
                description={BlobStoresStrings.WIZARD_BASIC.REDIRECT_DOWNLOADS_S3_BETTER_PERFORMANCE}
                checked={config.preSignedUrlEnabled}
                onChange={(v) => onChange('bucketConfiguration.preSignedUrlEnabled', v)}
              />
            )}
          </>
        )}

        {selectedType === BLOB_STORE_TYPE_IDS.AZURE && (
          <>
            <SettingsTextInput
              name="azure-account"
              label={BlobStoresStrings.WIZARD_BASIC.ACCOUNT_NAME}
              value={(config as Record<string, string>).accountName || ''}
              onChange={(v) => onChange('bucketConfiguration.accountName', v)}
              placeholder="mystorageaccount"
              helpText={BlobStoresStrings.WIZARD_BASIC.AZURE_STORAGE_ACCOUNT_NAME}
              required
              error={validationErrors['bucketConfiguration.accountName']}
            />
            <SettingsTextInput
              name="azure-container"
              label={BlobStoresStrings.WIZARD_BASIC.CONTAINER_NAME}
              value={(config as Record<string, string>).containerName || ''}
              onChange={(v) => onChange('bucketConfiguration.containerName', v)}
              placeholder="astrum-blobs"
              helpText={BlobStoresStrings.WIZARD_BASIC.CONTAINER_BLOBS_CREATED_IF_DOES}
              required
              error={validationErrors['bucketConfiguration.containerName']}
            />
            {isProEdition && (
              <div className="azure-blob-store-settings__sas-section">
                <SettingsCheckbox
                  name="azure-direct-download"
                  label={BlobStoresStrings.WIZARD_BASIC.DIRECT_DOWNLOAD_SAS_URLS}
                  description={BlobStoresStrings.WIZARD_BASIC.REDIRECT_DOWNLOADS_DIRECTLY_AZURE_BLOB}
                  checked={config.preSignedUrlEnabled || false}
                  onChange={(v) => onChange('bucketConfiguration.preSignedUrlEnabled', v)}
                />
                {config.preSignedUrlEnabled && (
                  <SettingsAlert type="info">
                    {BlobStoresStrings.WIZARD_BASIC.IDENTITY_NEEDS_AZURE_ROLE_CAN}
                  </SettingsAlert>
                )}
              </div>
            )}
          </>
        )}

        {selectedType === BLOB_STORE_TYPE_IDS.GOOGLE && (
          <>
            <SettingsTextInput
              name="google-project"
              label={BlobStoresStrings.WIZARD_BASIC.PROJECT_ID}
              value={((config as Record<string, unknown>)?.bucket as Record<string, string>)?.projectId || ''}
              onChange={(v) => onChange('bucketConfiguration.bucket.projectId', v)}
              placeholder="my-gcp-project"
              helpText={BlobStoresStrings.WIZARD_BASIC.OPTIONAL_GCP_PROJECT_ID}
            />
            <SettingsTextInput
              name="google-bucket"
              label={BlobStoresStrings.WIZARD_BASIC.BUCKET}
              value={((config as Record<string, unknown>)?.bucket as Record<string, string>)?.name || ''}
              onChange={(v) => onChange('bucketConfiguration.bucket.name', v)}
              placeholder="my-astrum-bucket"
              helpText={BlobStoresStrings.WIZARD_BASIC.GCP_BUCKET_NAME_3_63}
              required
              error={validationErrors['bucketConfiguration.bucket.name']}
            />
            <SettingsTextInput
              name="google-prefix"
              label={BlobStoresStrings.WIZARD_BASIC.PREFIX}
              value={((config as Record<string, unknown>)?.bucket as Record<string, string>)?.prefix || ''}
              onChange={(v) => onChange('bucketConfiguration.bucket.prefix', v)}
              placeholder="astrum/"
              helpText={BlobStoresStrings.WIZARD_BASIC.OPTIONAL_PATH_PREFIX}
            />
          </>
        )}

        {selectedType === BLOB_STORE_TYPE_IDS.GROUP && (
          <>
            <SettingsTransferList
              name="members"
              label={BlobStoresStrings.WIZARD_BASIC.MEMBERS}
              helpText={BlobStoresStrings.WIZARD_BASIC.SELECT_BLOB_STORES_INCLUDE_GROUP}
              availableItems={availableOptions}
              selectedItems={selectedOptions}
              availableLabel={BlobStoresStrings.WIZARD_BASIC.AVAILABLE}
              selectedLabel={BlobStoresStrings.WIZARD_BASIC.SELECTED}
              getItemId={(item) => (item as { value: string }).value}
              getItemLabel={(item) => (item as { label: string }).label}
              onChange={(items) => onChange('members', items.map((i) => (i as { value: string }).value))}
              disabled={groupLoading}
            />
            <SettingsSelect
              name="fill-policy"
              label={BlobStoresStrings.WIZARD_BASIC.FILL_POLICY}
              value={data.fillPolicy || ''}
              onChange={(v) => onChange('fillPolicy', v)}
              options={FILL_POLICY_OPTIONS}
              helpText={BlobStoresStrings.WIZARD_BASIC.HOW_BLOBS_DISTRIBUTED_ACROSS_MEMBERS}
              required
              error={validationErrors.fillPolicy}
              disabled={groupLoading}
            />
          </>
        )}
      </SettingsFormSection>
  );
}
