
import React from 'react';
import {useMachine} from '@xstate/react';
import {NxModal, NxFooter, NxButton, NxButtonBar} from '@globalart/react-shared-components';
import UIStrings from '../../../../../constants/UIStrings';
import LdapVerifyUserMappingMachine from './LdapVerifyUserMappingMachine';
import LdapVerifyUserMappingItem from './LdapVerifyUserMappingItem';
import LdapVerifyUserMappingList from './LdapVerifyUserMappingList';

const {CLOSE} = UIStrings;

export default function LdapVerifyUserMappingModal({ldapConfig, onCancel}) {
  const [state, _, service] = useMachine(LdapVerifyUserMappingMachine, {
    context: {
      ldapConfig
    },
    devTools: true
  });

  const showingItem = state.matches('showingItem');

  return (
    <NxModal
      variant="wide"
      aria-labelledby="modal-form-header"
      className="as-ldap-user-mapping-modal"
    >
      {showingItem ? (
        <LdapVerifyUserMappingItem service={service} />
      ) : (
        <LdapVerifyUserMappingList service={service} />
      )}
      <NxFooter>
        <NxButtonBar>
          <NxButton onClick={onCancel}>{CLOSE}</NxButton>
        </NxButtonBar>
      </NxFooter>
    </NxModal>
  );
}
