

import React from 'react';
import { AsButton, AsButtonBar, AsFooter, AsH2, AsModal, AsWarningAlert } from '@globalart/react-shared-components';

export default function CapabilitiesEditDeleteModal(
    { showModal, deleteWarningMessage, onConfirmDelete, onDeleteCanceled }
) {
  if (!showModal) {
    return false;
  }

  return <AsModal onCancel={onDeleteCanceled} aria-labelledby="modal-header-text">
    <AsModal.Header>
      <AsH2>Delete Capability?</AsH2>
    </AsModal.Header>

    <AsModal.Content>
      <AsWarningAlert>
        {buildMessage()}
      </AsWarningAlert>
    </AsModal.Content>

    <AsFooter>
      <AsButtonBar>
        <AsButton onClick={onDeleteCanceled}>Cancel</AsButton>
        <AsButton variant="primary" onClick={onConfirmDelete}>Delete</AsButton>
      </AsButtonBar>
    </AsFooter>
  </AsModal>

  function buildMessage() {
    return !deleteWarningMessage
      ? 'This will permanently remove this capability and its configuration. This action can’t be undone.'
      : deleteWarningMessage;
  }
}
