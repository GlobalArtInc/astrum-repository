
import React, {useState} from 'react';
import {isNil, isEmpty} from 'ramda';
import {
  NxModal,
  NxTextInput,
  NxStatefulForm,
  NxFormGroup,
  NxH2,
  nxTextInputStateHelpers,
} from '@globalart/react-shared-components';
import UIStrings from '../../../../constants/UIStrings';

const {
  LDAP_SERVERS: {FORM},
} = UIStrings;

const {initialState, userInput} = nxTextInputStateHelpers;

function validator(value) {
  return !isEmpty(value) ? null : UIStrings.ERROR.FIELD_REQUIRED;
}

export default function LdapServersModalPassword({onSubmit, onCancel}) {
  const [password, setPassword] = useState(initialState(''));
  const [showValidationErrors, setShowValidationErrors] = useState(false);
  const [submitError, setSubmitError] = useState(null);

  function onChange(val) {
    setShowValidationErrors(false);
    setPassword(userInput(validator, val));
  }

  return (
    <NxModal variant="narrow" aria-labelledby="modal-form-header">
      <NxStatefulForm
        onSubmit={() => {
          setShowValidationErrors(true);

          if (!isNil(password.validationErrors)) {
            return setSubmitError(password.validationErrors);
          }

          if (isEmpty(password.trimmedValue)) {
            return setSubmitError(UIStrings.ERROR.FIELD_REQUIRED);
          }

          onSubmit(password);
        }}
        onCancel={onCancel}
        submitError={submitError}
        showValidationErrors={showValidationErrors}
      >
        <NxModal.Header>
          <NxH2 id="modal-form-header">{FORM.MODAL_PASSWORD.TITLE}</NxH2>
        </NxModal.Header>
        <NxModal.Content>
          <NxFormGroup
            label={FORM.MODAL_PASSWORD.LABEL}
            sublabel={FORM.MODAL_PASSWORD.SUB_LABEL}
            isRequired
          >
            <NxTextInput
              type="password"
              onChange={onChange}
              validatable
              {...password}
            />
          </NxFormGroup>
        </NxModal.Content>
      </NxStatefulForm>
    </NxModal>
  );
}
