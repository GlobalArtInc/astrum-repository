
import React from 'react';
import {useActor} from '@xstate/react';

import {AsCombobox, AsFormGroup, AsTextInput} from '@globalart/react-shared-components';
import {FormUtils} from '@globalart/astrum-ui-plugin';

import UIStrings from '../../../../constants/UIStrings';
const { 
  ROLES: { 
    FORM: LABELS, 
    FORM: {EXTERNAL_TYPE: {LDAP: {MORE_CHARACTERS, NO_RESULTS}}} 
  } 
} = UIStrings;

export default function ExternalRolesCombobox({actor, parentMachine}) {
  const [state, send] = useActor(actor);

  const {data, query, error, externalRoleType, ldapQueryCharacterLimit} = state.context;
  const isLoading = state.matches('loading');
  const retry = () => send({type: 'RETRY'});
  const [parentState, sendParent] = parentMachine;

  const lowercaseQuery = query.toLowerCase();
  const externalRoles = data.filter(it => it.displayName.toLowerCase().indexOf(lowercaseQuery) !== -1);

  const onChangeMappedRole = (newQuery, dataItem) => {
    sendParent({type: 'UPDATE', name: 'id', value: dataItem?.id || newQuery});
  };

  const onSearchMappedRole = (newQuery) => {
    if (error && newQuery === query) {
      retry();
    } else {
      send({type: 'SET_QUERY', query: newQuery});
      if(externalRoleType.toLowerCase() !== 'ldap') {
        onChangeMappedRole(newQuery);
      }
    }
  };

  const emptyMessage = () => {
    if (externalRoleType.toLowerCase() === 'ldap') {
      if (query.length >= ldapQueryCharacterLimit) {
        return NO_RESULTS;
      } else {
        return MORE_CHARACTERS(ldapQueryCharacterLimit);
      }
    }
  }

  return <>
    {externalRoleType.toLowerCase() !== 'ldap' &&
      <AsFormGroup label={LABELS.MAPPED_ROLE.LABEL} isRequired>
        <AsTextInput
          {...FormUtils.fieldProps('id', parentState)}
          onChange={(value) => onSearchMappedRole(value)}
          value={query}
          />
      </AsFormGroup>
    }
    {externalRoleType.toLowerCase() === 'ldap' &&
      <AsFormGroup label={LABELS.MAPPED_ROLE.LABEL} isRequired>
        <AsCombobox
            {...FormUtils.fieldProps('id', parentState)}
            onChange={onChangeMappedRole}
            onSearch={onSearchMappedRole}
            loading={isLoading}
            autoComplete={false}
            matches={externalRoles}
            loadError={error}
            aria-label="combobox"
            emptyMessage={emptyMessage()}
        />
      </AsFormGroup>
    }
  </>
}
