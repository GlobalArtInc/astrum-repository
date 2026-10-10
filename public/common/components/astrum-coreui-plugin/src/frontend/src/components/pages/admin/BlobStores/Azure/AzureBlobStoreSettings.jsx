
import React from 'react';
import {useMachine, useActor} from '@xstate/react';

import {ExtJS, FormUtils} from '@globalart/astrum-ui-plugin';
import {
  AsButton,
  AsCheckbox,
  AsErrorAlert,
  AsFieldset,
  AsFormGroup,
  AsInfoAlert,
  AsLoadingSpinner,
  AsSuccessAlert,
  AsTextInput,
  AsRadio
} from '@globalart/react-shared-components';

import AzureBlobStoreSettingsMachine from './AzureBlobStoreSettingsMachine';
import UIStrings from '../../../../../constants/UIStrings';

const AZURE = UIStrings.BLOB_STORES.AZURE;

export default function AzureBlobStoreSettings({service}) {
  const [current, send] = useActor(service);
  const {bucketConfiguration = {}} = current.context.data;
  const isProEdition = ExtJS.isProEdition();
  const authMode = bucketConfiguration.authentication?.authenticationMethod;
  const isTokenCredentialAuth = authMode === 'MANAGEDIDENTITY' || authMode === 'ENVIRONMENTVARIABLE';

  const [azureState, sendAzureEvent] = useMachine(AzureBlobStoreSettingsMachine, {
    context: {
      // blobStoreName will only contain a value when the form is being edited
      blobStoreName: current.context.data.name,
      accountName: bucketConfiguration.accountName,
      containerName: bucketConfiguration.containerName,
      authenticationMethod: bucketConfiguration.authentication?.authenticationMethod,
      accountKey: bucketConfiguration.authentication?.accountKey
    },
    devTools: true
  });

  function testConnection() {
    sendAzureEvent({
      type: 'TEST_CONNECTION'
    });
  }

  function handleAccountNameChange(value) {
    send({
      type: 'UPDATE',
      name: 'bucketConfiguration.accountName',
      value
    });

    sendAzureEvent({
      type: 'UPDATE_ACCOUNT_NAME',
      accountName: value
    });
  }

  function handleContainerNameChange(value) {
    send({
      type: 'UPDATE',
      name: 'bucketConfiguration.containerName',
      value
    });

    sendAzureEvent({
      type: 'UPDATE_CONTAINER_NAME',
      containerName: value
    });
  }

  function handleAccountKeyChange(value) {
    send({
      type: 'UPDATE',
      name: 'bucketConfiguration.authentication.accountKey',
      value
    });

    sendAzureEvent({
      type: 'UPDATE_ACCOUNT_KEY',
      accountKey: value
    });
  }

  function handleAuthenticationMethodChange(value) {
    send({
      type: 'UPDATE',
      name: 'bucketConfiguration.authentication.authenticationMethod',
      value
    });

    sendAzureEvent({
      type: 'UPDATE_AUTH_METHOD',
      authenticationMethod: value
    });
  }

  return (
      <div className="as-azure-blobstore">
        <AsFormGroup {...AZURE.ACCOUNT_NAME} isRequired>
          <AsTextInput
              {...FormUtils.fieldProps(['bucketConfiguration', 'accountName'], current)}
              onChange={handleAccountNameChange}/>
        </AsFormGroup>
        <AsFormGroup {...AZURE.CONTAINER_NAME} isRequired>
          <AsTextInput
              {...FormUtils.fieldProps(['bucketConfiguration', 'containerName'], current)}
              onChange={handleContainerNameChange}/>
        </AsFormGroup>
        <AsFieldset label={AZURE.AUTHENTICATION.label}>
          <AsRadio
              radioId="useEnvironmentVariables"
              name="bucketConfiguration.authentication.authenticationMethod"
              value="ENVIRONMENTVARIABLE"
              isChecked={bucketConfiguration.authentication?.authenticationMethod === 'ENVIRONMENTVARIABLE'}
              onChange={handleAuthenticationMethodChange}>
            {AZURE.AUTHENTICATION.ENVIRONMENTVARIABLE}
          </AsRadio>
          <AsRadio
              radioId="useManagedIdentity"
              name="bucketConfiguration.authentication.authenticationMethod"
              value="MANAGEDIDENTITY"
              isChecked={bucketConfiguration.authentication?.authenticationMethod === 'MANAGEDIDENTITY'}
              onChange={handleAuthenticationMethodChange}>
            {AZURE.AUTHENTICATION.MANAGED}
          </AsRadio>
          <AsRadio
              radioId="useAccountKey"
              name="bucketConfiguration.authentication.authenticationMethod"
              value="ACCOUNTKEY"
              isChecked={bucketConfiguration.authentication?.authenticationMethod === 'ACCOUNTKEY'}
              onChange={handleAuthenticationMethodChange}>
            {AZURE.AUTHENTICATION.ACCOUNT_KEY.label}
          </AsRadio>
        </AsFieldset>

        {bucketConfiguration.authentication?.authenticationMethod === 'ACCOUNTKEY' &&
        <AsFormGroup {...AZURE.AUTHENTICATION.ACCOUNT_KEY} isRequired>
          <AsTextInput type="password"
                       autoComplete="new-password"
                       {...FormUtils.fieldProps(['bucketConfiguration', 'authentication', 'accountKey'], current)}
                       onChange={handleAccountKeyChange}/>
        </AsFormGroup>
        }

        {isProEdition && (
          <div className="as-azure-presigned">
            <AsFieldset label={AZURE.PRE_SIGNED_URL.LABEL}
                        sublabel={AZURE.PRE_SIGNED_URL.SUBLABEL}>
              <AsCheckbox
                  {...FormUtils.checkboxProps('bucketConfiguration.preSignedUrlEnabled', current)}
                  onChange={FormUtils.handleUpdate('bucketConfiguration.preSignedUrlEnabled', send)}>
                {AZURE.PRE_SIGNED_URL.CHECKBOX_LABEL}
              </AsCheckbox>
              {bucketConfiguration.preSignedUrlEnabled && isTokenCredentialAuth && (
                <AsInfoAlert>{AZURE.PRE_SIGNED_URL.RBAC_NOTE}</AsInfoAlert>
              )}
            </AsFieldset>
          </div>
        )}

        <div className="as-form-row">
          <AsButton type="button" variant="tertiary" disabled={azureState.matches('testing')} onClick={testConnection}>
            {AZURE.TEST_CONNECTION}
          </AsButton>
        </div>

        <div className="as-form-row">
          {azureState.matches('editing.error') && <AsErrorAlert>{AZURE.TEST_CONNECTION_ERROR}</AsErrorAlert>}
          {azureState.matches('editing.success') && <AsSuccessAlert>{AZURE.TEST_CONNECTION_SUCCESS}</AsSuccessAlert>}
          {azureState.matches('testing') && <AsLoadingSpinner>{AZURE.TESTING}</AsLoadingSpinner>}
        </div>
      </div>
  );
}
