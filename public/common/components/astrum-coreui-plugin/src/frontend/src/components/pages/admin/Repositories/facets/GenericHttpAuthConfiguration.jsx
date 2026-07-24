
import React from 'react';

import {FormUtils} from '@globalart/astrum-ui-plugin';

import {NxFormGroup, NxFormSelect, NxTextInput} from '@globalart/react-shared-components';

import UIStrings from '../../../../../constants/UIStrings';

import DockerEcrAuthConfiguration from './DockerEcrAuthConfiguration';

const {EDITOR} = UIStrings.REPOSITORIES;

export default function GenericHttpAuthConfiguration({parentMachine}) {
  const [currentParent, sendParent] = parentMachine;

  const {httpClient} = currentParent.context.data;

  const type = httpClient.authentication?.type;

  const updateType = (authType) => {
    if (authType === '') {
      sendParent({
        type: 'UPDATE',
        name: 'httpClient.authentication',
        value: null
      });
    } else {
      sendParent({
        type: 'UPDATE',
        name: 'httpClient.authentication.type',
        value: authType
      });
    }
  };

  return (
    <>
      <h2 className="nx-h2">{EDITOR.HTTP_AUTH_CAPTION}</h2>
      <NxFormGroup label={EDITOR.AUTH_TYPE_LABEL} className="as-form-group-http-auth-type">
        <NxFormSelect
          {...FormUtils.selectProps('httpClient.authentication.type', currentParent)}
          onChange={updateType}
        >
          <option value="">{EDITOR.NONE_OPTION}</option>
          <option value="username">{EDITOR.USERNAME_OPTION}</option>
          <option value="ntlm">{EDITOR.NTLM_OPTION}</option>
        </NxFormSelect>
      </NxFormGroup>

      {type && type !== '' && (
        <div className="nx-form-row">
          <NxFormGroup
            label={EDITOR.USERNAME_LABEL}
            isRequired
            className="as-form-group-http-username"
          >
            <NxTextInput
              {...FormUtils.fieldProps('httpClient.authentication.username', currentParent)}
              onChange={FormUtils.handleUpdate('httpClient.authentication.username', sendParent)}
            />
          </NxFormGroup>

          <NxFormGroup
            label={EDITOR.PASSWORD_LABEL}
            isRequired
            className="as-form-group-http-password"
          >
            <NxTextInput
              {...FormUtils.fieldProps('httpClient.authentication.password', currentParent)}
              onChange={FormUtils.handleUpdate('httpClient.authentication.password', sendParent)}
              type="password"
            />
          </NxFormGroup>
        </div>
      )}

      {/* AWS ECR session token — self-gated to Docker ECR proxies with username auth. */}
      <DockerEcrAuthConfiguration parentMachine={parentMachine} />

      {type === 'ntlm' && (
        <div className="nx-form-row">
          <NxFormGroup
            label={EDITOR.NTLM_HOST_LABEL}
            isRequired
            className="as-form-group-ntlm-host"
          >
            <NxTextInput
              {...FormUtils.fieldProps('httpClient.authentication.ntlmHost', currentParent)}
              onChange={FormUtils.handleUpdate('httpClient.authentication.ntlmHost', sendParent)}
            />
          </NxFormGroup>

          <NxFormGroup
            label={EDITOR.NTLM_DOMAIN_LABEL}
            isRequired
            className="as-form-group-ntlm-domain"
          >
            <NxTextInput
              {...FormUtils.fieldProps('httpClient.authentication.ntlmDomain', currentParent)}
              onChange={FormUtils.handleUpdate('httpClient.authentication.ntlmDomain', sendParent)}
            />
          </NxFormGroup>
        </div>
      )}
    </>
  );
}
