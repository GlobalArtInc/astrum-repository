
import React from 'react';
import {useMachine} from '@xstate/react';
import {
  NxModal,
  NxH2,
  NxFooter, NxButton, NxButtonBar,
} from '@globalart/react-shared-components';
import {
  faDatabase,
  faMedkit,
  faSearch,
  faLink,
} from '@fortawesome/free-solid-svg-icons';
import {ExtJS, Permissions} from '@globalart/astrum-ui-plugin';

import QuickAction from './QuickAction';
import OutreachActionsMachine from './OutreachActionsMachine';
import UIStrings from '../../../../constants/UIStrings';

import './OutreachActions.scss';

const {WELCOME: {
  ACTIONS: {
    SYSTEM_HEALTH,
    CLEANUP_POLICIES,
    BROWSE,
    SEARCH,
    CONNECT,
  },
  CONNECT_MODAL,
}} = UIStrings;


const ACTIONS = [
  {
    title: SYSTEM_HEALTH.title,
    subTitle: SYSTEM_HEALTH.subTitle,
    icon: faMedkit,
    isVisible: () => ExtJS.checkPermission(Permissions.METRICS.READ) && ExtJS.state().getUser(),
    event: {type: 'REDIRECT', url: '#admin/support/status'},
  },
  {
    title: CLEANUP_POLICIES.title,
    subTitle: CLEANUP_POLICIES.subTitle,
    icon: faSearch,
    isVisible: () => ExtJS.checkPermission(Permissions.ADMIN) && ExtJS.state().getUser(),
    event: {type: 'REDIRECT', url: '#admin/repository/cleanuppolicies'},
  },
  {
    title: BROWSE.title,
    subTitle: BROWSE.subTitle,
    icon: faDatabase,
    isVisible: () => ExtJS.state().getValue('browseableformats').length > 0,
    event: {type: 'REDIRECT', url: '#browse/browse'},
  },
  {
    title: SEARCH.title,
    subTitle: SEARCH.subTitle,
    icon: faSearch,
    isVisible: () => ExtJS.checkPermission(Permissions.SEARCH.READ) && ExtJS.state().getUser(),
    event: {type: 'REDIRECT', url: '#browse/search/generic'},
  },
  {
    title: CONNECT.title,
    subTitle: CONNECT.subTitle,
    icon: faLink,
    isVisible: (context) => {
      return ExtJS.state().getValue('browseableformats').length > 0 && context.showConnectAction;
    },
    event: {type: 'OPEN_CONNECT_MODAL'},
  },
];

const getActiveActions = (context) => {
  const maxSize = 3;
  const activeActions = ACTIONS.filter(action => action.isVisible(context));
  return activeActions.slice(0, maxSize);
}

export default function OutreachActions() {
  const [state, send] = useMachine(OutreachActionsMachine, {devTools: true});
  const showConnectModal = state.matches('showingConnectModal');
  const activeActions = getActiveActions(state.context);
  const closeConnectModal = () => send({type: 'CLOSE_MODAL'});

  return (
      <>
        <div className="as-outreach-actions">
          {activeActions.map(({title, subTitle, icon, event}) => (
              <QuickAction
                  key={title}
                  title={title}
                  subTitle={subTitle}
                  icon={icon}
                  action={() => send(event)}
                  className="as-outreach-action"
              />
          ))}
        </div>
        {showConnectModal &&
            <NxModal
                onCancel={closeConnectModal}
                aria-labelledby="modal-header-text"
                className="as-connect-modal"
            >
              <NxModal.Header>
                <NxH2 id="modal-header-text">{CONNECT_MODAL.TITLE}</NxH2>
              </NxModal.Header>
              <NxModal.Content>
                <dl className="as-connect-modal__steps">
                  <dt>{CONNECT_MODAL.FIRST_STEP_TEXT}</dt>
                  <dd>
                    <img
                        className="as-connect-modal__repositories-table-img"
                        alt="Repositories Table"
                        src="./static/rapture/resources/images/welcome/repositories_table.png"
                    />
                  </dd>
                  <dt>{CONNECT_MODAL.SECOND_STEP_TEXT}</dt>
                  <dd>
                    <img
                        alt="Copy repository URL"
                        src="./static/rapture/resources/images/welcome/copy_repository_url.png"
                    />
                  </dd>
                </dl>
              </NxModal.Content>
              <NxFooter>
                <NxButtonBar>
                  <NxButton
                      type="button"
                      variant="primary"
                      onClick={closeConnectModal}
                  >
                    {UIStrings.CLOSE}
                  </NxButton>
                </NxButtonBar>
              </NxFooter>
            </NxModal>
        }
      </>
  );
};
