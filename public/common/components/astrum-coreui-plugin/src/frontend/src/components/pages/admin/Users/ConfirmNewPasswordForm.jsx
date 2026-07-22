
import React from 'react';
import {useActor} from '@xstate/react';

import {
  NxFormGroup,
  NxH2,
  NxModal,
  NxStatefulForm,
  NxTextInput
} from '@globalart/react-shared-components';
import {FormUtils, ValidationUtils} from '@globalart/astrum-ui-plugin';

import UIStrings from '../../../../constants/UIStrings';

const {
  USERS: { MODAL }
} = UIStrings;

export default function ConfirmNewPasswordForm({ actor }) {
  const [state, send] = useActor(actor);
  const { isPristine, validationErrors } = state.context;
  const isInvalid = ValidationUtils.isInvalid(validationErrors);

  const cancel = () =>
    send({
      type: 'CANCEL',
    });

  return (
    <NxStatefulForm
      {...FormUtils.formProps(state, send)}
      submitMaskMessage={MODAL.CONFIRMING_ADMIN_PASSWORD}
      onCancel={cancel}
      validationErrors={FormUtils.saveTooltip({ isPristine, isInvalid })}
    >
      <NxModal.Header>
        <NxH2 id="modal-form-header">{MODAL.CHANGE_PASSWORD}</NxH2>
      </NxModal.Header>
      <NxModal.Content>
        <NxFormGroup label={MODAL.NEW_PASSWORD} isRequired>
          <NxTextInput
            {...FormUtils.fieldProps('passwordNew', state)}
            onChange={FormUtils.handleUpdate('passwordNew', send)}
            type="password"
            data-testid="newPassword"
          />
        </NxFormGroup>
        <NxFormGroup label={MODAL.CONFIRM_PASSWORD} isRequired>
          <NxTextInput
            {...FormUtils.fieldProps('passwordNewConfirm', state)}
            onChange={FormUtils.handleUpdate('passwordNewConfirm', send)}
            type="password"
            data-testid="confirmPassword"
          />
        </NxFormGroup>
      </NxModal.Content>
    </NxStatefulForm>
  );
}
