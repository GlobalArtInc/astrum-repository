
import React from 'react';
import {useMachine} from '@xstate/react';

import {
  AsLoadWrapper,
  AsInfoAlert,
  AsReadOnly,
} from '@globalart/astrum-react-shared-components';

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

  return <AsLoadWrapper loading={isLoading} error={loadError} retryHandler={retry}>
    <AsInfoAlert>{UIStrings.SETTINGS.READ_ONLY.WARNING}</AsInfoAlert>
    <AsReadOnly>
      <AsReadOnly.Label>{UIStrings.ANONYMOUS_SETTINGS.ENABLED_CHECKBOX_LABEL}</AsReadOnly.Label>
      <AsReadOnly.Data>{accessStatus}</AsReadOnly.Data>
      <AsReadOnly.Label>{UIStrings.ANONYMOUS_SETTINGS.USERNAME_TEXTFIELD_LABEL}</AsReadOnly.Label>
      <AsReadOnly.Data>{data.userId}</AsReadOnly.Data>
      <AsReadOnly.Label>{UIStrings.ANONYMOUS_SETTINGS.REALM_SELECT_LABEL}</AsReadOnly.Label>
      <AsReadOnly.Data>{realmName}</AsReadOnly.Data>
    </AsReadOnly>
  </AsLoadWrapper>;
}
