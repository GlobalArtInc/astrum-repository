
import React from 'react';

import {
  AsModal,
  AsButton,
  AsH2,
  AsFooter,
  AsP,
  AsButtonBar
} from '@globalart/astrum-react-shared-components';

import UIStrings from '../../../../../constants/UIStrings';
import {useRepositoriesService} from '../RepositoriesContextProvider';
import AnalyzeButton from './AnalyzeButton';

const {HEALTH_CHECK} = UIStrings.REPOSITORIES.LIST;
const {CANCEL_BUTTON_LABEL} = UIStrings.SETTINGS;

export default function AnalyzeConfirmationModal({close, name}) {  
  const [, send] = useRepositoriesService();

  const enableHealthCheck = (name) => {
    close();
    if (name) {
      send({type: 'ENABLE_HELTH_CHECK_SINGLE_REPO', repoName: name});
    } else {
      send({type: 'ENABLE_HELTH_CHECK_ALL_REPOS'});
    }
  };

  return (
    <AsModal
      className="as-enable-health-check-modal"
      onCancel={close}
      aria-labelledby="modal-header-text"
      onClick={(e) => e.stopPropagation()} // allows AsSegmentedButton dropdown to work properly, otherwise it closes immediately
    >
      <AsModal.Header>
        <AsH2 id="modal-header-text">{HEALTH_CHECK.MODAL_HEADER}</AsH2>
      </AsModal.Header>
      <AsModal.Content>
        <AsP>{HEALTH_CHECK.MODAL_CONTENT(name)}</AsP>
      </AsModal.Content>
      <AsFooter>
        <AsButtonBar>
          <AsButton onClick={close}>{CANCEL_BUTTON_LABEL}</AsButton>
          <AnalyzeButton enableHealthCheck={enableHealthCheck} name={name} />
        </AsButtonBar>
      </AsFooter>
    </AsModal>
  );
}
