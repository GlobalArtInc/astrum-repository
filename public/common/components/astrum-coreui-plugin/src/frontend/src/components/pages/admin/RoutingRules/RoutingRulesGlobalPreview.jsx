
import React, {Fragment} from 'react';
import {useMachine} from '@xstate/react';
import {faMinusSquare, faPlusSquare} from '@fortawesome/free-solid-svg-icons';

import {
  FieldWrapper,
} from '@globalart/astrum-ui-plugin';

import {
  AsButton,
  AsFilterInput,
  AsFontAwesomeIcon,
  AsFormSelect,
  AsLoadWrapper,
  AsModal,
  AsTable,
  AsTableBody,
  AsTableCell,
  AsTableHead,
  AsTableRow,
} from '@globalart/react-shared-components';

import {
  ContentBody,
  PageTitle,
  Section,
  SectionToolbar,
  Page
} from '@globalart/astrum-ui-plugin';

import UIStrings from '../../../../constants/UIStrings';
import RoutingRulesPreview from './RoutingRulesFormPreview';
import RoutingRulesStatus from './RoutingRulesStatus';
import RoutingRulesGlobalPreviewMachine from './RoutingRulesGlobalPreviewMachine';

import './RoutingRules.scss';

const {ROUTING_RULES} = UIStrings;

