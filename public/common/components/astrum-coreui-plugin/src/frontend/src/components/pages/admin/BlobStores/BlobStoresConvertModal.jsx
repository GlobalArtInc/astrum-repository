

import React from 'react';
import {useMachine} from '@xstate/react';

import {FormUtils} from '@globalart/astrum-ui-plugin';
import {
  AsWarningAlert,
  AsFormGroup,
  AsH2,
  AsModal,
  AsStatefulForm,
  AsTextInput,
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
      <AsModal onCancel={onCancel}>
        <AsStatefulForm
            {...FormUtils.formProps(state, send)}
            onCancel={onCancel}
            submitBtnText={CONVERT_TO_GROUP_MODAL.CONVERT_BUTTON}
        >
          <AsModal.Header>
            <AsH2>{CONVERT_TO_GROUP_MODAL.HEADER}</AsH2>
          </AsModal.Header>
          <AsModal.Content>
            <AsWarningAlert>{CONVERT_TO_GROUP_MODAL.ALERT}</AsWarningAlert>
            <AsFormGroup
                className="blob-store-new-name"
                label={CONVERT_TO_GROUP_MODAL.LABEL}
                sublabel={CONVERT_TO_GROUP_MODAL.SUBLABEL}
                isRequired
            >
              <AsTextInput
                  {...FormUtils.fieldProps('newName', state)}
                  onChange={FormUtils.handleUpdate('newName', send)}
              />
            </AsFormGroup>
          </AsModal.Content>
        </AsStatefulForm>
      </AsModal>
  );
};
