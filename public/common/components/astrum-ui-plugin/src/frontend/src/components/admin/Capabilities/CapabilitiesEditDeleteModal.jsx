

import React from 'react';
import { AsButton, AsButtonBar, AsFooter, AsH2, AsModal, AsWarningAlert } from '@globalart/astrum-react-shared-components';
import UIStrings from '../../../constants/UIStrings';

export default function CapabilitiesEditDeleteModal(
    { showModal, deleteWarningMessage, onConfirmDelete, onDeleteCanceled }
) {
  if (!showModal) {
    return false;
  }

  return <AsModal onCancel={onDeleteCanceled} aria-labelledby="modal-header-text">
    <AsModal.Header>
      <AsH2>{UIStrings.CAPABILITIES.EDIT.DELETE_TITLE}</AsH2>
    </AsModal.Header>

    <AsModal.Content>
      <AsWarningAlert>
        {buildMessage()}
      </AsWarningAlert>
    </AsModal.Content>

    <AsFooter>
      <AsButtonBar>
        <AsButton onClick={onDeleteCanceled}>{UIStrings.SETTINGS.CANCEL_BUTTON_LABEL}</AsButton>
        <AsButton variant="primary" onClick={onConfirmDelete}>{UIStrings.SETTINGS.DELETE_BUTTON_LABEL}</AsButton>
      </AsButtonBar>
    </AsFooter>
  </AsModal>

  function buildMessage() {
    return !deleteWarningMessage
      ? 'This will permanently remove this capability and its configuration. This action can’t be undone.'
      : deleteWarningMessage;
  }
}
