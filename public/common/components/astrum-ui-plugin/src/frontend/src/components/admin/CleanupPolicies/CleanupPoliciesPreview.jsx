
import React, {useEffect} from 'react';
import {useMachine} from '@xstate/react';

import ListMachineUtils from '../../../interface/ListMachineUtils';
import ValidationUtils from '../../../interface/ValidationUtils';
import {
  AsButton,
  AsFilterInput,
  AsFormSelect,
  AsLoadWrapper,
  AsTable,
  AsTableBody,
  AsTableCell,
  AsTableHead,
  AsTableRow,
  AsTooltip,
  AsWarningAlert
} from '@globalart/astrum-react-shared-components';

import {Section, SectionToolbar} from '../../layout';

import UIStrings from '../../../constants/UIStrings';

import CleanupPoliciesPreviewFormMachine from './CleanupPoliciesPreviewFormMachine';
import CleanupPoliciesPreviewListMachine from './CleanupPoliciesPreviewListMachine';

export default function CleanupPoliciesPreview({policyData}) {
  const [formState, sendToForm] = useMachine(CleanupPoliciesPreviewFormMachine, {devTools: true});
  const [listState, sendToList] = useMachine(CleanupPoliciesPreviewListMachine, {devTools: true});

  useEffect(() => {
    sendToForm({type: 'LOAD_REPOSITORIES', format: policyData.format});
    sendToList({type: 'CLEAR_PREVIEW'});
  }, [policyData]);

  const {error: repositoryLoadError, repository, repositories} = formState.context;
  const {error: previewError, data, total, filter, isAlertShown} = listState.context;
  const previewUnavailable = ValidationUtils.isBlank(repository) || (
      ValidationUtils.isBlank(policyData.criteriaLastBlobUpdated) &&
      ValidationUtils.isBlank(policyData.criteriaLastDownloaded) &&
      ValidationUtils.isBlank(policyData.criteriaReleaseType) &&
      ValidationUtils.isBlank(policyData.criteriaAssetRegex)
  );
  const isLoadingRepositories = formState.matches('loading');
  const isLoadingPreview = listState.matches('loading');
  const hasData = data?.length > 0;

  const nameSortDir = ListMachineUtils.getSortDirection('name', listState.context);
  const groupSortDir = ListMachineUtils.getSortDirection('group', listState.context);
  const versionSortDir = ListMachineUtils.getSortDirection('version', listState.context);
  const sortByName = () => sendToList({type: 'SORT_BY_NAME'});
  const sortByGroup = () => sendToList({type: 'SORT_BY_GROUP'});
  const sortByVersion = () => sendToList({type: 'SORT_BY_VERSION'});

  function repositoryChangeHandler(repository) {
    sendToForm({type: 'SET_REPOSITORY', repository});
  }

  function retryForm() {
    sendToForm({type: 'LOAD_REPOSITORIES', format: policyData.format});
  }

  function previewHandler() {
    sendToList({type: 'PREVIEW', repository, policyData});
  }

  function retryPreview() {
    sendToList({type: 'RETRY_PREVIEW'});
  }

  function filterPreview(filter) {
    sendToList({type: 'FILTER', filter});
  }

  return <Section className="as-cleanup-policies-preview">
    <AsLoadWrapper loading={isLoadingRepositories} error={repositoryLoadError} retryHandler={retryForm}>
      {() => <>
        <div className="as-form-group cleanup-preview-repository-group">
          <label id="preview-repository-label" className="as-label as-label--optional" htmlFor="repository">
            <span className="as-label__text">{UIStrings.CLEANUP_POLICIES.PREVIEW.REPOSITORY_LABEL}</span>
          </label>
          <div id="preview-repository-sub-label" className="as-sub-label">
            {UIStrings.CLEANUP_POLICIES.PREVIEW.REPOSITORY_DESCRIPTION}
          </div>
          <div className="as-form-row">
            <AsFormSelect
                id="repository"
                name="repository"
                onChange={repositoryChangeHandler}
                value={repository}
                aria-describedby="preview-repository-sub-label"
                disabled={!policyData.format}
            >
              <option value="">{UIStrings.CLEANUP_POLICIES.REPOSITORY_SELECT}</option>
              {repositories.map(({id, name}) =>
                  <option key={id} value={id}>{name}</option>
              )}
            </AsFormSelect>
            <AsTooltip title={previewUnavailable ? UIStrings.CLEANUP_POLICIES.PREVIEW.BUTTON_TOOLTIP : undefined}>
              <AsButton className={previewUnavailable ? 'disabled' : ''} onClick={previewHandler}>
                {UIStrings.CLEANUP_POLICIES.PREVIEW.BUTTON}
              </AsButton>
            </AsTooltip>
          </div>
        </div>
        {isAlertShown &&
            <AsWarningAlert onClose={() => sendToList({type: 'HIDE_ALERT'})}>
              <p className="as-p">{UIStrings.CLEANUP_POLICIES.PREVIEW.SAMPLE_WARNING}</p>
              <p className="as-p">{UIStrings.CLEANUP_POLICIES.PREVIEW.COMPONENT_COUNT(data.length, total)}</p>
            </AsWarningAlert>}
        <SectionToolbar>
          <div className="as-spacer"/>
          <AsFilterInput
              id="filter"
              onChange={filterPreview}
              value={filter}
              placeholder={UIStrings.CLEANUP_POLICIES.FILTER_PLACEHOLDER}/>
        </SectionToolbar>
        <AsTable>
          <AsTableHead>
            <AsTableRow>
              <AsTableCell onClick={sortByName} isSortable sortDir={nameSortDir}>
                {UIStrings.CLEANUP_POLICIES.PREVIEW.NAME_COLUMN}
              </AsTableCell>
              <AsTableCell onClick={sortByGroup} isSortable sortDir={groupSortDir}>
                {UIStrings.CLEANUP_POLICIES.PREVIEW.GROUP_COLUMN}
              </AsTableCell>
              <AsTableCell onClick={sortByVersion} isSortable sortDir={versionSortDir}>
                {UIStrings.CLEANUP_POLICIES.PREVIEW.VERSION_COLUMN}
              </AsTableCell>
            </AsTableRow>
          </AsTableHead>
          <AsTableBody isLoading={isLoadingPreview}
                       error={previewError}
                       retryHandler={retryPreview}
                       emptyMessage={UIStrings.CLEANUP_POLICIES.PREVIEW.EMPTY}>
            {hasData && data?.map(({name, group, version, repository}) =>
                <AsTableRow key={`${name}${group}${version}${repository}`}>
                  <AsTableCell>{name}</AsTableCell>
                  <AsTableCell>{group}</AsTableCell>
                  <AsTableCell>{version}</AsTableCell>
                </AsTableRow>
            )}
          </AsTableBody>
        </AsTable>
      </>}
    </AsLoadWrapper>
  </Section>;
}
