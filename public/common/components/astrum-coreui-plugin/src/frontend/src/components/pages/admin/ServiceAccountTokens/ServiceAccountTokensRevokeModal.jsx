
import React, {useEffect, useState} from 'react';
import {
  AsButton,
  AsButtonBar,
  AsFooter,
  AsFormGroup,
  AsH2,
  AsModal,
  AsSubmitMask,
  AsTextInput,
  AsWarningAlert,
} from '@globalart/astrum-react-shared-components';

import UIStrings from '../../../../constants/UIStrings';

const LABELS = UIStrings.SERVICE_ACCOUNT_TOKENS.REVOKE_MODAL;

export default function ServiceAccountTokensRevokeModal({tokenName, onConfirm, onClose, isRevoking = false}) {
  const [confirmation, setConfirmation] = useState('');
  const isPristine = confirmation.length === 0;
  const isMatch = confirmation === tokenName;
  const validationErrors = !isPristine && !isMatch ? LABELS.VALIDATION_ERROR : null;
  const submitDisabled = !isMatch || isRevoking;

  useEffect(() => {
    // Move focus to the warning when the modal opens so screen readers announce it.
    const alertEl = document.getElementById('as-sa-revoke-warning');
    if (alertEl) {
      alertEl.focus({preventScroll: true});
    }
  }, []);

  function handleSubmit() {
    if (submitDisabled) return;
    onConfirm();
  }

  return (
    <AsModal
      onCancel={isRevoking ? undefined : onClose}
      variant="narrow"
      aria-labelledby="revoke-sa-token-modal"
    >
      <AsModal.Header>
        <AsH2 id="revoke-sa-token-modal">{LABELS.TITLE}</AsH2>
      </AsModal.Header>
      <AsModal.Content>
        <AsWarningAlert
          id="as-sa-revoke-warning"
          tabIndex={-1}
          className="as-sa-sr-focus-target"
        >
          {LABELS.WARNING(tokenName)}
        </AsWarningAlert>
        <AsFormGroup label={LABELS.LABEL} sublabel={LABELS.SUBLABEL(tokenName)} isRequired>
          <AsTextInput
            id="revokeConfirmationString"
            name="revokeConfirmationString"
            value={confirmation}
            onChange={setConfirmation}
            validatable
            isPristine={isPristine}
            validationErrors={validationErrors}
          />
        </AsFormGroup>
      </AsModal.Content>
      <AsFooter>
        <AsButtonBar>
          <AsButton type="button" onClick={onClose} disabled={isRevoking}>
            {LABELS.CANCEL_BUTTON}
          </AsButton>
          <AsButton
            type="button"
            variant="error"
            disabled={submitDisabled}
            onClick={handleSubmit}
          >
            {LABELS.REVOKE_BUTTON}
          </AsButton>
        </AsButtonBar>
      </AsFooter>
      {isRevoking && <AsSubmitMask message={LABELS.REVOKING_MASK} />}
    </AsModal>
  );
}
