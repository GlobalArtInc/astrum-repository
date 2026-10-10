
import {
  AsButton,
  AsDivider,
  AsFontAwesomeIcon,
  AsFormGroup,
  AsFormSelect,
  AsInfoAlert,
  AsReadOnly,
  AsTextInput
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
    <AsFormGroup
        label={FIELDS.S3BlobStore_ReplicationBucketsSettings_Region_FieldLabel}
        sublabel={FIELDS.S3BlobStore_ReplicationBucketsSettings_Region_HelpText}
        className={showRegionFailoverActiveMessage ? "as-s3-region-select-message" : ""}
        isRequired>
      <AsFormSelect {...regionProps}
                    onChange={onRegionChange}>
        <option value={null}></option>
        {dropDownValues?.regions?.filter(region => region.id !== 'DEFAULT' &&
            (!selectedRegions.includes(region.id) || region.id === regionProps.value))
            .map(region => <option key={region.id} value={region.id}>{region.name}</option>)}
      </AsFormSelect>
    </AsFormGroup>
    {showRegionFailoverActiveMessage &&
        <AsReadOnly id="active-failover-region"
                    className={`as-active-failover-region-message ${showInvalidRegion ? "invalid" : ""}`}>
          <AsFontAwesomeIcon icon={faCheckCircle}/>
          <AsReadOnly.Data>{FIELDS.S3BlobStore_Region_FailoverRegionActiveText}</AsReadOnly.Data>
        </AsReadOnly>
    }
    <AsFormGroup
        label={FIELDS.S3BlobStore_ReplicationBucketsSettings_BucketName_FieldLabel}
        sublabel={FIELDS.S3BlobStore_ReplicationBucketsSettings_BucketName_HelpText}
        isRequired>
      <AsTextInput className="as-text-input--long"
                   {...bucketNameProps}
                   onChange={onBucketNameChange}/>
    </AsFormGroup>
    <AsInfoAlert><span>{FIELDS.S3BlobStore_ReplicationBucketsSettings_ConfigureBucketReplicationMessage}</span>
    </AsInfoAlert>
    <div id="delete-failover-bucket">
      <AsButton
          type="button"
          variant="tertiary"
          className="as-replication-bucket-button"
          onClick={() => onDelete(index)}>
        <AsFontAwesomeIcon icon={faTrash}/>
        <span>{FIELDS.S3BlobStore_ReplicationBucketsSettings_DeleteFailoverBucket}</span>
      </AsButton>
    </div>
    <AsDivider/>
  </div>;
}
