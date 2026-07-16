
import React from 'react';
import {useMachine} from '@xstate/react';
import {NxTile} from '@globalart/react-shared-components';
import {ExtJS, Permissions} from '@globalart/astrum-ui-plugin';
import {faEnvelope} from '@fortawesome/free-solid-svg-icons';

import {
  ContentBody,
  Page,
  PageHeader,
  PageTitle
} from '@globalart/astrum-ui-plugin';

import EmailServerForm from './EmailServerForm';
import EmailServerReadOnly from './EmailServerReadOnly';
import EmailVerifyServer from './EmailVerifyServer';

import UIStrings from '../../../../constants/UIStrings';

import Machine from './EmailServerMachine';

export default function EmailServer() {
  const canEdit = ExtJS.checkPermission(Permissions.SETTINGS.UPDATE);
  const stateMachine = useMachine(Machine, {devTools: true});
  const [state] = stateMachine;
  const actor = state.context.emailVerifyServer;
  
  if (!ExtJS.useUser()) {
    return null;
  }

  return (
    <Page>
      <PageHeader>
        <PageTitle
            icon={UIStrings.EMAIL_SERVER.MENU.icon}
            text={UIStrings.EMAIL_SERVER.MENU.text}
            description={UIStrings.EMAIL_SERVER.MENU.description}
        />
      </PageHeader>
      <ContentBody className="as-email-server">
        <NxTile>
          {canEdit ? (
            <EmailServerForm parentMachine={stateMachine} />
          ) : (
            <EmailServerReadOnly />
          )}
        </NxTile>
        {canEdit && actor && (
          <NxTile>
            <EmailVerifyServer actor={actor} />
          </NxTile>
        )}
      </ContentBody>
    </Page>
  );
}
