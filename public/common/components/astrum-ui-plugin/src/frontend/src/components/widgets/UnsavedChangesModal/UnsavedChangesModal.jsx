

import React, { useState, useEffect } from 'react';
import { setDialogSetter, handleContinue, handleCancel } from '../../../router/unsavedChangesDialog';
import { NxButton, NxModal, NxWarningAlert } from '@globalart/react-shared-components';

export default function UnsavedChangesModal() {
  const [visible, setVisible] = useState(false);

  useEffect(() => {
    setDialogSetter(setVisible);
  }, []);

  if (!visible) return null;

  return (
    <NxModal id="unsaved-modal" variant="narrow" onCancel={handleCancel}>
      <header className="nx-modal-header">
        <h2 className="nx-h2">Unsaved Changes</h2>
      </header>
      <div className="nx-modal-content">
        <NxWarningAlert className="nx-alert--modifier">
          <span>You have unsaved changes. Continuing will discard them.</span>
        </NxWarningAlert>
      </div>
      <footer className="nx-footer">
        <div className="nx-btn-bar">
          <NxButton onClick={handleCancel} id="unsaved-changes-modal-cancel-button">
            Cancel
          </NxButton>
          <NxButton variant="primary" id="unsaved-changes-modal-continue-button" onClick={handleContinue}>
            Continue
          </NxButton>
        </div>
      </footer>
    </NxModal>
  );
}
