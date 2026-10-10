
import React from 'react';
import {useActor} from '@xstate/react';

import {
  AsH2,
  AsFormGroup,
  AsModal,
  AsStatefulForm,
  AsStatefulWarningAlert,
  AsTextInput
} from '@globalart/react-shared-components';

import UIStrings from '../../../../constants/UIStrings';

const {
  USER_TOKEN_CONFIGURATION: {RESET_CONFIRMATION}
} = UIStrings;

export default function UserTokensResetModal({service}) {
  const [state, send] = useActor(service);

  const {confirmationString, confirmationStringValidationError} = state.context;

  const resetUserTokens = () => send({type: 'DELETE'});
  const cancelResetConfirmation = () => send({type: 'CANCEL_RESET_CONFIRMATION'});
  const setConfirmationString = (value) => send({type: 'SET_CONFIRMATION_STRING', value});

  return (
    <AsModal onCancel={cancelResetConfirmation} variant="narrow">
      <AsModal.Header>
        <AsH2>{RESET_CONFIRMATION.CAPTION}</AsH2>
      </AsModal.Header>
      <AsModal.Content>
        <AsStatefulWarningAlert>{RESET_CONFIRMATION.WARNING}</AsStatefulWarningAlert>
        <AsStatefulForm
          onSubmit={resetUserTokens}
          submitBtnText={RESET_CONFIRMATION.BUTTON}
          onCancel={cancelResetConfirmation}
          validationErrors={confirmationStringValidationError}
        >
          <AsFormGroup
            label={RESET_CONFIRMATION.LABEL}
            sublabel={RESET_CONFIRMATION.SUBLABEL}
            isRequired
          >
            <AsTextInput
              id="confirmationString"
              name="confirmationString"
              value={confirmationString}
              onChange={setConfirmationString}
              validationErrors={confirmationStringValidationError}
              validatable
              isPristine={confirmationString === ''}
            />
          </AsFormGroup>
        </AsStatefulForm>
      </AsModal.Content>
    </AsModal>
  );
}
