
import React from 'react';
import {useMachine} from '@xstate/react';

import {ExtJS, FormUtils} from '@globalart/astrum-ui-plugin';
import {
  AsButton,
  AsFormGroup,
  AsStatefulForm,
  AsTooltip,
  AsTextInput
} from '@globalart/react-shared-components';

import {Section} from '@globalart/astrum-ui-plugin';

import PasswordChangeMachine from './PasswordChangeMachine';
import UIStrings from '../../../../constants/UIStrings';

export default function PasswordChangeForm({userId}) {
  const [current, send] = useMachine(PasswordChangeMachine, {devTools: true});
  const {isPristine} = current.context;

  function save() {
    send({type: 'SAVE', userId: userId});
  }

  function discard() {
    send({type: 'RESET'});
  }
  const canUpdatePassword = ExtJS.checkPermission('astrum:userschangepw:create');

  if (!canUpdatePassword) {
    return null;
  }

  return (
    <Section className='user-account-settings'>
      <AsStatefulForm
        {...FormUtils.formProps(current, send)}
        onSubmit={save}
        submitBtnText={UIStrings.USER_ACCOUNT.ACTIONS.changePassword}
        additionalFooterBtns={
          <AsTooltip title={FormUtils.discardTooltip({ isPristine })}>
            <AsButton type='button' className={isPristine && 'disabled'} onClick={discard}>
              {UIStrings.USER_ACCOUNT.ACTIONS.discardChangePassword}
            </AsButton>
          </AsTooltip>
        }
      >
        <AsFormGroup label={UIStrings.USER_ACCOUNT.PASSWORD_CURRENT_FIELD_LABEL} isRequired>
          <AsTextInput
            {...FormUtils.fieldProps('passwordCurrent', current)}
            onChange={FormUtils.handleUpdate('passwordCurrent', send)}
            type='password'
          />
        </AsFormGroup>
        <AsFormGroup label={UIStrings.USER_ACCOUNT.PASSWORD_NEW_FIELD_LABEL} isRequired>
          <AsTextInput
            {...FormUtils.fieldProps('passwordNew', current)}
            onChange={FormUtils.handleUpdate('passwordNew', send)}
            type='password'
          />
        </AsFormGroup>
        <AsFormGroup label={UIStrings.USER_ACCOUNT.PASSWORD_NEW_CONFIRM_FIELD_LABEL} isRequired>
          <AsTextInput
            {...FormUtils.fieldProps('passwordNewConfirm', current)}
            onChange={FormUtils.handleUpdate('passwordNewConfirm', send)}
            type='password'
          />
        </AsFormGroup>
      </AsStatefulForm>
    </Section>
  );
}
