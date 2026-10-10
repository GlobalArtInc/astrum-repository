
import React, { useCallback } from 'react';
import {useMachine} from '@xstate/react';

import {faMapSigns} from '@fortawesome/free-solid-svg-icons';

import {Utils} from '@globalart/astrum-ui-plugin';
import {
  AsButton,
  AsFilterInput,
  AsTable,
  AsTableBody,
  AsTableCell,
  AsTableHead,
  AsTableRow,
} from '@globalart/astrum-react-shared-components';

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
        <AsButton variant="tertiary" onClick={preview}>
          <span>{LABELS.PREVIEW_BUTTON}</span>
        </AsButton>
        <AsButton variant="primary" onClick={onCreate}>
          <span>{LABELS.CREATE_BUTTON}</span>
        </AsButton>
      </PageActions>
    </PageHeader>
    <ContentBody>
      <Section className="as-routing-rules-list">
        <SectionToolbar>
          <div className="as-spacer" />
          <AsFilterInput
              id="filter"
              onChange={filter}
              value={filterText}
              placeholder={UIStrings.FILTER}/>
        </SectionToolbar>
        <AsTable>
          <AsTableHead>
            <AsTableRow>
              <AsTableCell onClick={() => send({type: 'SORT_BY_NAME'})} isSortable sortDir={nameSortDir}>
                {LABELS.NAME_LABEL}
              </AsTableCell>
              <AsTableCell onClick={() => send({type: 'SORT_BY_DESCRIPTION'})} isSortable sortDir={descriptionSortDir}>
                {LABELS.DESCRIPTION_LABEL}
              </AsTableCell>
              <AsTableCell onClick={() => send({type: 'SORT_BY_ASSIGNED_REPOSITORY_COUNT'})} isSortable sortDir={usedBySortDir}>
                {LABELS.USED_BY_LABEL}
              </AsTableCell>
              <AsTableCell chevron/>
            </AsTableRow>
          </AsTableHead>
          <AsTableBody isLoading={isLoading} error={error} emptyMessage={LABELS.EMPTY_LIST}>
            {data.map(({name, description, assignedRepositoryCount}) => (
                <AsTableRow key={name} onClick={() => onEdit(name)} isClickable>
                  <AsTableCell>{name}</AsTableCell>
                  <AsTableCell>{description}</AsTableCell>
                  <AsTableCell>
                    {assignedRepositoryCount === 0 ?
                        <a href="#admin/repository/repositories">{LABELS.NEEDS_ASSIGNMENT}</a> :
                        <span>{LABELS.USED_BY(assignedRepositoryCount)}</span>
                    }
                  </AsTableCell>
                  <AsTableCell chevron/>
                </AsTableRow>
            ))}
          </AsTableBody>
        </AsTable>
      </Section>
      <HelpTile header={LABELS.HELP_TITLE} body={LABELS.HELP_TEXT}/>
    </ContentBody>
  </Page>;
}
