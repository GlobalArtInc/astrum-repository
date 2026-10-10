
import React from 'react';
import {useMachine} from '@xstate/react';
import {
  AsH2,
  AsLoadWrapper,
  AsTile,
  AsTree
} from '@globalart/astrum-react-shared-components';
import {faDatabase} from '@fortawesome/free-solid-svg-icons';
import {isEmpty} from 'ramda';

import {
  ContentBody,
  PageHeader,
  PageTitle
} from '@globalart/astrum-ui-plugin';

import UIStrings from '../../../../constants/UIStrings';
import BrowseTreeMachine from './BrowseTreeMachine';
import BrowseTreeChildren from './BrowseTreeChildren';
import './Browse.scss';

const {BROWSE} = UIStrings;

export default function BrowseTree({itemId}) {
  const [repoId, path] = decodeURIComponent(itemId).split(':');
  const [state, send] = useMachine(BrowseTreeMachine, {
    context: {
      repositoryName: repoId,
      initialOpenPath: path ?? ''
    },
    devTools: true
  });
  const {children, loadError} = state.context;
  const isLoading = state.matches('chooseInitialState') || state.matches('loading');

  return (
    <div>
      <PageHeader>
        <PageTitle icon={faDatabase} {...BROWSE.MENU}/>
      </PageHeader>
      <ContentBody className="as-browse-tree">
        <AsTile>
          <AsTile.Header>
            <AsTile.HeaderTitle>
              <AsH2>{BROWSE.TREE_TITLE} {repoId}</AsH2>
            </AsTile.HeaderTitle>
          </AsTile.Header>
          <AsTile.Content>
            <AsLoadWrapper retryHandler={() => send({type: 'RETRY'})} loading={isLoading} error={loadError}>
              {() => (
                isEmpty(children) ?
                <span>{BROWSE.TREE_EMPTY_MESSAGE}</span> :
                <AsTree>
                  <BrowseTreeChildren children={children}/>
                </AsTree>
              )}
            </AsLoadWrapper>
          </AsTile.Content>
        </AsTile>
      </ContentBody>
    </div>
  )
};
