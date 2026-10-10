
import React from 'react';
import {useMachine} from '@xstate/react';

import {
  AsInfoAlert,
  AsList,
  AsTile,
  AsH2,
} from '@globalart/astrum-react-shared-components';

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
      <AsInfoAlert>{SETTINGS.READ_ONLY.WARNING}</AsInfoAlert>
      <AsTile>
        <AsTile.Header>
          <AsTile.HeaderTitle>
            <AsH2>{LABELS.SELECTED_TITLE}</AsH2>
          </AsTile.HeaderTitle>
        </AsTile.Header>
        <AsTile.Content>
          <AsList
            emptyMessage={LABELS.EMPTY_LIST}
            isLoading={isLoading}
            error={loadError ? loadError : null}
            retryHandler={retry}
          >
            {active?.length > 0 && active.map((realm) => (
              <AsList.Item key={realm}>
                <AsList.Text>{realm}</AsList.Text>
              </AsList.Item>
            ))}
          </AsList>
        </AsTile.Content>
      </AsTile>
    </>
  );
}
