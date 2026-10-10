
import React from 'react';
import {useMachine} from '@xstate/react';
import {faGlobe} from '@fortawesome/free-solid-svg-icons';
import {ExtJS} from '@globalart/astrum-ui-plugin';

import {AsButton, AsLoadWrapper} from '@globalart/astrum-react-shared-components';

import {
  ContentBody,
  Page,
  PageActions,
  PageHeader,
  PageTitle
} from '@globalart/astrum-ui-plugin';

import UIStrings from '../../../../constants/UIStrings';
import SystemInformationBody from './SystemInformationBody';
import NodeSelector from './NodeSelector';
import SystemInformationHaMachine from './SystemInformationHaMachine';
import SystemInformationMachine from './SystemInformationMachine';

import './SystemInformation.scss';

const {MENU, ACTIONS, LOAD_ERROR} = UIStrings.SYSTEM_INFORMATION;

export default function SystemInformation() {
  const isCluster = ExtJS.state().getValue('astrum.datastore.clustered.enabled');

  const machine = isCluster ? SystemInformationHaMachine : SystemInformationMachine;

  const [state, send, service] = useMachine(machine, {devTools: true});

  const isLoading = state.matches('loading');
  const loadError = state.matches('loadError') ? LOAD_ERROR : null;

  const {selectedNodeId, systemInformation} = state.context;

  function retry() {
    send({type: 'RETRY'});
  }

  function downloadSystemInformation() {
    const fileName = `sysinfo${selectedNodeId ? '_' + selectedNodeId : ''}.json`;
    saveAsFile(systemInformation, fileName);
  }

  return (
    <Page>
      <PageHeader>
        <PageTitle icon={faGlobe} {...MENU} />
        <PageActions>
          <AsButton
            variant="primary"
            onClick={downloadSystemInformation}
            disabled={isLoading || loadError}
          >
            <span>{ACTIONS.download}</span>
          </AsButton>
        </PageActions>
      </PageHeader>

      {isCluster && <NodeSelector service={service} />}

      <ContentBody className="as-system-information">
        <AsLoadWrapper loading={isLoading} error={loadError} retryHandler={retry}>
          <SystemInformationBody systemInformation={systemInformation} />
        </AsLoadWrapper>
      </ContentBody>
    </Page>
  );
}

const saveAsFile = (obj, filename) => {
  const content = JSON.stringify(obj, null, 2);
  const link = document.createElement('a');
  const blob = new Blob([content], {type: 'application/json;charset=UTF-8'});
  link.href = URL.createObjectURL(blob);
  link.download = filename;
  link.click();
  URL.revokeObjectURL(link.href);
};
