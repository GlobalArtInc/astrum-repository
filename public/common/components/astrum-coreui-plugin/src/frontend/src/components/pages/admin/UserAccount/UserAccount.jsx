
import React from 'react';
import {useMachine} from '@xstate/react';
import PasswordChangeForm from './PasswordChangeForm';
import {faUser} from '@fortawesome/free-solid-svg-icons';
import {FormUtils} from '@globalart/astrum-ui-plugin';

import './UserAccount.scss';
import UserAccountMachine from './UserAccountMachine';
import UIStrings from '../../../../constants/UIStrings';
import {
  AsButton,
  AsFormGroup,
  AsStatefulForm,
  AsTextInput,
  AsTile,
  AsTooltip
} from '@globalart/react-shared-components';

import {ContentBody, Page, PageHeader, PageTitle} from '@globalart/astrum-ui-plugin';

export default function UserAccount() {
  const [state, send] = useMachine(UserAccountMachine, {devTools: true});
  const {isPristine} = state.context;
  const isLoading = state.matches('loading');
  const isExternal = !isLoading && state.context.data.external === true;
  const isInternal = !isLoading && state.context.data.external === false;
  const userId = state.context.data?.userId;

  function discard() {
    send({type: 'RESET'});
  }

  return (
      <Page>
        <PageHeader>
          <PageTitle
              icon={UIStrings.USER_ACCOUNT.MENU.icon}
              text={UIStrings.USER_ACCOUNT.MENU.text}
              description={UIStrings.USER_ACCOUNT.MENU.description}
          />
        </PageHeader>
        <ContentBody className="as-user-account">
          <AsTile className="user-account-settings">
            <AsStatefulForm
                {...FormUtils.formProps(state, send)}
                additionalFooterBtns={
                  <AsTooltip title={FormUtils.discardTooltip({isPristine})}>
                    <AsButton type="button" className={(isExternal || isPristine) && 'disabled'} onClick={discard}>
                      {UIStrings.SETTINGS.DISCARD_BUTTON_LABEL}
                    </AsButton>
                  </AsTooltip>
                }>
              <AsFormGroup label={UIStrings.USER_ACCOUNT.ID_FIELD_LABEL} isRequired>
                <AsTextInput
                    {...FormUtils.fieldProps('userId', state)}
                    disabled
                />
              </AsFormGroup>
              <AsFormGroup label={UIStrings.USER_ACCOUNT.FIRST_FIELD_LABEL} isRequired>
                <AsTextInput
                    {...FormUtils.fieldProps('firstName', state)}
                    onChange={FormUtils.handleUpdate('firstName', send)}
                    disabled={isExternal}
                />
              </AsFormGroup>
              <AsFormGroup label={UIStrings.USER_ACCOUNT.LAST_FIELD_LABEL} isRequired>
                <AsTextInput
                    {...FormUtils.fieldProps('lastName', state)}
                    onChange={FormUtils.handleUpdate('lastName', send)}
                    disabled={isExternal}
                />
              </AsFormGroup>
              <AsFormGroup label={UIStrings.USER_ACCOUNT.EMAIL_FIELD_LABEL} isRequired>
                <AsTextInput
                    {...FormUtils.fieldProps('email', state)}
                    onChange={FormUtils.handleUpdate('email', send)}
                    disabled={isExternal}
                />
              </AsFormGroup>
            </AsStatefulForm>
          </AsTile>

          {isInternal && <PasswordChangeForm userId={userId}/>}
        </ContentBody>
      </Page>
  );
}


