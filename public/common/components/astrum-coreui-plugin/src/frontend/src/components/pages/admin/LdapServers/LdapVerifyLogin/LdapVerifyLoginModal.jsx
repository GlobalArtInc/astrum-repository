
import React from 'react';
import {useMachine} from '@xstate/react';
import {
  NxModal,
  NxTextInput,
  NxStatefulForm,
  NxFormGroup,
  NxH2,
  NxP,
  NxSuccessAlert
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
    <NxModal aria-labelledby="modal-form-header">
      <NxStatefulForm
        onSubmit={verifyLogin}
        onCancel={onCancel}
        submitBtnText={TEST_CONNECTION_BUTTON}
        submitMaskState={FormUtils.submitMaskState(state, ['verifying'])}
        submitError={error}
        submitErrorTitleMessage={ERROR_MESSAGE}
        validationErrors={FormUtils.getValidationErrorsMessage(state)}
      >
        <NxModal.Header>
          <NxH2 id="modal-form-header">{TITLE}</NxH2>
        </NxModal.Header>
        <NxModal.Content>
          <NxP>{DESCRIPTION}</NxP>
          <NxFormGroup label={USERNAME.LABEL} isRequired>
            <NxTextInput
              {...FormUtils.fieldProps('username', state)}
              onChange={FormUtils.handleUpdate('username', send)}
              placeholder={USERNAME.PLACEHOLDER}
            />
          </NxFormGroup>
          <NxFormGroup label={PASSWORD.LABEL} isRequired>
            <NxTextInput
              {...FormUtils.fieldProps('password', state)}
              onChange={FormUtils.handleUpdate('password', send)}
              type="password"
              placeholder={PASSWORD.PLACEHOLDER}
            />
          </NxFormGroup>
          {success && <NxSuccessAlert>{SUCCESS_MESSAGE + serverUrl}</NxSuccessAlert>}
        </NxModal.Content>
      </NxStatefulForm>
    </NxModal>
  );
}
