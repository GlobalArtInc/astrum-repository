
import React from 'react';
import {useMachine} from '@xstate/react';

import {
  NxInfoAlert,
  NxList,
  NxTile,
  NxH2,
} from '@globalart/react-shared-components';

import UIStrings from '../../../../constants/UIStrings';
import ProprietaryRepositoriesMachine from "./ProprietaryRepositoriesMachine";

const {PROPRIETARY_REPOSITORIES: { CONFIGURATION: LABELS }, SETTINGS} = UIStrings;

export default function ProprietaryRepositoriesReadOnly() {
  const [current, send] = useMachine(ProprietaryRepositoriesMachine, {devTools: true});
  const {data: {enabledRepositories}, loadError} = current.context;
  const isLoading = current.matches('loading');

  function retry() {
    send({type: 'RETRY'});
  }

  return <>
    <NxInfoAlert>{SETTINGS.READ_ONLY.WARNING}</NxInfoAlert>
    <NxTile>
      <NxTile.Header>
        <NxTile.HeaderTitle>
          <NxH2>{LABELS.SELECTED_TITLE}</NxH2>
        </NxTile.HeaderTitle>
      </NxTile.Header>
      <NxTile.Content>
        <NxList
            emptyMessage={LABELS.EMPTY_LIST}
            isLoading={isLoading}
            error={loadError}
            retryHandler={retry}
        >
          {enabledRepositories?.map(name => (
              <NxList.Item key={name}>
                <NxList.Text>{name}</NxList.Text>
              </NxList.Item>
          ))}
        </NxList>
      </NxTile.Content>
    </NxTile>
  </>;
}
