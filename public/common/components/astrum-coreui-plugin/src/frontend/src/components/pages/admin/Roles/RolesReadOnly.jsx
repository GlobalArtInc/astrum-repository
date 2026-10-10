
import React from 'react';
import {useActor} from '@xstate/react';
import {indexBy, prop} from 'ramda';

import {
  AsButton,
  AsH2,
  AsReadOnly,
  AsLoadWrapper,
  AsInfoAlert,
  AsList,
  AsFooter,
  AsButtonBar,
  AsTile,
} from '@globalart/react-shared-components';

import UIStrings from '../../../../constants/UIStrings';

const {ROLES: {FORM: LABELS}} = UIStrings;

export default function RolesReadOnly({service, onDone}) {
  const [current, send] = useActor(service);

  const {
    data: {id, name, description, readOnly:isDefaultRole = true, roles:selectedRoles, privileges},
    roles,
    loadError
  } = current.context;

  const rolesMap = indexBy(prop('id'), roles || []);

  const isLoading = current.matches('loading');

  const cancel = () => onDone();

  const retry = () => send({type: 'RETRY'});

  return <AsLoadWrapper loading={isLoading} error={loadError} retryHandler={retry}>
    <AsTile.Content>
      <AsInfoAlert>
        {isDefaultRole ? LABELS.DEFAULT_ROLE_WARNING : UIStrings.SETTINGS.READ_ONLY.WARNING}
      </AsInfoAlert>
      <AsH2>{LABELS.SECTIONS.SETUP}</AsH2>
      <AsReadOnly>
        <AsReadOnly.Label>{LABELS.ID.LABEL}</AsReadOnly.Label>
        <AsReadOnly.Data>{id}</AsReadOnly.Data>
        <AsReadOnly.Label>{LABELS.NAME.LABEL}</AsReadOnly.Label>
        <AsReadOnly.Data>{name}</AsReadOnly.Data>
        {description && <>
          <AsReadOnly.Label>{LABELS.DESCRIPTION.LABEL}</AsReadOnly.Label>
          <AsReadOnly.Data>{description}</AsReadOnly.Data>
        </>}
      </AsReadOnly>
      <AsH2>{LABELS.SECTIONS.PRIVILEGES}</AsH2>
      <AsList emptyMessage={LABELS.PRIVILEGES.EMPTY_LIST}>
        {(privileges || [])?.map(name => (
            <AsList.Item key={name}>
              <AsList.Text>{name}</AsList.Text>
            </AsList.Item>
        ))}
      </AsList>
      <AsH2>{LABELS.SECTIONS.ROLES}</AsH2>
      <AsList emptyMessage={LABELS.ROLES.EMPTY_LIST}>
        {(selectedRoles || [])?.map(roleId => (
            <AsList.Item key={roleId}>
              <AsList.Text>{rolesMap[roleId].name}</AsList.Text>
            </AsList.Item>
        ))}
      </AsList>
    </AsTile.Content>
    <AsFooter>
      <AsButtonBar>
        <AsButton type="button" onClick={cancel}>{UIStrings.SETTINGS.CANCEL_BUTTON_LABEL}</AsButton>
      </AsButtonBar>
    </AsFooter>
  </AsLoadWrapper>;
}
