
import React from 'react';
import {useMachine} from '@xstate/react';
import {AsButton, AsLoadWrapper} from '@globalart/react-shared-components';
import {AsFontAwesomeIcon} from '@globalart/react-shared-components';
import { faLock } from '@fortawesome/free-solid-svg-icons';

import {
  ContentBody,
  Page,
  PageHeader,
  PageTitle,
  Section,
  SectionFooter
} from '@globalart/astrum-ui-plugin';

import NuGetApiTokenMachine from './NuGetApiTokenMachine';
import NuGetApiTokenModal from './NuGetApiTokenModal';
import UIStrings from '../../../../constants/UIStrings';

export default function NuGetApiToken() {
  const [state, send] = useMachine(NuGetApiTokenMachine, {devTools: true});

  const isLoading = !(state.matches('idle') || state.matches('showToken'));
  const showNugetModal = state.matches('showToken');

  function handleAccessKey() {
    send({type: 'ACCESS'});
  }

  function handleResetKey() {
    send({type: 'RESET'});
  }

  function handleCloseKey() {
    send({type: 'HIDE'});
  }

  function retry() {
    send({type: 'RETRY'});
  }

  return <Page>
    <PageHeader>
      <PageTitle
          icon={UIStrings.NUGET_API_KEY.MENU.icon}
          text={UIStrings.NUGET_API_KEY.MENU.text}
          description={UIStrings.NUGET_API_KEY.MENU.description}
      />
    </PageHeader>
    <ContentBody>
      <Section>
        <AsLoadWrapper isLoading={isLoading} retryHandler={retry}>
          <p> {UIStrings.NUGET_API_KEY.INSTRUCTIONS} </p>
          <SectionFooter>
            <AsButton variant='primary' onClick={handleAccessKey}>
              <span>{UIStrings.NUGET_API_KEY.ACCESS.BUTTON}</span>
            </AsButton>
            <AsButton onClick={handleResetKey}>
              <AsFontAwesomeIcon icon={faLock}/>
              <span>{UIStrings.NUGET_API_KEY.RESET.BUTTON}</span>
            </AsButton>
          </SectionFooter>

          {showNugetModal && <NuGetApiTokenModal apiKey={state.context.token.apiKey} onCloseClick={handleCloseKey}/>}
        </AsLoadWrapper>
      </Section>
    </ContentBody>
  </Page>;
}
