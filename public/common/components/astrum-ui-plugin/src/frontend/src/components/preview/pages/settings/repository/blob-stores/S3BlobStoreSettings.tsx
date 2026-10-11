

import React from 'react';
import { Cloud, Plus, Trash2, AlertTriangle, Info } from 'lucide-react';
import { ExtJS } from '../../../../../../interface/ExtJS';
import {
  SettingsFormSection,
  SettingsTextInput,
  SettingsPasswordInput,
  SettingsSelect,
  SettingsCheckbox,
  SettingsAlert,
  SettingsButton
} from '../../../../shared/form';
import { useS3DropdownValues } from './useBlobStores';
import type { BlobStoreFormData, S3BlobStoreConfig, } from './types';
import './S3BlobStoreSettings.scss';
import BlobStoresStrings from '../../../../constants/pages/settings/repository/BlobStoresStrings';

interface S3BlobStoreSettingsProps {
  data: BlobStoreFormData;
  onChange: (path: string, value: unknown) => void;
  disabled?: boolean;
  isEdit?: boolean;
  errors?: Record<string, string | null>;
}

const STRINGS = {
  BASIC: {
    title: BlobStoresStrings.S3.S3_BUCKET_CONFIGURATION,
    description: BlobStoresStrings.S3.CONFIGURE_AWS_S3_BUCKET_BLOB,
    REGION: {
      label: BlobStoresStrings.S3.REGION,
      helpText: BlobStoresStrings.S3.SELECT_AWS_REGION
    },
    BUCKET: {
      label: BlobStoresStrings.S3.BUCKET,
      helpText: BlobStoresStrings.S3.S3_BUCKET_NAME_MUST_BETWEEN,
      placeholder: 'my-blob-store-bucket'
    },
    PREFIX: {
      label: BlobStoresStrings.S3.PREFIX,
      helpText: BlobStoresStrings.S3.S3_PATH_PREFIX_OPTIONAL,
      placeholder: 'astrum/'
    },
    PRESIGNED: {
      label: BlobStoresStrings.S3.PRE_SIGNED_URL,
      description: BlobStoresStrings.S3.ALLOW_ASTRUM_REPOSITORY_REDIRECT_USERS
    }
  },
  AUTH: {
    title: BlobStoresStrings.S3.AUTHENTICATION_OPTIONAL,
    description: BlobStoresStrings.S3.AWS_CREDENTIALS_S3_ACCESS_LEAVE,
    ACCESS_KEY: {
      label: BlobStoresStrings.S3.ACCESS_KEY_ID,
      placeholder: 'AKIAIOSFODNN7EXAMPLE'
    },
    SECRET_KEY: {
      label: BlobStoresStrings.S3.SECRET_ACCESS_KEY,
      placeholder: '••••••••••••••••••••'
    },
    ROLE_ARN: {
      label: BlobStoresStrings.S3.ASSUME_ROLE_ARN,
      placeholder: 'arn:aws:iam::123456789012:role/S3Access'
    },
    SESSION_TOKEN: {
      label: BlobStoresStrings.S3.SESSION_TOKEN,
      placeholder: BlobStoresStrings.S3.TEMPORARY_SESSION_TOKEN_OPTIONAL
    }
  },
  ENCRYPTION: {
    title: BlobStoresStrings.S3.ENCRYPTION_OPTIONAL,
    description: BlobStoresStrings.S3.CONFIGURE_SERVER_SIDE_ENCRYPTION_OBJECTS,
    TYPE: {
      label: BlobStoresStrings.S3.ENCRYPTION_TYPE,
      helpText: BlobStoresStrings.S3.TYPE_ENCRYPTION_OBJECTS_S3_BLOB
    },
    KEY_ID: {
      label: BlobStoresStrings.S3.KMS_KEY_ID_OPTIONAL,
      helpText: BlobStoresStrings.S3.IF_USING_KMS_ENCRYPTION_YOU,
      placeholder: 'arn:aws:kms:us-east-1:123456789012:key/...'
    }
  },
  ADVANCED: {
    title: BlobStoresStrings.S3.ADVANCED_CONNECTION_SETTINGS_OPTIONAL,
    description: BlobStoresStrings.S3.CONFIGURE_CUSTOM_ENDPOINTS_CONNECTION_SETTINGS,
    ENDPOINT: {
      label: BlobStoresStrings.S3.ENDPOINT_URL,
      helpText: BlobStoresStrings.S3.CUSTOM_ENDPOINT_URL_THIRD_PARTY,
      placeholder: 'https://s3.example.com'
    },
    MAX_CONNECTIONS: {
      label: BlobStoresStrings.S3.MAX_CONNECTION_POOL_SIZE,
      helpText: BlobStoresStrings.S3.OVERRIDES_DEFAULT_CONNECTION_POOL_SIZE,
      placeholder: '100'
    },
    PATH_STYLE: {
      label: BlobStoresStrings.S3.USE_PATH_STYLE_ACCESS,
      description: BlobStoresStrings.S3.SETTING_FLAG_RESULT_PATH_STYLE
    }
  },
  FAILOVER: {
    title: BlobStoresStrings.S3.AWS_S3_REPLICATION_BUCKETS_OPTIONAL,
    description: BlobStoresStrings.S3.CONFIGURE_FAILOVER_BUCKETS_HIGH_AVAILABILITY,
    ADD_BUTTON: BlobStoresStrings.S3.ADD_REPLICATION_BUCKET,
    REMOVE_BUTTON: BlobStoresStrings.S3.REMOVE,
    MAX_WARNING: BlobStoresStrings.S3.YOU_HAVE_REACHED_MAXIMUM_NUMBER,
    REGION: {
      label: BlobStoresStrings.S3.REGION,
      helpText: BlobStoresStrings.S3.SELECT_AWS_REGION
    },
    BUCKET: {
      label: BlobStoresStrings.S3.BUCKET_NAME,
      helpText: BlobStoresStrings.S3.S3_BUCKET_NAME_REPLICATION
    }
  },
  HELP: BlobStoresStrings.S3.S3_BLOB_STORES_REQUIRE_SPECIFIC
};

