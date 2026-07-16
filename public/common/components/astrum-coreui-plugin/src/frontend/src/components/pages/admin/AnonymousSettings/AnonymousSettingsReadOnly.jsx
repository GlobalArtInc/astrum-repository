
import React from 'react';
import {useMachine} from '@xstate/react';

import {
  NxLoadWrapper,
  NxInfoAlert,
  NxReadOnly,
} from '@globalart/react-shared-components';

import {FormUtils} from '@globalart/astrum-ui-plugin';

import UIStrings from '../../../../constants/UIStrings';

import AnonymousMachine from './AnonymousMachine';

export default function AnonymousSettingsReadOnly() {
  const [current, send] = useMachine(AnonymousMachine, {devTools: true});
  const {data, loadError, realms} = current.context;
  const isLoading = current.matches('loading');
  const realmName = realms?.find(realm => realm.id === data.realmName).name;
  const accessStatus = FormUtils.readOnlyCheckboxValueLabel(data.enabled);

  function retry() {
    send({type: 'RETRY'});
  }

  return <NxLoadWrapper loading={isLoading} error={loadError} retryHandler={retry}>
    <NxInfoAlert>{UIStrings.SETTINGS.READ_ONLY.WARNING}</NxInfoAlert>
    <NxReadOnly>
      <NxReadOnly.Label>{UIStrings.ANONYMOUS_SETTINGS.ENABLED_CHECKBOX_LABEL}</NxReadOnly.Label>
      <NxReadOnly.Data>{accessStatus}</NxReadOnly.Data>
      <NxReadOnly.Label>{UIStrings.ANONYMOUS_SETTINGS.USERNAME_TEXTFIELD_LABEL}</NxReadOnly.Label>
      <NxReadOnly.Data>{data.userId}</NxReadOnly.Data>
      <NxReadOnly.Label>{UIStrings.ANONYMOUS_SETTINGS.REALM_SELECT_LABEL}</NxReadOnly.Label>
      <NxReadOnly.Data>{realmName}</NxReadOnly.Data>
    </NxReadOnly>
  </NxLoadWrapper>;
}
