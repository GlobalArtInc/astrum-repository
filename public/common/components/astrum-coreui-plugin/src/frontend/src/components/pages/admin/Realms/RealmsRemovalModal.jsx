
import React, {useEffect, useRef, useState} from 'react';

import {
  AsModal,
  AsButton,
  AsH2,
  AsFooter,
  AsButtonBar,
  AsWarningAlert,
  AsFormGroup,
  AsTextInput,
  asTextInputStateHelpers,
  hasValidationErrors,
  AsForm
} from '@globalart/astrum-react-shared-components';

import UIStrings from '../../../../constants/UIStrings';

const {initialState, userInput} = asTextInputStateHelpers;

const {
  REALMS: {
    LOCAL_REALM_REMOVAL_MODAL: {
      HEADER,
      WARNING,
      CONFIRM_BUTTON,
      ACKNOWLEDGEMENT: {
        STRING,
        VALIDATION_ERROR,
        LABEL,
        SUBLABEL,
        PLACEHOLDER
      }
    }
  },
  SETTINGS: {CANCEL_BUTTON_LABEL}
} = UIStrings;

export default function RealmsRemovalModal({onClose, onConfirm}) {
  const [state, setState] = useState(initialState(''));

  const ref = useRef();
  useEffect(() => {
    const input = ref.current.children[0].children[0];
    input.focus();
  }, []);

  const validator = (val) => (val === STRING ? null : VALIDATION_ERROR);

  const handleChange = (val) => setState(userInput(validator, val));

  const isSaveAllowed = !state.isPristine && !hasValidationErrors(state.validationErrors);

  const handleEnter = (e) => isSaveAllowed && e.key === 'Enter' && onConfirm();

  return (
    <AsModal
      className="as-local-realm-removal-modal"
      onCancel={onClose}
      aria-labelledby="modal-header"
      variant="narrow"
    >
      <AsModal.Header>
        <AsH2 id="modal-header">{HEADER}</AsH2>
      </AsModal.Header>
      <AsModal.Content>
        <AsWarningAlert>{WARNING}</AsWarningAlert>
        <AsForm.RequiredFieldNotice />
        <AsFormGroup label={LABEL} sublabel={SUBLABEL} isRequired={true}>
          <AsTextInput
            {...state}
            onChange={handleChange}
            onKeyDown={handleEnter}
            validatable={true}
            placeholder={PLACEHOLDER}
            ref={ref}
          />
        </AsFormGroup>
      </AsModal.Content>
      <AsFooter>
        <AsButtonBar>
          <AsButton onClick={onClose}>{CANCEL_BUTTON_LABEL}</AsButton>
          <AsButton onClick={onConfirm} variant="primary" disabled={!isSaveAllowed}>
            {CONFIRM_BUTTON}
          </AsButton>
        </AsButtonBar>
      </AsFooter>
    </AsModal>
  );
}
