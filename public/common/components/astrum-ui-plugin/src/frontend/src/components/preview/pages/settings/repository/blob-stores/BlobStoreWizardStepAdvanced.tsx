

import React from 'react';
import { Flex, Text } from '@radix-ui/themes';
import { Plus, Trash2 } from 'lucide-react';
import {
  SettingsFormSection,
  SettingsTextInput,
  SettingsSelect,
  SettingsCheckbox,
  SettingsButton,
} from '../../../../shared/form';
import { ExtJS } from '../../../../../../interface/ExtJS';
import type { BlobStoreFormData, S3BlobStoreConfig, S3FailoverBucket } from './types';
import { BLOB_STORE_TYPE_IDS } from './blobStoreFormMachine';
import type { BlobStoreTypeId } from './BlobStoreTypeSelector';
import BlobStoresStrings from '../../../../constants/pages/settings/repository/BlobStoresStrings';

const SPACE_USED_QUOTA_ID = 'spaceUsedQuota';
const MAX_FAILOVER = 5;

export interface BlobStoreWizardStepAdvancedProps {
  data: BlobStoreFormData;
  selectedType: BlobStoreTypeId;
  onChange: (path: string, value: unknown) => void;
  updateField: (field: string, value: unknown) => void;
  quotaTypes: Array<{ id: string; name: string }>;
  validationErrors: Record<string, string>;
}

