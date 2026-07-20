
import React from 'react';
import {useMachine} from '@xstate/react';

import {
  NxInfoAlert,
  NxList,
  NxTile,
  NxH2,
} from '@globalart/react-shared-components';

import UIStrings from '../../../../constants/UIStrings';
import RealmsMachine from './RealmsMachine';

const {
  REALMS: {CONFIGURATION: LABELS},
  SETTINGS,
} = UIStrings;

export default function RealmsReadOnly() {
  const [current, send] = useMachine(RealmsMachine, {devTools: true});
  const {
    data: {active},
    loadError,
  } = current.context;
  const isLoading = current.matches('loading');

  function retry() {
    send({type: 'RETRY'});
  }

  return (
    <>
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
            error={loadError ? loadError : null}
            retryHandler={retry}
          >
            {active?.length > 0 && active.map((realm) => (
              <NxList.Item key={realm}>
                <NxList.Text>{realm}</NxList.Text>
              </NxList.Item>
            ))}
          </NxList>
        </NxTile.Content>
      </NxTile>
    </>
  );
}