export default function RoutingRulesList() {
  const [current, send] = useMachine(RoutingRulesGlobalPreviewMachine, {devTools: true});
  const isLoading = current.matches('preview');
  const isLoadingSelectedRow = current.matches('fetchSelectedRowDetails');
  const viewSelectedRow = isLoadingSelectedRow || current.matches('viewSelectedRow');
  const {error, filter, preview, path, repositories, selectedRule, selectedRowDetails} = current.context;

  function updateRepositories(value) {
    send({type: 'UPDATE_AND_CLEAR', name: 'repositories', value});
  }

  function updatePath(value) {
    send({type: 'UPDATE_AND_CLEAR', name: 'path', value});
  }

  function updateFilter(value) {
    send({type: 'UPDATE', name: 'filter', value});
  }

  function previewHandler() {
    send({type: 'PREVIEW'});
  }

  function selectRow(parent, child) {
    send({type: 'SELECT_ROW', parent, child});
  }

  function toggle(index) {
    send({type: 'TOGGLE', index});
  }

  function closeModal() {
    send({type: 'CLOSE'});
  }

  function retryPreview() {
    send({type: 'PREVIEW'});
  }

  function retryDetails() {
    send({type: 'RETRY'});
  }

  return <Page className="as-routing-rules">
    <PageTitle text={ROUTING_RULES.PREVIEW.TITLE}/>
    <ContentBody className="global-preview">
      <Section>
        <FieldWrapper labelText={ROUTING_RULES.PREVIEW.REPOSITORIES_LABEL}
                      descriptionText={ROUTING_RULES.PREVIEW.REPOSITORIES_DESCRIPTION}>
          <AsFormSelect name="repositories" value={repositories} onChange={updateRepositories}>
            <option value="all">{ROUTING_RULES.PREVIEW.REPOSITORIES.ALL}</option>
            <option value="groups">{ROUTING_RULES.PREVIEW.REPOSITORIES.GROUPS}</option>
            <option value="proxies">{ROUTING_RULES.PREVIEW.REPOSITORIES.PROXIES}</option>
          </AsFormSelect>
        </FieldWrapper>
        <RoutingRulesPreview value={path} onChange={updatePath} onTest={previewHandler} />

        <SectionToolbar>
          <div className="as-spacer"/>
          <AsFilterInput
              id="filter"
              onChange={updateFilter}
              value={filter}
              placeholder={ROUTING_RULES.PREVIEW.FILTER}/>
        </SectionToolbar>
        <AsTable>
          <AsTableHead>
            <AsTableRow>
              <AsTableCell>{ROUTING_RULES.PREVIEW.COLUMNS.REPOSITORY}</AsTableCell>
              <AsTableCell>{ROUTING_RULES.PREVIEW.COLUMNS.TYPE}</AsTableCell>
              <AsTableCell>{ROUTING_RULES.PREVIEW.COLUMNS.FORMAT}</AsTableCell>
              <AsTableCell>{ROUTING_RULES.PREVIEW.COLUMNS.RULE}</AsTableCell>
              <AsTableCell>{ROUTING_RULES.PREVIEW.COLUMNS.STATUS}</AsTableCell>
              <AsTableCell chevron/>
            </AsTableRow>
          </AsTableHead>
          <AsTableBody isLoading={isLoading} error={error} emptyMessage={ROUTING_RULES.PREVIEW.EMPTY_PREVIEW}
                       retryHandler={retryPreview}>
            {preview?.map(({repository, type, format, rule, allowed, expanded, expandable, children}, index) =>
                <Fragment key={`${index}`}>
                  <AsTableRow isClickable={expandable || !!rule}
                              onClick={() => {
                                if (expandable) {
                                  toggle(index);
                                }
                                else if (!!rule) {
                                  selectRow(index);
                                }
                              }}>
                    <AsTableCell>
                      {expandable && expanded && <AsFontAwesomeIcon icon={faMinusSquare}/>}
                      {expandable && !expanded && <AsFontAwesomeIcon icon={faPlusSquare}/>}
                      <span>{repository}</span>
                    </AsTableCell>
                    <AsTableCell>{type}</AsTableCell>
                    <AsTableCell>{format}</AsTableCell>
                    <AsTableCell>{rule || ROUTING_RULES.PREVIEW.NO_RULE}</AsTableCell>
                    <AsTableCell><RoutingRulesStatus allowed={allowed}/></AsTableCell>
                    <AsTableCell chevron={!!rule}/>
                  </AsTableRow>
                  {expanded ? children?.map(({repository, type, format, rule, allowed}, childIndex) =>
                      <AsTableRow key={`${index}-${childIndex}`}
                                  isClickable={!!rule}
                                  onClick={() => !!rule && selectRow(index, childIndex)}>
                        <AsTableCell className="repository-child">{repository}</AsTableCell>
                        <AsTableCell>{type}</AsTableCell>
                        <AsTableCell>{format}</AsTableCell>
                        <AsTableCell>{rule || ROUTING_RULES.PREVIEW.NO_RULE}</AsTableCell>
                        <AsTableCell><RoutingRulesStatus allowed={allowed}/></AsTableCell>
                        <AsTableCell chevron={!!rule}/>
                      </AsTableRow>
                  ) : null}
                </Fragment>)}
          </AsTableBody>
        </AsTable>
      </Section>

      {viewSelectedRow && <AsModal id="view-selected-routing-rule" onClose={closeModal}>
        <header className="as-modal-header"><h2 className="as-h2">{ROUTING_RULES.PREVIEW.DETAILS_TITLE(selectedRule)}</h2></header>
        <div className="as-modal-content">
          <AsLoadWrapper isLoading={isLoadingSelectedRow} retryHandler={retryDetails}>
            <div className="as-form-group">
              <h3 id="mode-label" className="as-label">{ROUTING_RULES.FORM.MODE_LABEL}</h3>
              <p className="as-p" aria-labelledby="mode-label">{selectedRowDetails?.mode}</p>
            </div>
            <div className="as-form-group">
              <h3 id="matchers-label" className="as-label">{ROUTING_RULES.FORM.MATCHERS_LABEL}</h3>
              <ul className="as-list">
              {selectedRowDetails?.matchers?.map(matcher =>
                  <li className="as-list__item"><span className="as-list__text"><code className="as-code" aria-labelledby="matchers-label">{matcher}</code></span></li>
              )}
              </ul>
            </div>

          </AsLoadWrapper>
        </div>
        <footer className="as-footer"><AsButton onClick={closeModal}>{UIStrings.CLOSE}</AsButton></footer>
      </AsModal>}
    </ContentBody>
  </Page>;
}
