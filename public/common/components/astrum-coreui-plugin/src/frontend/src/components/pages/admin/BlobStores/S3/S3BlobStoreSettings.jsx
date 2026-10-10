
import React from 'react';
import {useActor} from '@xstate/react';

import {ExtJS, FormUtils} from '@globalart/astrum-ui-plugin';

import {
  AsAccordion,
  AsButton,
  AsCheckbox,
  AsFieldset,
  AsFontAwesomeIcon,
  AsFormGroup,
  AsFormSelect,
  AsReadOnly,
  AsStatefulAccordion,
  AsTextInput,
  AsWarningAlert
} from '@globalart/astrum-react-shared-components';

import UIStrings from '../../../../../constants/UIStrings'

import './S3BlobStoreSettings.scss';
import {faExclamationCircle, faPlus} from '@fortawesome/free-solid-svg-icons';
import S3FailoverBucket from './S3FailoverBucket';

const FIELDS = UIStrings.S3_BLOBSTORE_CONFIGURATION;
const MAX_REPLICATION_BUCKETS = 5;

export default function S3BlobStoreSettings({service}) {
  const [current, send] = useActor(service);
  const {bucketConfiguration} = current.context.data;
  const pristineBucketConfig = current.context.pristineData.bucketConfiguration;
  const activeRegion = pristineBucketConfig.activeRegion;
  const hasActiveRegion = Boolean(activeRegion);
  const {dropDownValues} = current.context.type;
  const hasAuthenticationSettings = Boolean(bucketConfiguration.bucketSecurity?.accessKeyId);
  const hasEncryptionSettings =
      Boolean(bucketConfiguration.encryption?.encryptionType) ||
      Boolean(bucketConfiguration.encryption?.encryptionKey);
  const hasAdvancedConnectionSettings =
      Boolean(bucketConfiguration.advancedBucketConnection?.endpoint) ||
      Boolean(bucketConfiguration.advancedBucketConnection?.signerType) ||
      bucketConfiguration.advancedBucketConnection?.forcePathStyle;
  const hasFailoverBucketSettings =
      Boolean(bucketConfiguration.failoverBuckets?.length > 0);
  const isPrimaryRegionActive = activeRegion === pristineBucketConfig.bucket?.region;
  const isFailoverAvailable = ExtJS.state().getValue('S3FailoverEnabled', false);
  const showPrimaryRegionInactiveMessage = hasActiveRegion && isFailoverAvailable && !isPrimaryRegionActive;
  const isProEdition = ExtJS.isProEdition();

  function bucketConfigurationField(field) {
    return `bucketConfiguration.${field}`;
  }

  function bucketField(field) {
    return `bucketConfiguration.bucket.${field}`;
  }

  function securityField(field) {
    return `bucketConfiguration.bucketSecurity.${field}`;
  }

  function encryptionField(field) {
    return `bucketConfiguration.encryption.${field}`;
  }

  function advancedField(field) {
    return `bucketConfiguration.advancedBucketConnection.${field}`;
  }

  function failoverBucketsField(index, field) {
    return ['bucketConfiguration', 'failoverBuckets', index, field];
  }

  function handleAddFailoverBucket() {
    send({
      type: 'UPDATE',
      name: 'bucketConfiguration.failoverBuckets',
      value: bucketConfiguration.failoverBuckets.concat({})
    });
  }

  function handleDeleteFailoverBucket(index) {
    send({
      type: 'UPDATE',
      name: 'bucketConfiguration.failoverBuckets',
      value: bucketConfiguration.failoverBuckets.filter((_, i) => i !== index)
    });
  }

  function getSelectedRegions() {
    const principalRegion = current.context.data.bucketConfiguration?.bucket?.region;
    const failoverRegions = current.context.data.bucketConfiguration?.failoverBuckets?.map(bucket => bucket.region);
    return [principalRegion, ...failoverRegions];
  }

  return <div className="as-s3-blobstore">
    { (isFailoverAvailable && hasActiveRegion) &&
        <AsReadOnly>
          <AsReadOnly.Label>{FIELDS.S3BlobStore_RegionStatus_FieldLabel}</AsReadOnly.Label>
          <div className="as-active-region-section">
            <AsReadOnly.Data>{FIELDS.S3BlobStore_RegionStatus_InUseText}</AsReadOnly.Data>
            <AsReadOnly.Data className="s3-in-use-region">{activeRegion}</AsReadOnly.Data>
            <AsReadOnly.Data>
              {isPrimaryRegionActive ? FIELDS.S3BlobStore_RegionStatus_PrimaryRegionText : FIELDS.S3BlobStore_RegionStatus_FailoverRegionText}
            </AsReadOnly.Data>
          </div>
        </AsReadOnly>
    }
    <AsFormGroup
        label={FIELDS.S3BlobStore_Region_FieldLabel}
        sublabel={FIELDS.S3BlobStore_Region_HelpText}
        className={showPrimaryRegionInactiveMessage ? "as-s3-region-select-message": ""}
        data-testid="s3-primary-region"
    >
      <AsFormSelect {...FormUtils.fieldProps(bucketField('region'), current)}
                    onChange={FormUtils.handleUpdate(bucketField('region'), send)}>
        {dropDownValues?.regions?.map(region =>
            <option key={region.id} value={region.id}>{region.name}</option>
        )}
      </AsFormSelect>
    </AsFormGroup>
    {showPrimaryRegionInactiveMessage &&
        <AsReadOnly id="as-inactive-region" className="as-inactive-region-message">
          <AsFontAwesomeIcon icon={faExclamationCircle}/>
          <AsReadOnly.Data>{FIELDS.S3BlobStore_Region_PrimaryRegionInactiveText}</AsReadOnly.Data>
        </AsReadOnly>
    }
    <AsFormGroup
        label={FIELDS.S3BlobStore_Bucket_FieldLabel}
        sublabel={FIELDS.S3BlobStore_Bucket_HelpText}
        isRequired
    >
      <AsTextInput className="as-text-input--long"
                 {...FormUtils.fieldProps(bucketField('name'), current)}
                 onChange={FormUtils.handleUpdate(bucketField('name'), send)}/>
    </AsFormGroup>
    <AsFormGroup
        label={FIELDS.S3BlobStore_Prefix_FieldLabel}
        sublabel={FIELDS.S3BlobStore_Prefix_HelpText}
    >
      <AsTextInput className="as-text-input--long"
                 {...FormUtils.fieldProps(bucketField('prefix'), current)}
                 onChange={FormUtils.handleUpdate(bucketField('prefix'), send)}/>
    </AsFormGroup>
    {isProEdition &&
        <div className="as-s3-presigned">
          <AsFieldset label={FIELDS.S3BlobStore_Presigned_FieldLabel}>
            <AsCheckbox {...FormUtils.checkboxProps(bucketConfigurationField('preSignedUrlEnabled'), current)}
                        onChange={FormUtils.handleUpdate(bucketConfigurationField('preSignedUrlEnabled'), send)}
                        className="as-text-input--long">
              {FIELDS.S3BlobStore_Presigned_HelpText}
            </AsCheckbox>
          </AsFieldset>
        </div>
    }

      <AsFieldset>
          <h3>{FIELDS.S3BlobStore_Authentication_Title}</h3>
          <AsFormGroup label={FIELDS.S3BlobStore_Authentication_AccessKeyId}>
              <AsTextInput className="as-text-input--long"
                           {...FormUtils.fieldProps(securityField('accessKeyId'), current)}
                           onChange={FormUtils.handleUpdate(securityField('accessKeyId'), send)}/>
          </AsFormGroup>
          <AsFormGroup label={FIELDS.S3BlobStore_Authentication_SecretAccessKey}>
              <AsTextInput className="as-text-input--long"
                           type="password"
                           autoComplete="new-password"
                           {...FormUtils.fieldProps(securityField('secretAccessKey'), current)}
                           onChange={FormUtils.handleUpdate(securityField('secretAccessKey'), send)}/>
          </AsFormGroup>
          <AsFormGroup
              label={FIELDS.S3BlobStore_Authentication_AssumeRoleArn}
          >
              <AsTextInput className="as-text-input--long"
                           {...FormUtils.fieldProps(securityField('role'), current)}
                           onChange={FormUtils.handleUpdate(securityField('role'), send)}/>
          </AsFormGroup>
          <AsFormGroup label={FIELDS.S3BlobStore_Authentication_SessionToken}>
              <AsTextInput className="as-text-input--long"
                           type="password"
                           {...FormUtils.fieldProps(securityField('sessionToken'), current)}
                           onChange={FormUtils.handleUpdate(securityField('sessionToken'), send)}/>
          </AsFormGroup>
      </AsFieldset>
    <AsStatefulAccordion defaultOpen={hasEncryptionSettings}>
      <AsAccordion.Header>
        <h2 className="as-accordion__header-title">{FIELDS.S3BlobStore_EncryptionSettings_Title}</h2>
      </AsAccordion.Header>
      <AsFormGroup
          label={FIELDS.S3BlobStore_EncryptionSettings_Type_FieldLabel}
          sublabel={FIELDS.S3BlobStore_EncryptionSettings_Type_HelpText}
      >
        <AsFormSelect {...FormUtils.fieldProps(encryptionField('encryptionType'), current)}
                onChange={FormUtils.handleUpdate(encryptionField('encryptionType'), send)}>
          <option value={null}></option>
          {dropDownValues?.encryptionTypes?.map(encryptionType =>
              <option key={encryptionType.id} value={encryptionType.id}>{encryptionType.name}</option>
          )}
        </AsFormSelect>
      </AsFormGroup>
      <AsFormGroup
          label={FIELDS.S3BlobStore_EncryptionSettings_KeyID_FieldLabel}
          sublabel={FIELDS.S3BlobStore_EncryptionSettings_KeyID_HelpText}
      >
        <AsTextInput className="as-text-input--long"
                   {...FormUtils.fieldProps(encryptionField('encryptionKey'), current)}
                   onChange={FormUtils.handleUpdate(encryptionField('encryptionKey'), send)}/>
      </AsFormGroup>
    </AsStatefulAccordion>
    <AsStatefulAccordion defaultOpen={hasAdvancedConnectionSettings}>
      <AsAccordion.Header>
        <h2 className="as-accordion__header-title">{FIELDS.S3BlobStore_AdvancedConnectionSettings_Title}</h2>
      </AsAccordion.Header>
      <AsFormGroup
          label={FIELDS.S3BlobStore_AdvancedConnectionSettings_EndPointUrl_FieldLabel}
          sublabel={FIELDS.S3BlobStore_AdvancedConnectionSettings_EndPointUrl_HelpText}
      >
        <AsTextInput className="as-text-input--long"
                   {...FormUtils.fieldProps(advancedField('endpoint'), current)}
                   onChange={FormUtils.handleUpdate(advancedField('endpoint'), send)}/>
      </AsFormGroup>
      <AsFormGroup
        label={FIELDS.S3Blobstore_AdvancedConnectionSettings_MaxConnectionPoolSize_FieldLabel}
        sublabel={FIELDS.S3Blobstore_AdvancedConnectionSettings_MaxConnectionPoolSize_HelpText}
      >
        <AsTextInput {...FormUtils.fieldProps(advancedField('maxConnectionPoolSize'), current)}
          onChange={FormUtils.handleUpdate(advancedField('maxConnectionPoolSize'), send)} />
      </AsFormGroup>
      <AsFieldset label={FIELDS.S3BlobStore_AdvancedConnectionSettings_PathStyleAccess_FieldLabel}>
        <AsCheckbox {...FormUtils.checkboxProps(advancedField('forcePathStyle'), current)}
                    onChange={FormUtils.handleUpdate(advancedField('forcePathStyle'), send)}>
          {FIELDS.S3BlobStore_AdvancedConnectionSettings_PathStyleAccess_HelpText}
        </AsCheckbox>
      </AsFieldset>
    </AsStatefulAccordion>
    {isFailoverAvailable &&
        <AsStatefulAccordion defaultOpen={hasFailoverBucketSettings}>
          <AsAccordion.Header>
            <h2 className="as-accordion__header-title">{FIELDS.S3BlobStore_ReplicationBucketsSettings_Title}</h2>
          </AsAccordion.Header>
          {bucketConfiguration.failoverBuckets?.map((bucket, index) =>
              <S3FailoverBucket
                  key={`failover-bucket-${index}`}
                  index={index}
                  activeRegion={activeRegion}
                  mainRegionProps={FormUtils.fieldProps(bucketField('region'), current)}
                  regionProps={FormUtils.fieldProps(failoverBucketsField(index, 'region'), current, [])}
                  onRegionChange={FormUtils.handleUpdate(failoverBucketsField(index, 'region'), send)}
                  dropDownValues={dropDownValues}
                  selectedRegions={getSelectedRegions()}
                  bucketNameProps={FormUtils.fieldProps(failoverBucketsField(index, 'bucketName'), current, [])}
                  onBucketNameChange={FormUtils.handleUpdate(failoverBucketsField(index, 'bucketName'), send)}
                  onDelete={handleDeleteFailoverBucket}/>)
          }
          {bucketConfiguration.failoverBuckets.length < MAX_REPLICATION_BUCKETS ? (
                  <div id="add-failover-bucket">
                    <AsButton
                        type="button"
                        className="as-replication-bucket-button"
                        onClick={handleAddFailoverBucket}>
                      <AsFontAwesomeIcon icon={faPlus}/>
                      <span>{FIELDS.S3BlobStore_ReplicationBucketsSettings_AddFailoverBucket}</span>
                    </AsButton>
                  </div>) :
              <AsWarningAlert>{FIELDS.S3BlobStore_ReplicationBucketsSettings_MaxFailoverBucketsWarning}</AsWarningAlert>
          }
        </AsStatefulAccordion>}
  </div>;
}
