
import React from 'react';
import {useMachine} from '@xstate/react';

import {FieldWrapper, Utils} from '@globalart/astrum-ui-plugin';
import {
  NxButton,
  NxFilterInput,
  NxFormSelect,
  NxLoadWrapper,
  NxTable,
  NxTableBody,
  NxTableCell,
  NxTableHead,
  NxTableRow,
} from '@globalart/react-shared-components';

import {Section, SectionToolbar} from '@globalart/astrum-ui-plugin';

import UIStrings from '../../../../constants/UIStrings';

import ContentSelectorsPreviewMachine from './ContentSelectorsPreviewMachine';

export default function ContentSelectorsPreview({type, expression}) {
  const [state, send] = useMachine(ContentSelectorsPreviewMachine, {devTools: true});
  const previewUnavailable = Utils.isBlank(expression);
  const {allRepositories, filterText, preview, previewError, repositories} = state.context;
  const isLoading = state.matches('loading');
  const isLoadingPreview = state.matches('preview');

  const repositoryChangeHandler = (repositories) => send({type: 'SET_REPOSITORIES', repositories});
  const previewHandler = () => send({type: 'PREVIEW', selectorType: type, expression});
  const filter = (value) => send({type: 'FILTER', filter: value});

  function retry() {
    send({type: 'RETRY'});
  }

  return <Section className="as-content-selectors-preview">
    <h2>{UIStrings.CONTENT_SELECTORS.TITLE}</h2>
    <NxLoadWrapper isLoading={isLoading} retryHandler={retry}>
      <FieldWrapper labelText={UIStrings.CONTENT_SELECTORS.PREVIEW.REPOSITORY_LABEL}
                    descriptionText={UIStrings.CONTENT_SELECTORS.PREVIEW.REPOSITORY_DESCRIPTION}
                    isOptional={true}
      >
        <div className="nx-form-row">
          <NxFormSelect name="repository" onChange={repositoryChangeHandler} value={repositories}>
            {allRepositories.map(({id, name}) =>
                <option key={id} value={id}>{name}</option>
            )}
          </NxFormSelect>
          <NxButton disabled={previewUnavailable} onClick={previewHandler} data-analytics-id="as-content-selector-preview">{UIStrings.CONTENT_SELECTORS.PREVIEW.BUTTON}</NxButton>
        </div>
      </FieldWrapper>
      <SectionToolbar>
        <div className="as-spacer" />
        <NxFilterInput
            id="filter"
            onChange={filter}
            value={filterText}
            placeholder={UIStrings.CONTENT_SELECTORS.FILTER_PLACEHOLDER}/>
      </SectionToolbar>
      <NxTable>
        <NxTableHead>
          <NxTableRow>
            <NxTableCell>{UIStrings.CONTENT_SELECTORS.PREVIEW.NAME_COLUMN}</NxTableCell>
          </NxTableRow>
        </NxTableHead>
        <NxTableBody isLoading={isLoadingPreview} error={previewError} emptyMessage={UIStrings.CONTENT_SELECTORS.PREVIEW.EMPTY}>
          {preview?.map(name =>
              <NxTableRow key={name}>
                <NxTableCell>{name}</NxTableCell>
              </NxTableRow>
          )}
        </NxTableBody>
      </NxTable>
    </NxLoadWrapper>
  </Section>;
}
