
import React from 'react';
import {useMachine} from '@xstate/react';
import {faKey} from '@fortawesome/free-solid-svg-icons';

import {ExtJS, Permissions} from '@globalart/astrum-ui-plugin';

import {NxTile, NxLoadWrapper} from '@globalart/react-shared-components';

import {
  ContentBody,
  Page,
  PageHeader,
  PageTitle
} from '@globalart/astrum-ui-plugin';

import UserTokensReadOnly from './UserTokensReadOnly';
import UserTokensMachine from './UserTokensMachine';
import UIStrings from '../../../../constants/UIStrings';
import UserTokensForm from './UserTokensForm';

import './UserTokens.scss';

const {
  USER_TOKEN_CONFIGURATION: {MENU}
} = UIStrings;

export default function UserTokens() {
  const canUpdate = ExtJS.checkPermission(Permissions.USER_TOKENS_SETTINGS.UPDATE);
  const canDelete = ExtJS.checkPermission(Permissions.USER_TOKENS_USERS.DELETE);

  const [state, send, service] = useMachine(UserTokensMachine, {
    actions: {
      onSaveSuccess: () => {},
      onDeleteSuccess: () => {}
    },
    guards: {
      canDelete: () => canDelete
    },
    devTools: true
  });

  const isLoading = state.matches('loading') || state.matches('delete');

  const {loadError, data} = state.context;

  const retry = () => send({type: 'RETRY'});

  return (
    <Page>
      <PageHeader>
        <PageTitle icon={faKey} {...MENU} />
      </PageHeader>
      <ContentBody className="as-user-tokens">
        <NxTile>
          <NxTile.Content>
            <NxLoadWrapper loading={isLoading} error={loadError} retryHandler={retry}>
              {canUpdate ? (
                <UserTokensForm service={service} />
              ) : (
                <UserTokensReadOnly data={data} />
              )}
            </NxLoadWrapper>
          </NxTile.Content>
        </NxTile>
      </ContentBody>
    </Page>
  );
}
