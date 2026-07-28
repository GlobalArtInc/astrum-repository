

import React from 'react';
import { NxButton, NxButtonBar, NxFooter, NxH2, NxModal, NxWarningAlert } from '@globalart/react-shared-components';

export default function CapabilitiesEditDeleteModal(
    { showModal, deleteWarningMessage, onConfirmDelete, onDeleteCanceled }
) {
  if (!showModal) {
    return false;
  }

  return <NxModal onCancel={onDeleteCanceled} aria-labelledby="modal-header-text">
    <NxModal.Header>
      <NxH2>Delete Capability?</NxH2>
    </NxModal.Header>

    <NxModal.Content>
      <NxWarningAlert>
        {buildMessage()}
      </NxWarningAlert>
    </NxModal.Content>

    <NxFooter>
      <NxButtonBar>
        <NxButton onClick={onDeleteCanceled}>Cancel</NxButton>
        <NxButton variant="primary" onClick={onConfirmDelete}>Delete</NxButton>
      </NxButtonBar>
    </NxFooter>
  </NxModal>

  function buildMessage() {
    return !deleteWarningMessage
      ? 'This will permanently remove this capability and its configuration. This action can’t be undone.'
      : deleteWarningMessage;
  }
}
