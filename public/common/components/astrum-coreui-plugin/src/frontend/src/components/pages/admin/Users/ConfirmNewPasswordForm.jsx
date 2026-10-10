
import React from 'react';
import {useActor} from '@xstate/react';

import {
  AsFormGroup,
  AsH2,
  AsModal,
  AsStatefulForm,
  AsTextInput
} from '@globalart/astrum-react-shared-components';
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
    <AsStatefulForm
      {...FormUtils.formProps(state, send)}
      submitMaskMessage={MODAL.CONFIRMING_ADMIN_PASSWORD}
      onCancel={cancel}
      validationErrors={FormUtils.saveTooltip({ isPristine, isInvalid })}
    >
      <AsModal.Header>
        <AsH2 id="modal-form-header">{MODAL.CHANGE_PASSWORD}</AsH2>
      </AsModal.Header>
      <AsModal.Content>
        <AsFormGroup label={MODAL.NEW_PASSWORD} isRequired>
          <AsTextInput
            {...FormUtils.fieldProps('passwordNew', state)}
            onChange={FormUtils.handleUpdate('passwordNew', send)}
            type="password"
            data-testid="newPassword"
          />
        </AsFormGroup>
        <AsFormGroup label={MODAL.CONFIRM_PASSWORD} isRequired>
          <AsTextInput
            {...FormUtils.fieldProps('passwordNewConfirm', state)}
            onChange={FormUtils.handleUpdate('passwordNewConfirm', send)}
            type="password"
            data-testid="confirmPassword"
          />
        </AsFormGroup>
      </AsModal.Content>
    </AsStatefulForm>
  );
}
