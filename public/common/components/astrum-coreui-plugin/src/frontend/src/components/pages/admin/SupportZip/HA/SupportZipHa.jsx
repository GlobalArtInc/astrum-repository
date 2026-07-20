
import React from 'react';
import classNames from 'classnames';
import UIStrings from '../../../../../constants/UIStrings';

const {SUPPORT_ZIP: LABELS} = UIStrings;

import {
  NxCard,
  NxPageTitle,
  NxH2,
  NxButton,
} from '@globalart/react-shared-components';

import {faArchive} from '@fortawesome/free-solid-svg-icons';

import {useMachine} from '@xstate/react';

import {
  ContentBody,
  Page,
  PageActions,
  PageHeader,
  PageTitle
} from '@globalart/astrum-ui-plugin';

import SupportZipHaMachine from './SupportZipHaMachine';
import SupportZipMachine from '../SupportZipMachine';

import NodeCard from './NodeCard';
import SupportZipHaModalForm from './SupportZipHaModalForm';

import './SupportZipHa.scss';

export default function SupportZipHa() {
  const [astrumNodesState, sendToAstrumNodes] = useMachine(SupportZipHaMachine, {
    devTools: true,
  });

  const {astrumNodes, isBlobStoreConfigured} = astrumNodesState.context;

  const showCreateZipModal =
    astrumNodesState.matches('loaded.createSingleNodeSupportZip') ||
    astrumNodesState.matches('loaded.createAllNodesSupportZip');

  const [supportZipState, sendToSupportZip] = useMachine(SupportZipMachine, {
    devTools: true,
  });

  const {params} = supportZipState.context;

  function showCreateZipModalForm(node) {
    sendToAstrumNodes({
      type: 'CREATE_SUPPORT_ZIP_FOR_NODE',
      node,
    });
  }

  function closeSupportZipFormModal() {
    sendToAstrumNodes('CANCEL');
  }

  function setSupportFormParams(name, value) {
    sendToSupportZip({
      type: 'UPDATE',
      params: {
        ...params,
        [name]: value,
      },
    });
  }

  function submitCreateNodeSupportZip() {
    sendToAstrumNodes({type: 'GENERATE', params});
  }

  function generateSupportZipForAllNodes() {
    if (isBlobStoreConfigured) {
      sendToAstrumNodes('CREATE_SUPPORT_ZIP_FOR_ALL_NODES');
    }
  }

  return (
    <Page className="as-support-zip-ha">
      <PageHeader>
        <PageTitle icon={faArchive} {...LABELS.MENU_HA} />
        <PageActions>
          <NxButton
            type="button"
            variant="primary"
            className={classNames({
              disabled: !isBlobStoreConfigured,
            })}
            onClick={generateSupportZipForAllNodes}
          >
            {LABELS.GENERATE_ALL_ZIP_FILES}
          </NxButton>
        </PageActions>
      </PageHeader>

      <ContentBody>
        <NxPageTitle>
          <NxH2>{LABELS.AVAILABLE_NODES}</NxH2>
        </NxPageTitle>

        <NxCard.Container className="as-nodes-container">
          {astrumNodes.map(({...node}) => (
            <NodeCard
              key={node.nodeId}
              actor={node.machineRef}
              createZip={() => showCreateZipModalForm(node)}
              isBlobStoreConfigured={isBlobStoreConfigured}
            />
          ))}
        </NxCard.Container>
      </ContentBody>

      {showCreateZipModal && (
        <SupportZipHaModalForm
          formParams={params}
          setFormParams={setSupportFormParams}
          submitHandler={submitCreateNodeSupportZip}
          cancelHandler={closeSupportZipFormModal}
        />
      )}
    </Page>
  );
}
