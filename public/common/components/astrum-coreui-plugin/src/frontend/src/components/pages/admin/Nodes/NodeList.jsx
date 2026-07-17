
import React from 'react';
import {useMachine} from '@xstate/react';
import {NxCard, NxH2, NxH3, NxLoadWrapper} from '@globalart/react-shared-components';
import {faArchive} from '@fortawesome/free-solid-svg-icons';

import {
  ContentBody,
  Page,
  PageActions,
  PageHeader,
  PageTitle
} from '@globalart/astrum-ui-plugin';
import {HelpTile} from '@globalart/astrum-ui-plugin';

import UIStrings from '../../../../constants/UIStrings';
import NodeListMachine from './NodeListMachine';
import './NodeList.scss';

const {
  NODES: {MENU, HELP}
} = UIStrings;

export default function NodeList() {
  const [state, sendEvent] = useMachine(NodeListMachine, {
    devTools: true
  });

  const retryHandler = () => sendEvent({type: 'LOAD'});

  const {data, error} = state.context;

  const isLoading = state.matches('loading');

  return (
    <Page className="as-nodes">
      <PageHeader>
        <PageTitle icon={faArchive} {...MENU} />
      </PageHeader>

      <ContentBody>
        <NxH2>{MENU.text}</NxH2>
        <NxLoadWrapper loading={isLoading} error={error} retryHandler={retryHandler}>
          <NxCard.Container>
            {data.map((node) => (
              <NxCard key={node.nodeId}>
                <NxCard.Header>
                  <NxH3>{node.hostname}</NxH3>
                </NxCard.Header>
              </NxCard>
            ))}
          </NxCard.Container>
        </NxLoadWrapper>

        <HelpTile header={HELP.TITLE} body={HELP.TEXT} />
      </ContentBody>
    </Page>
  );
}
