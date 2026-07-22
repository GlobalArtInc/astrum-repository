
import React from 'react';
import {useActor} from '@xstate/react';

import {
  NxFormGroup,
  NxH2,
  NxModal,
  NxP,
  NxStatefulForm,
  NxTextInput
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
    <NxStatefulForm
      {...FormUtils.formProps(state, send)}
      submitBtnText={confirmLabel}
      submitMaskMessage={confirmLabel}
      onCancel={cancel}
      validationErrors={FormUtils.saveTooltip({isPristine, isInvalid})}
    >
      <NxModal.Header>
        <NxH2 id="modal-form-header">{title}</NxH2>
      </NxModal.Header>
      <NxModal.Content>
        <NxP>{text}</NxP>
        <NxFormGroup label={MODAL.ADMIN_PASSWORD} isRequired>
          <NxTextInput
            {...FormUtils.fieldProps('adminPassword', state)}
            onChange={FormUtils.handleUpdate('adminPassword', send)}
            type="password"
            data-testid="adminPassword"
          />
        </NxFormGroup>
      </NxModal.Content>
    </NxStatefulForm>
  );
}
