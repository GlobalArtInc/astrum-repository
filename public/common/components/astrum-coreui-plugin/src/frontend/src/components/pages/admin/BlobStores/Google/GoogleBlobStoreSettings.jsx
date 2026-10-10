
import React, {useState, useEffect} from 'react';
import {useActor} from '@xstate/react';
import {FormUtils} from '@globalart/astrum-ui-plugin';
import {
  AsFieldset,
  AsFileUpload,
  AsFormGroup,
  AsRadio,
  AsTextInput,
  AsCheckbox,
  AsP
} from '@globalart/astrum-react-shared-components';

import UIStrings from '../../../../../constants/UIStrings';

const GOOGLE = UIStrings.BLOB_STORES.GOOGLE;

export default function GoogleBlobStoreSettings({ service }) {
  const [current, send] = useActor(service);
  const { bucketConfiguration = {} } = current.context.data;
  const { pristineData } = current.context;

  function bucketField(field) {
    return `bucketConfiguration.bucket.${field}`;
  }

  function authenticationField(field) {
    return `bucketConfiguration.bucketSecurity.${field}`;
  }

  function encryptionField(field) {
    return `bucketConfiguration.encryption.${field}`;
  }

  const setFiles = (file) => {
    send({ type: 'SET_FILES', file });
  };

  const isEdit = Boolean(pristineData.name);

  const [isChecked, setIsChecked] = useState(bucketConfiguration.encryption?.encryptionType === 'kmsManagedEncryption');

  useEffect(() => {
    setIsChecked(bucketConfiguration.encryption?.encryptionType === 'kmsManagedEncryption');
  }, [bucketConfiguration.encryption?.encryptionType]);

  const handleCheckboxChange = (isChecked) => {
    setIsChecked(isChecked);
    send({
      type: 'UPDATE',
      name: encryptionField('encryptionType'),
      value: isChecked ? 'kmsManagedEncryption' : 'default'
    });
  };

  return (
      <div className="as-google-blobstore">
        <AsFormGroup {...GOOGLE.REGION}>
          <AsP className="as-sub-label" aria-label={GOOGLE.REGION.label}>
            The region is automatically set based on where Astrum Repository is running in GCP. Ensure the bucket is in the same region.
          </AsP>
        </AsFormGroup>
        <AsFormGroup {...GOOGLE.PROJECT_ID}>
          <AsTextInput
              {...FormUtils.fieldProps(bucketField('projectId'), current)}
              onChange={FormUtils.handleUpdate(bucketField('projectId'), send)}
          />
        </AsFormGroup>
        <AsFormGroup {...GOOGLE.BUCKET} isRequired>
          <AsTextInput
              {...FormUtils.fieldProps(bucketField('name'), current)}
          onChange={FormUtils.handleUpdate(bucketField('name'), send)}/>
        </AsFormGroup>
        <AsFormGroup {...GOOGLE.PREFIX}>
          <AsTextInput
              {...FormUtils.fieldProps(bucketField('prefix'), current)}
                 onChange={FormUtils.handleUpdate(bucketField('prefix'), send)}/>
        </AsFormGroup>
        <AsFieldset label={GOOGLE.AUTHENTICATION.LABEL}>
          <AsRadio
              radioId="applicationDefaultCredentials"
              name="bucketConfiguration.bucketSecurity.authenticationMethod"
              value="applicationDefault"
              isChecked={bucketConfiguration.bucketSecurity?.authenticationMethod === 'applicationDefault'}
          onChange={FormUtils.handleUpdate(authenticationField('authenticationMethod'), send)}>
            {GOOGLE.AUTHENTICATION.APPLICATION_DEFAULT_CREDENTIALS}
          </AsRadio>
          <AsRadio
              radioId="credentialJSONFile"
              name="bucketConfiguration.bucketSecurity.authenticationMethod"
              value="accountKey"
              isChecked={bucketConfiguration.bucketSecurity?.authenticationMethod === 'accountKey'}
          onChange={FormUtils.handleUpdate(authenticationField('authenticationMethod'), send)}>
            {GOOGLE.AUTHENTICATION.CREDENTIAL_JSON_FILE}
          </AsRadio>
        </AsFieldset>
        {bucketConfiguration.bucketSecurity?.authenticationMethod === 'accountKey' && (
            <AsFormGroup {...GOOGLE.AUTHENTICATION.JSON_PATH} isRequired={!isEdit}>
              <AsFileUpload
                  onChange={setFiles}
                  isRequired={!isEdit}
                  aria-label="gcp credential json file upload"
                  {...FormUtils.fileUploadProps(authenticationField('file'), current)}
              />
            </AsFormGroup>
        )}
        <AsFieldset label={GOOGLE.ENCRYPTION.LABEL}>
          <p>Data is encrypted by default. Enable KMS to use a custom encryption key.<br />Note: Encryption settings cannot be changed once the bucket is created</p>
          <AsCheckbox
              checkboxId="encryptionKey"
              name="bucketConfiguration.encryption.encryptionType"
              isChecked={isChecked}
              onChange={handleCheckboxChange}
          >
            {GOOGLE.ENCRYPTION.KMS_MANAGED}
          </AsCheckbox>
        </AsFieldset>
        {isChecked && (
            <AsFormGroup {...GOOGLE.ENCRYPTION.KEY_NAME} isRequired={!isEdit}>
              <AsTextInput
                  {...FormUtils.fieldProps(encryptionField('encryptionKey'), current)}
                  onChange={FormUtils.handleUpdate(encryptionField('encryptionKey'), send)}
                  placeholder="e.g. projects/PROJECT_ID/locations/LOCATION/keyRings/KEY_RING_NAME/cryptoKeys/KEY_NAME"
              />
            </AsFormGroup>
        )}
      </div>
  );
}
