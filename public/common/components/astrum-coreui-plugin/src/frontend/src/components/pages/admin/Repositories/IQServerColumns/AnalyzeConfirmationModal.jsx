
import React from 'react';

import {
  NxModal,
  NxButton,
  NxH2,
  NxFooter,
  NxP,
  NxButtonBar
} from '@globalart/react-shared-components';

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
    <NxModal
      className="as-enable-health-check-modal"
      onCancel={close}
      aria-labelledby="modal-header-text"
      onClick={(e) => e.stopPropagation()} // allows NxSegmentedButton dropdown to work properly, otherwise it closes immediately
    >
      <NxModal.Header>
        <NxH2 id="modal-header-text">{HEALTH_CHECK.MODAL_HEADER}</NxH2>
      </NxModal.Header>
      <NxModal.Content>
        <NxP>{HEALTH_CHECK.MODAL_CONTENT(name)}</NxP>
      </NxModal.Content>
      <NxFooter>
        <NxButtonBar>
          <NxButton onClick={close}>{CANCEL_BUTTON_LABEL}</NxButton>
          <AnalyzeButton enableHealthCheck={enableHealthCheck} name={name} />
        </NxButtonBar>
      </NxFooter>
    </NxModal>
  );
}
