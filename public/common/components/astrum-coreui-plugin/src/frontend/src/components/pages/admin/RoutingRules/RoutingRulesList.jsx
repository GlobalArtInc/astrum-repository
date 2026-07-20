
import React, { useCallback } from 'react';
import {useMachine} from '@xstate/react';

import {faMapSigns} from '@fortawesome/free-solid-svg-icons';

import {Utils} from '@globalart/astrum-ui-plugin';
import {
  NxButton,
  NxFilterInput,
  NxTable,
  NxTableBody,
  NxTableCell,
  NxTableHead,
  NxTableRow,
} from '@globalart/react-shared-components';

import {
  ContentBody,
  PageActions,
  PageHeader,
  PageTitle,
  Section,
  SectionToolbar,
  Page
} from '@globalart/astrum-ui-plugin';
import {HelpTile} from '@globalart/astrum-ui-plugin';

import RoutingRulesListMachine from './RoutingRulesListMachine';

import UIStrings from '../../../../constants/UIStrings';
import { useRouter } from '@uirouter/react';
import { ROUTE_NAMES } from '../../../../routerConfig/routeNames/routeNames';

import './RoutingRules.scss';

const {ROUTING_RULES: {LIST: LABELS, MENU}} = UIStrings;

export default function RoutingRulesList() {
  const router = useRouter();
  const onEdit = useCallback(itemId => router.stateService.go(ROUTE_NAMES.ADMIN.REPOSITORY.ROUTINGRULES.EDIT, { itemId }));
  const onCreate = useCallback(() => router.stateService.go(ROUTE_NAMES.ADMIN.REPOSITORY.ROUTINGRULES.CREATE));
  const onPreview = useCallback(() => router.stateService.go(ROUTE_NAMES.ADMIN.REPOSITORY.ROUTINGRULES.PREVIEW));

  const [current, send] = useMachine(RoutingRulesListMachine, {devTools: true});
  const isLoading = current.matches('loading');
  const data = current.context.data;
  const filterText = current.context.filter;
  const error = current.context.error;

  function getSortDirection(fieldName, sortField, sortDirection) {
    if (sortField !== fieldName) {
      return null;
    }
    return sortDirection === Utils.ASC ? 'asc' : 'desc';
  }

  const nameSortDir = getSortDirection('name', current.context.sortField, current.context.sortDirection);
  const descriptionSortDir = getSortDirection('description', current.context.sortField, current.context.sortDirection);
  const usedBySortDir = getSortDirection('assignedRepositoryCount', current.context.sortField, current.context.sortDirection);

  function filter(value) {
    send({type: 'FILTER', filter: value});
  }

  function preview() {
    send({type: 'PREVIEW'});
    onPreview();
  }

  return <Page className="as-routing-rules">
    <PageHeader>
      <PageTitle icon={faMapSigns} {...MENU}/>
      <PageActions>
        <NxButton variant="tertiary" onClick={preview}>
          <span>{LABELS.PREVIEW_BUTTON}</span>
        </NxButton>
        <NxButton variant="primary" onClick={onCreate}>
          <span>{LABELS.CREATE_BUTTON}</span>
        </NxButton>
      </PageActions>
    </PageHeader>
    <ContentBody>
      <Section className="as-routing-rules-list">
        <SectionToolbar>
          <div className="as-spacer" />
          <NxFilterInput
              id="filter"
              onChange={filter}
              value={filterText}
              placeholder={UIStrings.FILTER}/>
        </SectionToolbar>
        <NxTable>
          <NxTableHead>
            <NxTableRow>
              <NxTableCell onClick={() => send({type: 'SORT_BY_NAME'})} isSortable sortDir={nameSortDir}>
                {LABELS.NAME_LABEL}
              </NxTableCell>
              <NxTableCell onClick={() => send({type: 'SORT_BY_DESCRIPTION'})} isSortable sortDir={descriptionSortDir}>
                {LABELS.DESCRIPTION_LABEL}
              </NxTableCell>
              <NxTableCell onClick={() => send({type: 'SORT_BY_ASSIGNED_REPOSITORY_COUNT'})} isSortable sortDir={usedBySortDir}>
                {LABELS.USED_BY_LABEL}
              </NxTableCell>
              <NxTableCell chevron/>
            </NxTableRow>
          </NxTableHead>
          <NxTableBody isLoading={isLoading} error={error} emptyMessage={LABELS.EMPTY_LIST}>
            {data.map(({name, description, assignedRepositoryCount}) => (
                <NxTableRow key={name} onClick={() => onEdit(name)} isClickable>
                  <NxTableCell>{name}</NxTableCell>
                  <NxTableCell>{description}</NxTableCell>
                  <NxTableCell>
                    {assignedRepositoryCount === 0 ?
                        <a href="#admin/repository/repositories">{LABELS.NEEDS_ASSIGNMENT}</a> :
                        <span>{LABELS.USED_BY(assignedRepositoryCount)}</span>
                    }
                  </NxTableCell>
                  <NxTableCell chevron/>
                </NxTableRow>
            ))}
          </NxTableBody>
        </NxTable>
      </Section>
      <HelpTile header={LABELS.HELP_TITLE} body={LABELS.HELP_TEXT}/>
    </ContentBody>
  </Page>;
}
