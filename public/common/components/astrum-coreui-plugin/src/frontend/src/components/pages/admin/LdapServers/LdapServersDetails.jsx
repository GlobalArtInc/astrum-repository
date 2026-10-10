

import React, {useRef} from 'react';
import {useMachine} from '@xstate/react';
import {
  AsTile,
  AsTabs,
  AsTabList,
  AsTab,
  AsTabPanel,
  AsLoadWrapper,
} from '@globalart/astrum-react-shared-components';
import {faBook} from '@fortawesome/free-solid-svg-icons';

import {PageHeader, PageTitle} from '@globalart/astrum-ui-plugin';

import LdapServersConfigurationForm from './LdapServersConfigurationForm';
import LdapServersUserAndGroupForm from './LdapServersUserAndGroupForm';
import LdapServersConfigurationFormReadOnly from './LdapServersConfigurationFormReadOnly';
import LdapServersUserAndGroupFormReadOnly from './LdapServersUserAndGroupFormReadOnly';
import {canUpdate} from './LdapServersHelper';
import Machine from './LdapServersDetailsMachine';

import UIStrings from '../../../../constants/UIStrings';

const {
  LDAP_SERVERS: {MENU, FORM},
} = UIStrings;
import {TABS_INDEX} from './LdapServersHelper';
import {isNil} from 'ramda';

export default function LdapServersDetails({itemId, onDone}) {
  const readOnly = !canUpdate();
  const isEdit = canUpdate() && Boolean(itemId);
  const isCreate = !isEdit && canUpdate();

  const ref = useRef();
  const [state, send] = useMachine(Machine, {
    devTools: true,
    context: {
      itemId: itemId ? decodeURIComponent(itemId) : null,
      isEdit,
    },
    actions: {
      onSaveSuccess: onDone,
      onDeleteSuccess: onDone,
    },
  });
  const connection = state.matches('loaded.creatingConnection');
  const userAndGroup = state.matches('loaded.creatingUserAndGroup');
  const {activeTab} = state.context;

  const onTabSelectedEditMode = (value) => {
    if (TABS_INDEX.CREATE_CONNECTION === value) {
      send({type: 'CREATE_CONNECTION'});
    }

    if (TABS_INDEX.USER_AND_GROUP === value && !isNil(ref.current)) {
      // We use the submit event instead of xstate events to be able use the same validation defined for the form.
      const submitButton = ref.current.querySelector('.as-form__submit-btn');
      submitButton.click();
    }
  };

  const onTabSelectedReadOnlyMode = (value) => {
    if (TABS_INDEX.CREATE_CONNECTION === value) {
      send({type: 'CREATE_CONNECTION'});
    }

    if (TABS_INDEX.USER_AND_GROUP === value) {
      send({type: 'NEXT'});
    }
  };

  const onTabSelected = (value) => {
    if (isEdit) {
      onTabSelectedEditMode(value);
    }

    if (readOnly) {
      onTabSelectedReadOnlyMode(value);
    }
  };

  const retryHandler = () => send({type: 'RETRY'});
  const isLoading = state.matches('loading');
  const hasLoadError = state.matches('loadError');

  const configurationForm = () =>
    connection && (
      <LdapServersConfigurationForm
        actor={state.context.createConnection}
        onDone={onDone}
        ref={ref}
      />
    );

  const userAndGroupForm = () =>
    userAndGroup && (
      <LdapServersUserAndGroupForm
        actor={state.context.userAndGroup}
        onDone={onDone}
      />
    );

  return (
    <div className="as-ldap-servers">
      <PageHeader>
        <PageTitle icon={faBook} {...MENU} />
      </PageHeader>
      <AsTile>
        <AsLoadWrapper
          retryHandler={retryHandler}
          loading={isLoading}
          error={hasLoadError && error}
        >
          {(isEdit || readOnly) && (
            <AsTabs activeTab={activeTab} onTabSelect={onTabSelected}>
              <AsTabList>
                <AsTab>{FORM.TABS.CONNECTION}</AsTab>
                <AsTab>{FORM.TABS.USER_AND_GROUP}</AsTab>
              </AsTabList>
              <AsTabPanel>
                {isEdit && configurationForm()}
                {readOnly && (
                  <LdapServersConfigurationFormReadOnly
                    actor={state.context.createConnection}
                  />
                )}
              </AsTabPanel>
              <AsTabPanel>
                {isEdit && userAndGroupForm()}
                {readOnly && (
                  <LdapServersUserAndGroupFormReadOnly
                    actor={state.context.userAndGroup}
                  />
                )}
              </AsTabPanel>
            </AsTabs>
          )}
          {isCreate && (
            <>
              {configurationForm()}
              {userAndGroupForm()}
            </>
          )}
        </AsLoadWrapper>
      </AsTile>
    </div>
  );
}
