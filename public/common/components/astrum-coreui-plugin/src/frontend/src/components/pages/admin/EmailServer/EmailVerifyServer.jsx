
import React from 'react';
import {useActor} from '@xstate/react';
import {isEmpty} from 'ramda';
import {
  NxFormGroup,
  NxTextInput,
  NxFormRow,
  NxButton,
  NxSuccessAlert,
  NxErrorAlert,
  NxLoadWrapper,
} from '@globalart/react-shared-components';
import {FormUtils, ValidationUtils} from '@globalart/astrum-ui-plugin';

import UIStrings from '../../../../constants/UIStrings';
const {EMAIL_SERVER: LABELS} = UIStrings;

export default function EmailVerifyServer({actor}) {
  const [state, send] = useActor(actor);

  const {validationErrors, data, saveError, testResult, isTouched} =
    state.context;
  const isSaving = state.matches('saving');
  const isInvalid = FormUtils.isInvalid(validationErrors);
  const disabled = isInvalid || isSaving || ValidationUtils.isBlank(data.email);

  const onTest = () => send({type: 'SAVE'});

  const handleEnter = (event) => {
    if (event.key === 'Enter') {
      onTest();
    }
  };

  const success = isEmpty(isTouched) && testResult === true;
  const error = isEmpty(isTouched) && testResult === false;

  return (
    <>
      <NxFormRow>
        <>
          <NxFormGroup
            label={LABELS.VERIFY.LABEL}
            sublabel={LABELS.VERIFY.SUB_LABEL}
          >
            <NxTextInput
              {...FormUtils.fieldProps('email', state)}
              onChange={FormUtils.handleUpdate('email', send)}
              onKeyDown={handleEnter}
            />
          </NxFormGroup>
          <NxButton
            variant="primary"
            type="button"
            onClick={onTest}
            disabled={disabled}
          >
            {LABELS.VERIFY.TEST}
          </NxButton>
        </>
      </NxFormRow>
      <NxLoadWrapper
        loading={isSaving}
        loadError={saveError}
        retryHandler={onTest}
      >
        {success && <NxSuccessAlert>{LABELS.VERIFY.SUCCESS}</NxSuccessAlert>}
        {error && <NxErrorAlert>{LABELS.VERIFY.ERROR}</NxErrorAlert>}
      </NxLoadWrapper>
    </>
  );
}
