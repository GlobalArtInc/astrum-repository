
import React from 'react';
import {useActor} from '@xstate/react';
import {NxModal, NxH2, NxTable} from '@globalart/react-shared-components';
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
      <NxModal.Header>
        <NxH2 id="modal-form-header">{HEADER}</NxH2>
      </NxModal.Header>
      <NxModal.Content>
        <NxTable>
          <NxTable.Head>
            <NxTable.Row>
              <NxTable.Cell>{USER_ID}</NxTable.Cell>
              <NxTable.Cell>{NAME}</NxTable.Cell>
              <NxTable.Cell>{EMAIL}</NxTable.Cell>
              <NxTable.Cell>{ROLES}</NxTable.Cell>
              <NxTable.Cell chevron />
            </NxTable.Row>
          </NxTable.Head>
          <NxTable.Body
            isLoading={isLoading}
            error={error}
            retryHandler={retryHandler}
            emptyMessage={EMPTY_MESSAGE}
          >
            {data.map(({username, realName, email, membership}, index) => (
              <NxTable.Row key={username} isClickable onClick={() => showItem(index)}>
                <NxTable.Cell>{username}</NxTable.Cell>
                <NxTable.Cell>{realName}</NxTable.Cell>
                <NxTable.Cell>{email}</NxTable.Cell>
                <NxTable.Cell>{membership}</NxTable.Cell>
                <NxTable.Cell chevron />
              </NxTable.Row>
            ))}
          </NxTable.Body>
        </NxTable>
      </NxModal.Content>
    </>
  );
}
