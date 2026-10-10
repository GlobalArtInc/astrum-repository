
import React from 'react';
import {useActor} from '@xstate/react';
import {indexBy, prop} from 'ramda';

import {
  AsButton,
  AsH2,
  AsReadOnly,
  AsLoadWrapper,
  AsInfoAlert,
  AsFooter,
  AsButtonBar,
  AsTile,
  AsList,
} from '@globalart/react-shared-components';

import UIStrings from '../../../../constants/UIStrings';
import {STATUSES, isExternalUser} from './UsersHelper';

const {USERS: {FORM: LABELS}} = UIStrings;

export default function UsersReadOnly({service, onDone}) {
  const [current, send] = useActor(service);

  const {
    data,
    allRoles,
    loadError
  } = current.context;

  const {userId, firstName, lastName, emailAddress, status, roles, externalRoles = [], source} = data;
  const isLoading = current.matches('loading');
  const isExternal = isExternalUser(source);
  const hasExternalRoles = Boolean(externalRoles.length);
  const rolesMap = indexBy(prop('id'), allRoles || []);

  const cancel = () => onDone();

  const retry = () => send({type: 'RETRY'});

  return <AsLoadWrapper loading={isLoading} error={loadError} retryHandler={retry}>
    <AsTile.Content>
      <AsInfoAlert>{UIStrings.SETTINGS.READ_ONLY.WARNING}</AsInfoAlert>
      <AsH2>{LABELS.SECTIONS.SETUP}</AsH2>
      <AsReadOnly>
        <AsReadOnly.Label>{LABELS.ID.LABEL}</AsReadOnly.Label>
        <AsReadOnly.Data>{userId}</AsReadOnly.Data>
        <AsReadOnly.Label>{LABELS.FIRST_NAME.LABEL}</AsReadOnly.Label>
        <AsReadOnly.Data>{firstName}</AsReadOnly.Data>
        <AsReadOnly.Label>{LABELS.LAST_NAME.LABEL}</AsReadOnly.Label>
        <AsReadOnly.Data>{lastName}</AsReadOnly.Data>
        <AsReadOnly.Label>{LABELS.EMAIL.LABEL}</AsReadOnly.Label>
        <AsReadOnly.Data>{emailAddress}</AsReadOnly.Data>
        <AsReadOnly.Label>{LABELS.STATUS.LABEL}</AsReadOnly.Label>
        <AsReadOnly.Data>{status ? STATUSES.active.label : STATUSES.disabled.label}</AsReadOnly.Data>
      </AsReadOnly>
      <AsH2>{LABELS.SECTIONS.ROLES}</AsH2>
      <AsList emptyMessage={LABELS.ROLES.EMPTY_LIST}>
        {(roles || [])?.map(roleId => (
            <AsList.Item key={roleId}>
              <AsList.Text>{rolesMap[roleId].name}</AsList.Text>
            </AsList.Item>
        ))}
      </AsList>
      {isExternal &&
          <AsReadOnly>
            <AsReadOnly.Label>{LABELS.EXTERNAL_ROLES.LABEL}</AsReadOnly.Label>
            <div className="read-only-external-roles">
              {hasExternalRoles && externalRoles.map(name => (
                  <AsReadOnly.Data key={name}>{name}</AsReadOnly.Data>
              ))}
            </div>
            {!hasExternalRoles &&
                <AsReadOnly.Data>{LABELS.EXTERNAL_ROLES.EMPTY_LIST}</AsReadOnly.Data>
            }
          </AsReadOnly>
      }
    </AsTile.Content>
    <AsFooter>
      <AsButtonBar>
        <AsButton type="button" onClick={cancel}>{UIStrings.SETTINGS.CANCEL_BUTTON_LABEL}</AsButton>
      </AsButtonBar>
    </AsFooter>
  </AsLoadWrapper>;
}
