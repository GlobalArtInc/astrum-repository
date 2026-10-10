
import React from 'react';
import {useMachine} from '@xstate/react';

import {FieldWrapper, Utils} from '@globalart/astrum-ui-plugin';
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
} from '@globalart/astrum-react-shared-components';

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
    <AsLoadWrapper isLoading={isLoading} retryHandler={retry}>
      <FieldWrapper labelText={UIStrings.CONTENT_SELECTORS.PREVIEW.REPOSITORY_LABEL}
                    descriptionText={UIStrings.CONTENT_SELECTORS.PREVIEW.REPOSITORY_DESCRIPTION}
                    isOptional={true}
      >
        <div className="as-form-row">
          <AsFormSelect name="repository" onChange={repositoryChangeHandler} value={repositories}>
            {allRepositories.map(({id, name}) =>
                <option key={id} value={id}>{name}</option>
            )}
          </AsFormSelect>
          <AsButton disabled={previewUnavailable} onClick={previewHandler} data-analytics-id="as-content-selector-preview">{UIStrings.CONTENT_SELECTORS.PREVIEW.BUTTON}</AsButton>
        </div>
      </FieldWrapper>
      <SectionToolbar>
        <div className="as-spacer" />
        <AsFilterInput
            id="filter"
            onChange={filter}
            value={filterText}
            placeholder={UIStrings.CONTENT_SELECTORS.FILTER_PLACEHOLDER}/>
      </SectionToolbar>
      <AsTable>
        <AsTableHead>
          <AsTableRow>
            <AsTableCell>{UIStrings.CONTENT_SELECTORS.PREVIEW.NAME_COLUMN}</AsTableCell>
          </AsTableRow>
        </AsTableHead>
        <AsTableBody isLoading={isLoadingPreview} error={previewError} emptyMessage={UIStrings.CONTENT_SELECTORS.PREVIEW.EMPTY}>
          {preview?.map(name =>
              <AsTableRow key={name}>
                <AsTableCell>{name}</AsTableCell>
              </AsTableRow>
          )}
        </AsTableBody>
      </AsTable>
    </AsLoadWrapper>
  </Section>;
}
