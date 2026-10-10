
import React, {useState} from 'react';
import {isNil, isEmpty} from 'ramda';
import {
  AsModal,
  AsTextInput,
  AsStatefulForm,
  AsFormGroup,
  AsH2,
  asTextInputStateHelpers,
} from '@globalart/astrum-react-shared-components';
import UIStrings from '../../../../constants/UIStrings';

const {
  LDAP_SERVERS: {FORM},
} = UIStrings;

const {initialState, userInput} = asTextInputStateHelpers;

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
    <AsModal variant="narrow" aria-labelledby="modal-form-header">
      <AsStatefulForm
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
        <AsModal.Header>
          <AsH2 id="modal-form-header">{FORM.MODAL_PASSWORD.TITLE}</AsH2>
        </AsModal.Header>
        <AsModal.Content>
          <AsFormGroup
            label={FORM.MODAL_PASSWORD.LABEL}
            sublabel={FORM.MODAL_PASSWORD.SUB_LABEL}
            isRequired
          >
            <AsTextInput
              type="password"
              onChange={onChange}
              validatable
              {...password}
            />
          </AsFormGroup>
        </AsModal.Content>
      </AsStatefulForm>
    </AsModal>
  );
}
