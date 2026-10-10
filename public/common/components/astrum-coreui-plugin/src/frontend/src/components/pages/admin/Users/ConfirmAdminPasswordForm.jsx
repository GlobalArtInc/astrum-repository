
import React from 'react';
import {useActor} from '@xstate/react';

import {
  AsFormGroup,
  AsH2,
  AsModal,
  AsP,
  AsStatefulForm,
  AsTextInput
} from '@globalart/react-shared-components';
import {FormUtils, ValidationUtils} from '@globalart/astrum-ui-plugin';

import UIStrings from '../../../../constants/UIStrings';

const {MODAL} = UIStrings.USERS;

export default function ConfirmAdminPasswordForm({
  actor,
  title,
  text,
  confirmLabel,
}) {
  const [state, send] = useActor(actor);
  const {isPristine, validationErrors} = state.context;
  const isInvalid = ValidationUtils.isInvalid(validationErrors);

  const cancel = () =>
    send({
      type: 'CANCEL',
    });

  return (
    <AsStatefulForm
      {...FormUtils.formProps(state, send)}
      submitBtnText={confirmLabel}
      submitMaskMessage={confirmLabel}
      onCancel={cancel}
      validationErrors={FormUtils.saveTooltip({isPristine, isInvalid})}
    >
      <AsModal.Header>
        <AsH2 id="modal-form-header">{title}</AsH2>
      </AsModal.Header>
      <AsModal.Content>
        <AsP>{text}</AsP>
        <AsFormGroup label={MODAL.ADMIN_PASSWORD} isRequired>
          <AsTextInput
            {...FormUtils.fieldProps('adminPassword', state)}
            onChange={FormUtils.handleUpdate('adminPassword', send)}
            type="password"
            data-testid="adminPassword"
          />
        </AsFormGroup>
      </AsModal.Content>
    </AsStatefulForm>
  );
}
