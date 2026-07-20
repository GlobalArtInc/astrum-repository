
import React, {useEffect, useRef, useState} from 'react';

import {
  NxModal,
  NxButton,
  NxH2,
  NxFooter,
  NxButtonBar,
  NxWarningAlert,
  NxFormGroup,
  NxTextInput,
  nxTextInputStateHelpers,
  hasValidationErrors,
  NxForm
} from '@globalart/react-shared-components';

import UIStrings from '../../../../constants/UIStrings';

const {initialState, userInput} = nxTextInputStateHelpers;

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
    <NxModal
      className="as-local-realm-removal-modal"
      onCancel={onClose}
      aria-labelledby="modal-header"
      variant="narrow"
    >
      <NxModal.Header>
        <NxH2 id="modal-header">{HEADER}</NxH2>
      </NxModal.Header>
      <NxModal.Content>
        <NxWarningAlert>{WARNING}</NxWarningAlert>
        <NxForm.RequiredFieldNotice />
        <NxFormGroup label={LABEL} sublabel={SUBLABEL} isRequired={true}>
          <NxTextInput
            {...state}
            onChange={handleChange}
            onKeyDown={handleEnter}
            validatable={true}
            placeholder={PLACEHOLDER}
            ref={ref}
          />
        </NxFormGroup>
      </NxModal.Content>
      <NxFooter>
        <NxButtonBar>
          <NxButton onClick={onClose}>{CANCEL_BUTTON_LABEL}</NxButton>
          <NxButton onClick={onConfirm} variant="primary" disabled={!isSaveAllowed}>
            {CONFIRM_BUTTON}
          </NxButton>
        </NxButtonBar>
      </NxFooter>
    </NxModal>
  );
}
