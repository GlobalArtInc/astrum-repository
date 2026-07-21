
import React from 'react';
import {useMachine} from '@xstate/react';
import PasswordChangeForm from './PasswordChangeForm';
import {faUser} from '@fortawesome/free-solid-svg-icons';
import {FormUtils} from '@globalart/astrum-ui-plugin';

import './UserAccount.scss';
import UserAccountMachine from './UserAccountMachine';
import UIStrings from '../../../../constants/UIStrings';
import {
  NxButton,
  NxFormGroup,
  NxStatefulForm,
  NxTextInput,
  NxTile,
  NxTooltip
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
          <NxTile className="user-account-settings">
            <NxStatefulForm
                {...FormUtils.formProps(state, send)}
                additionalFooterBtns={
                  <NxTooltip title={FormUtils.discardTooltip({isPristine})}>
                    <NxButton type="button" className={(isExternal || isPristine) && 'disabled'} onClick={discard}>
                      {UIStrings.SETTINGS.DISCARD_BUTTON_LABEL}
                    </NxButton>
                  </NxTooltip>
                }>
              <NxFormGroup label={UIStrings.USER_ACCOUNT.ID_FIELD_LABEL} isRequired>
                <NxTextInput
                    {...FormUtils.fieldProps('userId', state)}
                    disabled
                />
              </NxFormGroup>
              <NxFormGroup label={UIStrings.USER_ACCOUNT.FIRST_FIELD_LABEL} isRequired>
                <NxTextInput
                    {...FormUtils.fieldProps('firstName', state)}
                    onChange={FormUtils.handleUpdate('firstName', send)}
                    disabled={isExternal}
                />
              </NxFormGroup>
              <NxFormGroup label={UIStrings.USER_ACCOUNT.LAST_FIELD_LABEL} isRequired>
                <NxTextInput
                    {...FormUtils.fieldProps('lastName', state)}
                    onChange={FormUtils.handleUpdate('lastName', send)}
                    disabled={isExternal}
                />
              </NxFormGroup>
              <NxFormGroup label={UIStrings.USER_ACCOUNT.EMAIL_FIELD_LABEL} isRequired>
                <NxTextInput
                    {...FormUtils.fieldProps('email', state)}
                    onChange={FormUtils.handleUpdate('email', send)}
                    disabled={isExternal}
                />
              </NxFormGroup>
            </NxStatefulForm>
          </NxTile>

          {isInternal && <PasswordChangeForm userId={userId}/>}
        </ContentBody>
      </Page>
  );
}