export function BlobStoreWizardStepAdvanced({
  data,
  selectedType,
  onChange,
  updateField,
  quotaTypes,
  validationErrors,
}: BlobStoreWizardStepAdvancedProps) {
  const isProEdition = ExtJS.isProEdition();
  const isFailoverAvailable = ExtJS.state().getValue('S3FailoverEnabled', false);

  const config = (data.bucketConfiguration || {}) as S3BlobStoreConfig;
  const adv = config.advancedBucketConnection || {};
  const failoverBuckets: S3FailoverBucket[] = config.failoverBuckets || [];
  const softQuota = data.softQuota || { enabled: false };

  const addFailover = () => {
    onChange('bucketConfiguration.failoverBuckets', [
      ...failoverBuckets,
      { region: '', bucketName: '' },
    ]);
  };

  const removeFailover = (i: number) => {
    onChange(
      'bucketConfiguration.failoverBuckets',
      failoverBuckets.filter((_, idx) => idx !== i)
    );
  };

  const updateFailover = (i: number, field: keyof S3FailoverBucket, value: string) => {
    const next = failoverBuckets.map((b, idx) =>
      idx === i ? { ...b, [field]: value } : b
    );
    onChange('bucketConfiguration.failoverBuckets', next);
  };

  return (
    <>
      {selectedType === BLOB_STORE_TYPE_IDS.S3 && (
        <SettingsFormSection
          title={BlobStoresStrings.WIZARD_ADVANCED.ADVANCED_CONNECTION}
          description={BlobStoresStrings.WIZARD_ADVANCED.CUSTOM_ENDPOINTS_CONNECTION_SETTINGS}
        >
          <SettingsTextInput
            name="s3-endpoint"
            label={BlobStoresStrings.WIZARD_ADVANCED.ENDPOINT_URL}
            value={adv.endpoint || ''}
            onChange={(v) => onChange('bucketConfiguration.advancedBucketConnection.endpoint', v)}
            placeholder="https://s3.example.com"
            helpText={BlobStoresStrings.WIZARD_ADVANCED.CUSTOM_ENDPOINT_S3_COMPATIBLE_STORES}
          />
          <SettingsTextInput
            name="s3-max-pool"
            label={BlobStoresStrings.WIZARD_ADVANCED.MAX_CONNECTION_POOL_SIZE}
            value={adv.maxConnectionPoolSize?.toString() || ''}
            onChange={(v) => {
              const n = parseInt(String(v), 10);
              onChange('bucketConfiguration.advancedBucketConnection.maxConnectionPoolSize', Number.isNaN(n) ? undefined : n);
            }}
            placeholder="100"
            type="number"
          />
          <SettingsCheckbox
            name="s3-path-style"
            label={BlobStoresStrings.WIZARD_ADVANCED.USE_PATH_STYLE_ACCESS}
            description={BlobStoresStrings.WIZARD_ADVANCED.USE_PATH_STYLE_URLS_ALL}
            checked={adv.forcePathStyle}
            onChange={(v) => onChange('bucketConfiguration.advancedBucketConnection.forcePathStyle', v)}
          />
          {isFailoverAvailable && isProEdition && (
            <>
              <Text weight="medium" size="2" mb="2" as="p">
                {BlobStoresStrings.WIZARD_ADVANCED.REPLICATION_BUCKETS}
              </Text>
              {failoverBuckets.map((fb, i) => (
                <Flex key={i} gap="2" mb="2" align="center">
                  <SettingsTextInput
                    name={`failover-region-${i}`}
                    label={BlobStoresStrings.WIZARD_ADVANCED.REGION}
                    value={fb.region}
                    onChange={(v) => updateFailover(i, 'region', String(v))}
                    placeholder="us-east-1"
                  />
                  <SettingsTextInput
                    name={`failover-bucket-${i}`}
                    label={BlobStoresStrings.WIZARD_ADVANCED.BUCKET}
                    value={fb.bucketName}
                    onChange={(v) => updateFailover(i, 'bucketName', String(v))}
                    placeholder="replica-bucket"
                  />
                  <SettingsButton
                    variant="ghost"
                    size="1"
                    onClick={() => removeFailover(i)}
                    icon={Trash2}
                    title={BlobStoresStrings.WIZARD_ADVANCED.REMOVE}
                  />
                </Flex>
              ))}
              {failoverBuckets.length < MAX_FAILOVER && (
                <SettingsButton variant="secondary" size="1" onClick={addFailover} icon={Plus}>
                  {BlobStoresStrings.WIZARD_ADVANCED.ADD_REPLICATION_BUCKET}
                </SettingsButton>
              )}
            </>
          )}
        </SettingsFormSection>
      )}

      <SettingsFormSection
        title={BlobStoresStrings.WIZARD_ADVANCED.SOFT_QUOTA}
        description={BlobStoresStrings.WIZARD_ADVANCED.ALERT_WHEN_STORAGE_EXCEEDS_CONFIGURED}
      >
        <SettingsCheckbox
          name="soft-quota-enabled"
          label={BlobStoresStrings.WIZARD_ADVANCED.ENABLE_SOFT_QUOTA}
          checked={softQuota.enabled}
          onChange={(v) =>
            updateField('softQuota', {
              ...softQuota,
              enabled: v,
              type: v ? softQuota.type || SPACE_USED_QUOTA_ID : undefined,
              limit: v ? (softQuota.limit ?? 1024) : undefined,
            })
          }
        />
        {softQuota.enabled && (
          <>
            <SettingsSelect
              name="soft-quota-type"
              label={BlobStoresStrings.WIZARD_ADVANCED.CONSTRAINT_TYPE}
              value={softQuota.type || ''}
              onChange={(v) => updateField('softQuota', { ...softQuota, type: v })}
              options={[
                { value: '', label: BlobStoresStrings.WIZARD_ADVANCED.SELECT_TYPE },
                ...quotaTypes.map((q) => ({ value: q.id, label: q.name })),
              ]}
              required
              error={validationErrors['softQuota.type']}
            />
            <SettingsTextInput
              name="soft-quota-limit"
              label={BlobStoresStrings.WIZARD_ADVANCED.CONSTRAINT_LIMIT_MB}
              value={softQuota.limit?.toString() || ''}
              onChange={(v) => {
                const n = parseInt(String(v), 10);
                updateField('softQuota', {
                  ...softQuota,
                  limit: Number.isNaN(n) ? undefined : n,
                });
              }}
              placeholder="1024"
              type="number"
              required
              error={validationErrors['softQuota.limit']}
            />
          </>
        )}
      </SettingsFormSection>
    </>
  );
}
