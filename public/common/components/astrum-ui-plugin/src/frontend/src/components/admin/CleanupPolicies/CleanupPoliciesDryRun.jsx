
import React, {useEffect} from 'react';
import {useMachine} from '@xstate/react';

import {
  NxButtonBar,
  NxFormRow,
  NxFormSelect,
  NxH2,
  NxLoadWrapper,
  NxP,
  NxTooltip
} from '@globalart/react-shared-components';
import ExtJS from '../../../interface/ExtJS';
import FormUtils from '../../../interface/FormUtils';

import UIStrings from '../../../constants/UIStrings';
import cleanupPoliciesDryRunMachine from './CleanupPoliciesDryRunMachine';

const {BUTTON, REPOSITORY_DESCRIPTION, REPOSITORY_SELECT, TITLE} = UIStrings.CLEANUP_POLICIES.DRY_RUN;

export default function CleanupPoliciesDryRun({policyData}) {
  const [state, send] = useMachine(cleanupPoliciesDryRunMachine, {devTools: true});
  const {loadError, repository, repositories} = state.context;
  const isLoading = state.matches('loading');
  const criteriaSelected = (policyData.criteriaLastDownloaded || policyData.criteriaLastBlobUpdated || policyData.criteriaAssetRegex);
  const generateCSVUrl = ExtJS.urlOf('/service/rest/internal/cleanup-policies/preview/components/csv' +
    '?repository=' + repository +
    (policyData.name ? `&name=${encodeURIComponent(policyData.name)}` : '') +
    (policyData.criteriaLastBlobUpdated ? `&criteriaLastBlobUpdated=${policyData.criteriaLastBlobUpdated}` : '') +
    (policyData.criteriaLastDownloaded ? `&criteriaLastDownloaded=${policyData.criteriaLastDownloaded}` : '') +
    (policyData.criteriaReleaseType ? `&criteriaReleaseType=${policyData.criteriaReleaseType}` : '') +
    (policyData.criteriaAssetRegex ? `&criteriaAssetRegex=${encodeURIComponent(policyData.criteriaAssetRegex)}` : '') +
    (policyData.retain? `&criteriaRetain=${policyData.retain}` : '') +
    (policyData.sortBy? `&criteriaSortBy=${policyData.sortBy}` : ''))

  useEffect(() => {
    send({type: 'LOAD_REPOSITORIES', format: policyData.format});
  }, [policyData.format]);

  function repositoryChangeHandler(repository) {
    send({type: 'SET_REPOSITORY', repository});
  }

  function retry() {
    send({type: 'RETRY', format: policyData.format});
  }

  return <div className="as-cleanup-policies-dry-run">
    <NxLoadWrapper loading={isLoading} error={loadError} retryHandler={retry}>
      {() =>
        <>
          <NxH2>{TITLE}</NxH2>
          <NxP id="dryRunDesc">{REPOSITORY_DESCRIPTION}</NxP>
          <NxFormRow>
            <NxFormSelect aria-describedby="dryRunDesc"
                          disabled={!policyData.format}
                          {...FormUtils.fieldProps('repository', state)}
                          onChange={repositoryChangeHandler}
                          value={repository}>
              <option value="">{REPOSITORY_SELECT}</option>
              {repositories?.map(({id, name}) =>
                <option key={id} value={id}>{name}</option>
              )}
            </NxFormSelect>
            <NxButtonBar>
              <NxTooltip title={!repository || !criteriaSelected ? UIStrings.CLEANUP_POLICIES.DRY_RUN.BUTTON_TOOLTIP : undefined}>
                <a aria-disabled={!repository || !criteriaSelected}
                  className={`nx-btn ${(!repository || !criteriaSelected) ? 'disabled' : ''}`}
                  download
                  href={!repository || !criteriaSelected ? undefined : generateCSVUrl}
                  role={!repository || !criteriaSelected ? 'link' : undefined}>
                  {BUTTON}
                </a>
              </NxTooltip>
            </NxButtonBar>
          </NxFormRow>
        </>
      }
    </NxLoadWrapper>
  </div>
}
