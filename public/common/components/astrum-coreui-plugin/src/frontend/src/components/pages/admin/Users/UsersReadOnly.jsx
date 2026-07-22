
import React from 'react';
import {useActor} from '@xstate/react';
import {indexBy, prop} from 'ramda';

import {
  NxButton,
  NxH2,
  NxReadOnly,
  NxLoadWrapper,
  NxInfoAlert,
  NxFooter,
  NxButtonBar,
  NxTile,
  NxList,
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

  return <NxLoadWrapper loading={isLoading} error={loadError} retryHandler={retry}>
    <NxTile.Content>
      <NxInfoAlert>{UIStrings.SETTINGS.READ_ONLY.WARNING}</NxInfoAlert>
      <NxH2>{LABELS.SECTIONS.SETUP}</NxH2>
      <NxReadOnly>
        <NxReadOnly.Label>{LABELS.ID.LABEL}</NxReadOnly.Label>
        <NxReadOnly.Data>{userId}</NxReadOnly.Data>
        <NxReadOnly.Label>{LABELS.FIRST_NAME.LABEL}</NxReadOnly.Label>
        <NxReadOnly.Data>{firstName}</NxReadOnly.Data>
        <NxReadOnly.Label>{LABELS.LAST_NAME.LABEL}</NxReadOnly.Label>
        <NxReadOnly.Data>{lastName}</NxReadOnly.Data>
        <NxReadOnly.Label>{LABELS.EMAIL.LABEL}</NxReadOnly.Label>
        <NxReadOnly.Data>{emailAddress}</NxReadOnly.Data>
        <NxReadOnly.Label>{LABELS.STATUS.LABEL}</NxReadOnly.Label>
        <NxReadOnly.Data>{status ? STATUSES.active.label : STATUSES.disabled.label}</NxReadOnly.Data>
      </NxReadOnly>
      <NxH2>{LABELS.SECTIONS.ROLES}</NxH2>
      <NxList emptyMessage={LABELS.ROLES.EMPTY_LIST}>
        {(roles || [])?.map(roleId => (
            <NxList.Item key={roleId}>
              <NxList.Text>{rolesMap[roleId].name}</NxList.Text>
            </NxList.Item>
        ))}
      </NxList>
      {isExternal &&
          <NxReadOnly>
            <NxReadOnly.Label>{LABELS.EXTERNAL_ROLES.LABEL}</NxReadOnly.Label>
            <div className="read-only-external-roles">
              {hasExternalRoles && externalRoles.map(name => (
                  <NxReadOnly.Data key={name}>{name}</NxReadOnly.Data>
              ))}
            </div>
            {!hasExternalRoles &&
                <NxReadOnly.Data>{LABELS.EXTERNAL_ROLES.EMPTY_LIST}</NxReadOnly.Data>
            }
          </NxReadOnly>
      }
    </NxTile.Content>
    <NxFooter>
      <NxButtonBar>
        <NxButton type="button" onClick={cancel}>{UIStrings.SETTINGS.CANCEL_BUTTON_LABEL}</NxButton>
      </NxButtonBar>
    </NxFooter>
  </NxLoadWrapper>;
}
