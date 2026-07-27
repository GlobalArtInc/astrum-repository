
import {
  NxButton,
  NxDivider,
  NxFontAwesomeIcon,
  NxFormGroup,
  NxFormSelect,
  NxInfoAlert,
  NxReadOnly,
  NxTextInput
} from '@globalart/react-shared-components';
import React from 'react';
import {faCheckCircle, faTrash} from '@fortawesome/free-solid-svg-icons';
import UIStrings from '../../../../../constants/UIStrings';

const FIELDS = UIStrings.S3_BLOBSTORE_CONFIGURATION;

export default function S3FailoverBucket({
                                           index,
                                           activeRegion,
                                           mainRegionProps,
                                           regionProps,
                                           onRegionChange,
                                           dropDownValues,
                                           selectedRegions,
                                           bucketNameProps,
                                           onBucketNameChange,
                                           onDelete,
                                         }) {
  const showRegionFailoverActiveMessage = activeRegion === regionProps.value;
  const isInvalidRegion = Boolean(regionProps.validationErrors)
  const notPristine = Boolean(!regionProps.isPristine) || Boolean(!mainRegionProps.isPristine)
  const showInvalidRegion = isInvalidRegion && notPristine;
  regionProps = {...regionProps, isPristine: !notPristine}

  return <div id={`replication-bucket-${index}`}>
    <NxFormGroup
        label={FIELDS.S3BlobStore_ReplicationBucketsSettings_Region_FieldLabel}
        sublabel={FIELDS.S3BlobStore_ReplicationBucketsSettings_Region_HelpText}
        className={showRegionFailoverActiveMessage ? "as-s3-region-select-message" : ""}
        isRequired>
      <NxFormSelect {...regionProps}
                    onChange={onRegionChange}>
        <option value={null}></option>
        {dropDownValues?.regions?.filter(region => region.id !== 'DEFAULT' &&
            (!selectedRegions.includes(region.id) || region.id === regionProps.value))
            .map(region => <option key={region.id} value={region.id}>{region.name}</option>)}
      </NxFormSelect>
    </NxFormGroup>
    {showRegionFailoverActiveMessage &&
        <NxReadOnly id="active-failover-region"
                    className={`as-active-failover-region-message ${showInvalidRegion ? "invalid" : ""}`}>
          <NxFontAwesomeIcon icon={faCheckCircle}/>
          <NxReadOnly.Data>{FIELDS.S3BlobStore_Region_FailoverRegionActiveText}</NxReadOnly.Data>
        </NxReadOnly>
    }
    <NxFormGroup
        label={FIELDS.S3BlobStore_ReplicationBucketsSettings_BucketName_FieldLabel}
        sublabel={FIELDS.S3BlobStore_ReplicationBucketsSettings_BucketName_HelpText}
        isRequired>
      <NxTextInput className="nx-text-input--long"
                   {...bucketNameProps}
                   onChange={onBucketNameChange}/>
    </NxFormGroup>
    <NxInfoAlert><span>{FIELDS.S3BlobStore_ReplicationBucketsSettings_ConfigureBucketReplicationMessage}</span>
    </NxInfoAlert>
    <div id="delete-failover-bucket">
      <NxButton
          type="button"
          variant="tertiary"
          className="as-replication-bucket-button"
          onClick={() => onDelete(index)}>
        <NxFontAwesomeIcon icon={faTrash}/>
        <span>{FIELDS.S3BlobStore_ReplicationBucketsSettings_DeleteFailoverBucket}</span>
      </NxButton>
    </div>
    <NxDivider/>
  </div>;
}
