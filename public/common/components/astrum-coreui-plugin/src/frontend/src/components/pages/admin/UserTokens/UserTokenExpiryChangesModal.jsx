
import React from 'react';

import {AsButton, AsButtonBar, AsFooter, AsH2, AsModal, AsWarningAlert} from '@globalart/react-shared-components';

import UIStrings from '../../../../constants/UIStrings';

const {USER_TOKEN_CONFIGURATION: {
  USER_TOKEN_EXPIRY_CONFIRMATION:{
    CAPTION,
    WARNING,
    CONFIRM_BUTTON,
    CANCEL_BUTTON
  }}} = UIStrings;

export default function UserTokenExpiryChangesModal({onClose, onConfirm, expirationEnabled}) {

  return (
    <AsModal aria-label="user-token-expiry-confirmation" variant="narrow">
      <AsModal.Header>
        <AsH2>{CAPTION}</AsH2>
      </AsModal.Header>
      <AsModal.Content>
        <AsWarningAlert role="alert">{WARNING(expirationEnabled)}</AsWarningAlert>
      </AsModal.Content>
      <AsFooter>
        <AsButtonBar>
          <AsButton onClick={onClose}>{CANCEL_BUTTON}</AsButton>
          <AsButton onClick={onConfirm} variant="primary">{CONFIRM_BUTTON}</AsButton>
        </AsButtonBar>
      </AsFooter>
    </AsModal>
  )
}
