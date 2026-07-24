

import React from 'react';
import {useMachine} from '@xstate/react';

import {FormUtils} from '@globalart/astrum-ui-plugin';
import {
  NxWarningAlert,
  NxFormGroup,
  NxH2,
  NxModal,
  NxStatefulForm,
  NxTextInput,
} from '@globalart/react-shared-components';

import Machine from './BlobStoresConvertModalMachine';
import UIStrings from '../../../../constants/UIStrings';

const {BLOB_STORES: {FORM: {CONVERT_TO_GROUP_MODAL}}} = UIStrings;

export default function BlobStoresConvertModal({name, onDone, onCancel}) {
  const [state, send] = useMachine(Machine, {
    context: {
      pristineData: {name},
      data: {
        newName: `${name}-original`,
      },
    },
    actions: {
      onSaveSuccess: onDone,
    },
    devTools: true
  });

  return (
      <NxModal onCancel={onCancel}>
        <NxStatefulForm
            {...FormUtils.formProps(state, send)}
            onCancel={onCancel}
            submitBtnText={CONVERT_TO_GROUP_MODAL.CONVERT_BUTTON}
        >
          <NxModal.Header>
            <NxH2>{CONVERT_TO_GROUP_MODAL.HEADER}</NxH2>
          </NxModal.Header>
          <NxModal.Content>
            <NxWarningAlert>{CONVERT_TO_GROUP_MODAL.ALERT}</NxWarningAlert>
            <NxFormGroup
                className="blob-store-new-name"
                label={CONVERT_TO_GROUP_MODAL.LABEL}
                sublabel={CONVERT_TO_GROUP_MODAL.SUBLABEL}
                isRequired
            >
              <NxTextInput
                  {...FormUtils.fieldProps('newName', state)}
                  onChange={FormUtils.handleUpdate('newName', send)}
              />
            </NxFormGroup>
          </NxModal.Content>
        </NxStatefulForm>
      </NxModal>
  );
};
