
import React from 'react';
import {useMachine} from '@xstate/react';

import {
  AsInfoAlert,
  AsList,
  AsTile,
  AsH2,
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
            error={loadError}
            retryHandler={retry}
        >
          {enabledRepositories?.map(name => (
              <AsList.Item key={name}>
                <AsList.Text>{name}</AsList.Text>
              </AsList.Item>
          ))}
        </AsList>
      </AsTile.Content>
    </AsTile>
  </>;
}
