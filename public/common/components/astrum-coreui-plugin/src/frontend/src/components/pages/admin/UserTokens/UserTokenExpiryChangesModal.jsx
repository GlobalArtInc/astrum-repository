
import React from 'react';

import {NxButton, NxButtonBar, NxFooter, NxH2, NxModal, NxWarningAlert} from '@globalart/react-shared-components';

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
    <NxModal aria-label="user-token-expiry-confirmation" variant="narrow">
      <NxModal.Header>
        <NxH2>{CAPTION}</NxH2>
      </NxModal.Header>
      <NxModal.Content>
        <NxWarningAlert role="alert">{WARNING(expirationEnabled)}</NxWarningAlert>
      </NxModal.Content>
      <NxFooter>
        <NxButtonBar>
          <NxButton onClick={onClose}>{CANCEL_BUTTON}</NxButton>
          <NxButton onClick={onConfirm} variant="primary">{CONFIRM_BUTTON}</NxButton>
        </NxButtonBar>
      </NxFooter>
    </NxModal>
  )
}