const MAX_FAILOVER_BUCKETS = 5;

export default function S3BlobStoreSettings({
  data,
  onChange,
  disabled = false,
  isEdit = false,
  errors = {},
}: S3BlobStoreSettingsProps) {
  const { values: dropdownValues, loading: dropdownLoading } = useS3DropdownValues();

  const isProEdition = ExtJS.isProEdition();
  const isFailoverAvailable = ExtJS.state().getValue('S3FailoverEnabled', false);

  const config: S3BlobStoreConfig = (data.bucketConfiguration as S3BlobStoreConfig) || {
    bucket: { region: '', name: '' },
    bucketSecurity: {},
    encryption: {},
    advancedBucketConnection: {},
    failoverBuckets: []
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

  const updateAdvanced = (field: string, value: unknown) => {
    onChange(`bucketConfiguration.advancedBucketConnection.${field}`, value);
  };

  const addFailoverBucket = () => {
    const current = config.failoverBuckets || [];
    onChange('bucketConfiguration.failoverBuckets', [...current, { region: '', bucketName: '' }]);
  };

  const removeFailoverBucket = (index: number) => {
    const current = config.failoverBuckets || [];
    onChange('bucketConfiguration.failoverBuckets', current.filter((_, i) => i !== index));
  };

  const updateFailoverBucket = (index: number, field: string, value: string) => {
    const current = config.failoverBuckets || [];
    const updated = current.map((bucket, i) =>
      i === index ? { ...bucket, [field]: value } : bucket
    );
    onChange('bucketConfiguration.failoverBuckets', updated);
  };

  const regions = dropdownValues?.regions || [];
  const encryptionTypes = dropdownValues?.encryptionTypes || [];
  const hasAuth = Boolean(config.bucketSecurity?.accessKeyId);
  const hasEncryption = Boolean(config.encryption?.encryptionType || config.encryption?.encryptionKey);
  const hasAdvanced = Boolean(config.advancedBucketConnection?.endpoint || config.advancedBucketConnection?.forcePathStyle);
  const hasFailover = (config.failoverBuckets?.length || 0) > 0;

  return (
    <div className="s3-blob-store-settings">
      <SettingsAlert variant="info" icon={<Info size={16} />}>
        {STRINGS.HELP}
        <a
          href="https://links.globalart.dev/products/astrum/blobstores/s3/docs"
          target="_blank"
          rel="noopener noreferrer"
          className="s3-blob-store-settings__link"
        >
          {BlobStoresStrings.S3.VIEW_DOCUMENTATION}
        </a>
      </SettingsAlert>

      {/* Basic Configuration */}
      <SettingsFormSection
        title={STRINGS.BASIC.title}
        description={STRINGS.BASIC.description}
        icon={<Cloud size={20} />}
        collapsible
        defaultExpanded
      >
        <SettingsSelect
          name="s3-region"
          label={STRINGS.BASIC.REGION.label}
          value={config.bucket?.region || ''}
          onChange={(value) => updateBucket('region', value)}
          options={[
            { value: '', label: BlobStoresStrings.S3.SELECT_REGION },
            ...regions.map(r => ({ value: r.id, label: r.name }))
          ]}
          helpText={STRINGS.BASIC.REGION.helpText}
          disabled={disabled || dropdownLoading}
        />

        <SettingsTextInput
          name="s3-bucket"
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
          name="s3-prefix"
          label={STRINGS.BASIC.PREFIX.label}
          value={config.bucket?.prefix || ''}
          onChange={(value) => updateBucket('prefix', value)}
          helpText={STRINGS.BASIC.PREFIX.helpText}
          placeholder={STRINGS.BASIC.PREFIX.placeholder}
          disabled={disabled}
        />

        {isProEdition && (
          <SettingsCheckbox
            name="s3-presigned"
            label={STRINGS.BASIC.PRESIGNED.label}
            description={STRINGS.BASIC.PRESIGNED.description}
            checked={config.preSignedUrlEnabled || false}
            onChange={(checked) => onChange('bucketConfiguration.preSignedUrlEnabled', checked)}
            disabled={disabled}
          />
        )}
      </SettingsFormSection>

      {/* Authentication */}
      <SettingsFormSection
        title={STRINGS.AUTH.title}
        description={STRINGS.AUTH.description}
        collapsible
        defaultExpanded={hasAuth}
      >
        <SettingsTextInput
          name="s3-access-key"
          label={STRINGS.AUTH.ACCESS_KEY.label}
          value={config.bucketSecurity?.accessKeyId || ''}
          onChange={(value) => updateSecurity('accessKeyId', value)}
          placeholder={STRINGS.AUTH.ACCESS_KEY.placeholder}
          disabled={disabled}
        />

        <SettingsPasswordInput
          name="s3-secret-key"
          label={STRINGS.AUTH.SECRET_KEY.label}
          value={config.bucketSecurity?.secretAccessKey || ''}
          onChange={(value) => updateSecurity('secretAccessKey', value)}
          placeholder={STRINGS.AUTH.SECRET_KEY.placeholder}
          disabled={disabled}
          autoComplete="new-password"
        />

        <SettingsTextInput
          name="s3-role-arn"
          label={STRINGS.AUTH.ROLE_ARN.label}
          value={config.bucketSecurity?.role || ''}
          onChange={(value) => updateSecurity('role', value)}
          placeholder={STRINGS.AUTH.ROLE_ARN.placeholder}
          disabled={disabled}
        />

        <SettingsPasswordInput
          name="s3-session-token"
          label={STRINGS.AUTH.SESSION_TOKEN.label}
          value={config.bucketSecurity?.sessionToken || ''}
          onChange={(value) => updateSecurity('sessionToken', value)}
          placeholder={STRINGS.AUTH.SESSION_TOKEN.placeholder}
          disabled={disabled}
        />
      </SettingsFormSection>

      {/* Encryption */}
      <SettingsFormSection
        title={STRINGS.ENCRYPTION.title}
        description={STRINGS.ENCRYPTION.description}
        collapsible
        defaultExpanded={hasEncryption}
      >
        <SettingsSelect
          label={STRINGS.ENCRYPTION.TYPE.label}
          value={config.encryption?.encryptionType || ''}
          onChange={(value) => updateEncryption('encryptionType', value)}
          options={[
            { value: '', label: BlobStoresStrings.S3.NONE },
            ...encryptionTypes.map(e => ({ value: e.id, label: e.name }))
          ]}
          helpText={STRINGS.ENCRYPTION.TYPE.helpText}
          disabled={disabled || dropdownLoading}
        />

        <SettingsTextInput
          label={STRINGS.ENCRYPTION.KEY_ID.label}
          value={config.encryption?.encryptionKey || ''}
          onChange={(value) => updateEncryption('encryptionKey', value)}
          helpText={STRINGS.ENCRYPTION.KEY_ID.helpText}
          placeholder={STRINGS.ENCRYPTION.KEY_ID.placeholder}
          disabled={disabled}
          monospace
        />
      </SettingsFormSection>

      {/* Advanced Settings */}
      <SettingsFormSection
        title={STRINGS.ADVANCED.title}
        description={STRINGS.ADVANCED.description}
        collapsible
        defaultExpanded={hasAdvanced}
      >
        <SettingsTextInput
          label={STRINGS.ADVANCED.ENDPOINT.label}
          value={config.advancedBucketConnection?.endpoint || ''}
          onChange={(value) => updateAdvanced('endpoint', value)}
          helpText={STRINGS.ADVANCED.ENDPOINT.helpText}
          placeholder={STRINGS.ADVANCED.ENDPOINT.placeholder}
          disabled={disabled}
        />

        <SettingsTextInput
          label={STRINGS.ADVANCED.MAX_CONNECTIONS.label}
          value={config.advancedBucketConnection?.maxConnectionPoolSize?.toString() || ''}
          onChange={(value) => updateAdvanced('maxConnectionPoolSize', value ? parseInt(value, 10) : undefined)}
          helpText={STRINGS.ADVANCED.MAX_CONNECTIONS.helpText}
          placeholder={STRINGS.ADVANCED.MAX_CONNECTIONS.placeholder}
          type="number"
          disabled={disabled}
        />

        <SettingsCheckbox
          label={STRINGS.ADVANCED.PATH_STYLE.label}
          description={STRINGS.ADVANCED.PATH_STYLE.description}
          checked={config.advancedBucketConnection?.forcePathStyle || false}
          onChange={(checked) => updateAdvanced('forcePathStyle', checked)}
          disabled={disabled}
        />
      </SettingsFormSection>

      {/* Failover Buckets */}
      {isFailoverAvailable && (
        <SettingsFormSection
          title={STRINGS.FAILOVER.title}
          description={STRINGS.FAILOVER.description}
          collapsible
          defaultExpanded={hasFailover}
        >
          {config.failoverBuckets?.map((bucket, index) => (
            <div key={index} className="s3-blob-store-settings__failover-bucket">
              <div className="s3-blob-store-settings__failover-fields">
                <SettingsSelect
                  label={`${STRINGS.FAILOVER.REGION.label} ${index + 1}`}
                  value={bucket.region || ''}
                  onChange={(value) => updateFailoverBucket(index, 'region', value)}
                  options={[
                    { value: '', label: BlobStoresStrings.S3.SELECT_REGION },
                    ...regions.map(r => ({ value: r.id, label: r.name }))
                  ]}
                  helpText={STRINGS.FAILOVER.REGION.helpText}
                  disabled={disabled || dropdownLoading}
                />

                <SettingsTextInput
                  label={`${STRINGS.FAILOVER.BUCKET.label} ${index + 1}`}
                  value={bucket.bucketName || ''}
                  onChange={(value) => updateFailoverBucket(index, 'bucketName', value)}
                  helpText={STRINGS.FAILOVER.BUCKET.helpText}
                  disabled={disabled}
                />
              </div>

              <SettingsButton
                variant="ghost"
                onClick={() => removeFailoverBucket(index)}
                disabled={disabled}
                icon={Trash2}
                className="s3-blob-store-settings__remove-btn"
              >
                {STRINGS.FAILOVER.REMOVE_BUTTON}
              </SettingsButton>
            </div>
          ))}

          {(config.failoverBuckets?.length || 0) < MAX_FAILOVER_BUCKETS ? (
            <SettingsButton
              variant="secondary"
              onClick={addFailoverBucket}
              disabled={disabled}
              icon={Plus}
            >
              {STRINGS.FAILOVER.ADD_BUTTON}
            </SettingsButton>
          ) : (
            <SettingsAlert variant="warning" icon={<AlertTriangle size={16} />}>
              {STRINGS.FAILOVER.MAX_WARNING}
            </SettingsAlert>
          )}
        </SettingsFormSection>
      )}
    </div>
  );
}

