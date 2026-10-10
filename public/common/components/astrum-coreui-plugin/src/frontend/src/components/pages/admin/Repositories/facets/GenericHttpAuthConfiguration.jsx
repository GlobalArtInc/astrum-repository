
import React from 'react';

import {FormUtils} from '@globalart/astrum-ui-plugin';

import {AsFormGroup, AsFormSelect, AsTextInput} from '@globalart/react-shared-components';

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
      <h2 className="as-h2">{EDITOR.HTTP_AUTH_CAPTION}</h2>
      <AsFormGroup label={EDITOR.AUTH_TYPE_LABEL} className="as-form-group-http-auth-type">
        <AsFormSelect
          {...FormUtils.selectProps('httpClient.authentication.type', currentParent)}
          onChange={updateType}
        >
          <option value="">{EDITOR.NONE_OPTION}</option>
          <option value="username">{EDITOR.USERNAME_OPTION}</option>
          <option value="ntlm">{EDITOR.NTLM_OPTION}</option>
        </AsFormSelect>
      </AsFormGroup>

      {type && type !== '' && (
        <div className="as-form-row">
          <AsFormGroup
            label={EDITOR.USERNAME_LABEL}
            isRequired
            className="as-form-group-http-username"
          >
            <AsTextInput
              {...FormUtils.fieldProps('httpClient.authentication.username', currentParent)}
              onChange={FormUtils.handleUpdate('httpClient.authentication.username', sendParent)}
            />
          </AsFormGroup>

          <AsFormGroup
            label={EDITOR.PASSWORD_LABEL}
            isRequired
            className="as-form-group-http-password"
          >
            <AsTextInput
              {...FormUtils.fieldProps('httpClient.authentication.password', currentParent)}
              onChange={FormUtils.handleUpdate('httpClient.authentication.password', sendParent)}
              type="password"
            />
          </AsFormGroup>
        </div>
      )}

      {/* AWS ECR session token — self-gated to Docker ECR proxies with username auth. */}
      <DockerEcrAuthConfiguration parentMachine={parentMachine} />

      {type === 'ntlm' && (
        <div className="as-form-row">
          <AsFormGroup
            label={EDITOR.NTLM_HOST_LABEL}
            isRequired
            className="as-form-group-ntlm-host"
          >
            <AsTextInput
              {...FormUtils.fieldProps('httpClient.authentication.ntlmHost', currentParent)}
              onChange={FormUtils.handleUpdate('httpClient.authentication.ntlmHost', sendParent)}
            />
          </AsFormGroup>

          <AsFormGroup
            label={EDITOR.NTLM_DOMAIN_LABEL}
            isRequired
            className="as-form-group-ntlm-domain"
          >
            <AsTextInput
              {...FormUtils.fieldProps('httpClient.authentication.ntlmDomain', currentParent)}
              onChange={FormUtils.handleUpdate('httpClient.authentication.ntlmDomain', sendParent)}
            />
          </AsFormGroup>
        </div>
      )}
    </>
  );
}
