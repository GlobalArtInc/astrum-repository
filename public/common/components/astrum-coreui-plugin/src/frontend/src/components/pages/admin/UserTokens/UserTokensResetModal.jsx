
import React from 'react';
import {useActor} from '@xstate/react';

import {
  NxH2,
  NxFormGroup,
  NxModal,
  NxStatefulForm,
  NxStatefulWarningAlert,
  NxTextInput
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
    <NxModal onCancel={cancelResetConfirmation} variant="narrow">
      <NxModal.Header>
        <NxH2>{RESET_CONFIRMATION.CAPTION}</NxH2>
      </NxModal.Header>
      <NxModal.Content>
        <NxStatefulWarningAlert>{RESET_CONFIRMATION.WARNING}</NxStatefulWarningAlert>
        <NxStatefulForm
          onSubmit={resetUserTokens}
          submitBtnText={RESET_CONFIRMATION.BUTTON}
          onCancel={cancelResetConfirmation}
          validationErrors={confirmationStringValidationError}
        >
          <NxFormGroup
            label={RESET_CONFIRMATION.LABEL}
            sublabel={RESET_CONFIRMATION.SUBLABEL}
            isRequired
          >
            <NxTextInput
              id="confirmationString"
              name="confirmationString"
              value={confirmationString}
              onChange={setConfirmationString}
              validationErrors={confirmationStringValidationError}
              validatable
              isPristine={confirmationString === ''}
            />
          </NxFormGroup>
        </NxStatefulForm>
      </NxModal.Content>
    </NxModal>
  );
}
