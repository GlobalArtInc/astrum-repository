
import React from 'react';
import {useActor} from '@xstate/react';
import {AsModal, AsH2, AsTable} from '@globalart/react-shared-components';
import UIStrings from '../../../../../constants/UIStrings';

const {
  LDAP_SERVERS: {
    FORM: {
      MODAL_VERIFY_USER_MAPPING: {
        LIST: {
          HEADER,
          TABLE: {USER_ID, NAME, EMAIL, ROLES}
        }
      }
    }
  },
  EMPTY_MESSAGE
} = UIStrings;

export default function LdapVerifyUserMappingList({service}) {
  const [state, send] = useActor(service);

  const {data = [], error} = state.context;

  const isLoading = state.matches('loading');

  const showItem = (itemIndex) => send({type: 'SHOW_ITEM', itemIndex});
  const retryHandler = () => send({type: 'RETRY'});

  return (
    <>
      <AsModal.Header>
        <AsH2 id="modal-form-header">{HEADER}</AsH2>
      </AsModal.Header>
      <AsModal.Content>
        <AsTable>
          <AsTable.Head>
            <AsTable.Row>
              <AsTable.Cell>{USER_ID}</AsTable.Cell>
              <AsTable.Cell>{NAME}</AsTable.Cell>
              <AsTable.Cell>{EMAIL}</AsTable.Cell>
              <AsTable.Cell>{ROLES}</AsTable.Cell>
              <AsTable.Cell chevron />
            </AsTable.Row>
          </AsTable.Head>
          <AsTable.Body
            isLoading={isLoading}
            error={error}
            retryHandler={retryHandler}
            emptyMessage={EMPTY_MESSAGE}
          >
            {data.map(({username, realName, email, membership}, index) => (
              <AsTable.Row key={username} isClickable onClick={() => showItem(index)}>
                <AsTable.Cell>{username}</AsTable.Cell>
                <AsTable.Cell>{realName}</AsTable.Cell>
                <AsTable.Cell>{email}</AsTable.Cell>
                <AsTable.Cell>{membership}</AsTable.Cell>
                <AsTable.Cell chevron />
              </AsTable.Row>
            ))}
          </AsTable.Body>
        </AsTable>
      </AsModal.Content>
    </>
  );
}
