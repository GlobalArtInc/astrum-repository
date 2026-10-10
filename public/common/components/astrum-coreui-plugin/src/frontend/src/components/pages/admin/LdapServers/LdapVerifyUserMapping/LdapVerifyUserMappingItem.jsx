
import React from 'react';
import {useActor} from '@xstate/react';
import {AsModal, AsH2} from '@globalart/react-shared-components';
import {ReadOnlyField} from '@globalart/astrum-ui-plugin';
import UIStrings from '../../../../../constants/UIStrings';
import BackButton from './BackButton/BackButton';

const {
  LDAP_SERVERS: {
    FORM: {
      MODAL_VERIFY_USER_MAPPING: {
        ITEM: {
          BACK_BUTTON,
          LABELS: {USER_ID, NAME, EMAIL, ROLES}
        }
      }
    }
  }
} = UIStrings;

export default function LdapVerifyUserMappingItem({service}) {
  const [state, send] = useActor(service);

  const backToList = () => send({type: 'SHOW_LIST'});

  const {item} = state.context;

  const {username, realName, email, membership} = item;

  return (
    <>
      <AsModal.Header>
        <AsH2 id="modal-form-header">{realName}</AsH2>
      </AsModal.Header>
      <AsModal.Content>
        <BackButton onClick={backToList} text={BACK_BUTTON} />
        <ReadOnlyField label={USER_ID} value={username} />
        <ReadOnlyField label={NAME} value={realName} />
        <ReadOnlyField label={EMAIL} value={email} />
        <ReadOnlyField label={ROLES} value={membership.join(', ')} />
      </AsModal.Content>
    </>
  );
}
