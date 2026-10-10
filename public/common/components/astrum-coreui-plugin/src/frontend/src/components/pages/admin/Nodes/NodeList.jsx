
import React from 'react';
import {useMachine} from '@xstate/react';
import {AsCard, AsH2, AsH3, AsLoadWrapper} from '@globalart/astrum-react-shared-components';
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
        <AsH2>{MENU.text}</AsH2>
        <AsLoadWrapper loading={isLoading} error={error} retryHandler={retryHandler}>
          <AsCard.Container>
            {data.map((node) => (
              <AsCard key={node.nodeId}>
                <AsCard.Header>
                  <AsH3>{node.hostname}</AsH3>
                </AsCard.Header>
              </AsCard>
            ))}
          </AsCard.Container>
        </AsLoadWrapper>

        <HelpTile header={HELP.TITLE} body={HELP.TEXT} />
      </ContentBody>
    </Page>
  );
}
