

import React, { useState, useEffect } from 'react';
import { setDialogSetter, handleContinue, handleCancel } from '../../../router/unsavedChangesDialog';
import { AsButton, AsModal, AsWarningAlert } from '@globalart/react-shared-components';

export default function UnsavedChangesModal() {
  const [visible, setVisible] = useState(false);

  useEffect(() => {
    setDialogSetter(setVisible);
  }, []);

  if (!visible) return null;

  return (
    <AsModal id="unsaved-modal" variant="narrow" onCancel={handleCancel}>
      <header className="as-modal-header">
        <h2 className="as-h2">Unsaved Changes</h2>
      </header>
      <div className="as-modal-content">
        <AsWarningAlert className="as-alert--modifier">
          <span>You have unsaved changes. Continuing will discard them.</span>
        </AsWarningAlert>
      </div>
      <footer className="as-footer">
        <div className="as-btn-bar">
          <AsButton onClick={handleCancel} id="unsaved-changes-modal-cancel-button">
            Cancel
          </AsButton>
          <AsButton variant="primary" id="unsaved-changes-modal-continue-button" onClick={handleContinue}>
            Continue
          </AsButton>
        </div>
      </footer>
    </AsModal>
  );
}
