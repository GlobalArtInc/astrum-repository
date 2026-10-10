
import React from 'react';
import {useMachine} from '@xstate/react';
import {
  AsModal,
  AsTextInput,
  AsStatefulForm,
  AsFormGroup,
  AsH2,
  AsP,
  AsSuccessAlert
} from '@globalart/react-shared-components';
import {FormUtils} from '@globalart/astrum-ui-plugin';
import UIStrings from '../../../../../constants/UIStrings';
import LdapVerifyLoginMachine from './LdapVerifyLoginMachine.js';

const {
  LDAP_SERVERS: {
    FORM: {
      MODAL_VERIFY_LOGIN: {
        TITLE,
        DESCRIPTION,
        USERNAME,
        PASSWORD,
        TEST_CONNECTION_BUTTON,
        SUCCESS_MESSAGE,
        ERROR_MESSAGE
      }
    }
  }
} = UIStrings;

export default function LdapVerifyUserMappingModal({ldapConfig, onCancel}) {
  const [state, send] = useMachine(LdapVerifyLoginMachine, {
    context: {
      ldapConfig
    },
    devTools: true
  });

  const {success, error} = state.context;

  const {protocol, host, port} = ldapConfig;
  const serverUrl = `${protocol?.toLowerCase()}://${host}:${port}`;

  const verifyLogin = () => send({type: 'VERIFY'});

  return (
    <AsModal aria-labelledby="modal-form-header">
      <AsStatefulForm
        onSubmit={verifyLogin}
        onCancel={onCancel}
        submitBtnText={TEST_CONNECTION_BUTTON}
        submitMaskState={FormUtils.submitMaskState(state, ['verifying'])}
        submitError={error}
        submitErrorTitleMessage={ERROR_MESSAGE}
        validationErrors={FormUtils.getValidationErrorsMessage(state)}
      >
        <AsModal.Header>
          <AsH2 id="modal-form-header">{TITLE}</AsH2>
        </AsModal.Header>
        <AsModal.Content>
          <AsP>{DESCRIPTION}</AsP>
          <AsFormGroup label={USERNAME.LABEL} isRequired>
            <AsTextInput
              {...FormUtils.fieldProps('username', state)}
              onChange={FormUtils.handleUpdate('username', send)}
              placeholder={USERNAME.PLACEHOLDER}
            />
          </AsFormGroup>
          <AsFormGroup label={PASSWORD.LABEL} isRequired>
            <AsTextInput
              {...FormUtils.fieldProps('password', state)}
              onChange={FormUtils.handleUpdate('password', send)}
              type="password"
              placeholder={PASSWORD.PLACEHOLDER}
            />
          </AsFormGroup>
          {success && <AsSuccessAlert>{SUCCESS_MESSAGE + serverUrl}</AsSuccessAlert>}
        </AsModal.Content>
      </AsStatefulForm>
    </AsModal>
  );
}
