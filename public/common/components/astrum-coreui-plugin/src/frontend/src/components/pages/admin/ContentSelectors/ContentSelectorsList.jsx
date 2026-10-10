
import React from 'react';
import {useMachine} from '@xstate/react';

import {faScroll} from '@fortawesome/free-solid-svg-icons';

import {ExtJS, ListMachineUtils} from '@globalart/astrum-ui-plugin';
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
  Page,
} from '@globalart/astrum-ui-plugin';
import {HelpTile} from '@globalart/astrum-ui-plugin';

import ContentSelectorsListMachine from './ContentSelectorsListMachine';

import UIStrings from '../../../../constants/UIStrings';
import {useRouter} from '@uirouter/react';
import {ROUTE_NAMES} from '../../../../routerConfig/routeNames/routeNames';

const ADMIN = ROUTE_NAMES.ADMIN;
const {CONTENT_SELECTORS: LABELS} = UIStrings;

export default function ContentSelectorsList() {
  const router = useRouter();
  const onEdit = (itemId) => router.stateService.go(ADMIN.REPOSITORY.SELECTORS.EDIT, {itemId});
  const onCreate = () => router.stateService.go(ADMIN.REPOSITORY.SELECTORS.CREATE);

  const [state, send] = useMachine(ContentSelectorsListMachine, {devTools: true});
  const isLoading = state.matches('loading');
  const data = state.context.data;
  const filterText = state.context.filter;
  const error = state.context.error;

  const nameSortDir = ListMachineUtils.getSortDirection('name', state.context);
  const typeSortDir = ListMachineUtils.getSortDirection('type', state.context);
  const descriptionSortDir = ListMachineUtils.getSortDirection('description', state.context);

  function filter(value) {
    send({type: 'FILTER', filter: value});
  }

  const canCreate = ExtJS.checkPermission('astrum:selectors:create');

  return <Page className="as-content-selectors">
    <PageHeader>
      <PageTitle icon={faScroll} {...LABELS.MENU}/>
      <PageActions>
        <AsButton variant="primary" disabled={!canCreate} onClick={onCreate} data-analytics-id="as-content-selector-create">
          <span>{LABELS.CREATE_BUTTON}</span>
        </AsButton>
      </PageActions>
    </PageHeader>
    <ContentBody>
      <Section className="as-content-selectors-list">
        <SectionToolbar>
          <div className="as-spacer" />
          <AsFilterInput
              id="filter"
              inputAttributes={{
                'aria-label': 'Filter content selectors by name',
                'data-analytics-id': 'as-content-selector-filter'
              }}
              onChange={filter}
              value={filterText}
              placeholder={LABELS.FILTER_PLACEHOLDER}/>
        </SectionToolbar>
        <AsTable>
          <AsTableHead>
            <AsTableRow>
              <AsTableCell onClick={() => send({type: 'SORT_BY_NAME'})} isSortable sortDir={nameSortDir}>
                {LABELS.NAME_LABEL}
              </AsTableCell>
              <AsTableCell onClick={() => send({type: 'SORT_BY_TYPE'})} isSortable sortDir={typeSortDir}>
                {LABELS.TYPE_LABEL}
              </AsTableCell>
              <AsTableCell onClick={() => send({type: 'SORT_BY_DESCRIPTION'})} isSortable sortDir={descriptionSortDir}>
                {LABELS.DESCRIPTION_LABEL}
              </AsTableCell>
              <AsTableCell chevron/>
            </AsTableRow>
          </AsTableHead>
          <AsTableBody isLoading={isLoading} error={error} emptyMessage={LABELS.EMPTY_MESSAGE}>
            {data.map(({name, type, description}) => (
                <AsTableRow key={name} onClick={() => onEdit(name)} isClickable>
                  <AsTableCell>{name}</AsTableCell>
                  <AsTableCell>{type?.toUpperCase()}</AsTableCell>
                  <AsTableCell>{description}</AsTableCell>
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
