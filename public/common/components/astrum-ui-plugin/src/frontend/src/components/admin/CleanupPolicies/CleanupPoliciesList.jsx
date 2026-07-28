
import React from 'react';
import {useMachine} from '@xstate/react';

import {faBroom} from '@fortawesome/free-solid-svg-icons';

import ListMachineUtils from '../../../interface/ListMachineUtils';

import {
  NxButton,
  NxFilterInput,
  NxTable,
  NxTableBody,
  NxTableCell,
  NxTableHead,
  NxTableRow,
} from '@globalart/react-shared-components';

import { Page } from '../../layout';

import {
  ContentBody,
  PageActions,
  PageHeader,
  PageTitle,
  Section,
  SectionToolbar
} from '../../layout';
import HelpTile from '../../widgets/HelpTile/HelpTile';

import CleanupPoliciesListMachine from './CleanupPoliciesListMachine';

import UIStrings from '../../../constants/UIStrings';
import {RouteNames} from '../../../constants/RouteNames';

import { useRouter } from '@uirouter/react';

const {CLEANUP_POLICIES: LABELS} = UIStrings;
const CLEANUP_POLICIES_ROUTES = RouteNames.ADMIN.REPOSITORY.CLEANUPPOLICIES;

export default function CleanupPoliciesList() {
  const router = useRouter();
  const onEdit = (itemId) => router.stateService.go(CLEANUP_POLICIES_ROUTES.EDIT, {itemId});
  const onCreate = () => router.stateService.go(CLEANUP_POLICIES_ROUTES.CREATE);
  const [current, send] = useMachine(CleanupPoliciesListMachine, {devTools: true});
  const isLoading = current.matches('loading');
  const data = current.context.data;
  const filterText = current.context.filter;
  const error = current.context.error;

  const nameSortDir = ListMachineUtils.getSortDirection('name', current.context);
  const formatSortDir = ListMachineUtils.getSortDirection('format', current.context);
  const notesSortDir = ListMachineUtils.getSortDirection('notes', current.context);

  function filter(value) {
    send({type: 'FILTER', filter: value});
  }

  return <Page className="as-cleanup-policies">
    <PageHeader>
      <PageTitle icon={faBroom} {...LABELS.MENU}/>
      <PageActions>
        <NxButton variant="primary" onClick={onCreate}>
          <span>{LABELS.CREATE_BUTTON}</span>
        </NxButton>
      </PageActions>
    </PageHeader>
    <ContentBody>
      <Section className="as-cleanup-policies-list">
        <SectionToolbar>
          <div className="as-spacer" />
          <NxFilterInput
              id="filter"
              onChange={filter}
              value={filterText}
              placeholder={LABELS.FILTER_PLACEHOLDER}/>
        </SectionToolbar>
        <NxTable>
          <NxTableHead>
            <NxTableRow>
              <NxTableCell onClick={() => send({type: 'SORT_BY_NAME'})} isSortable sortDir={nameSortDir}>
                {LABELS.NAME_LABEL}
              </NxTableCell>
              <NxTableCell onClick={() => send({type: 'SORT_BY_FORMAT'})} isSortable sortDir={formatSortDir}>
                {LABELS.FORMAT_LABEL}
              </NxTableCell>
              <NxTableCell onClick={() => send({type: 'SORT_BY_NOTES'})} isSortable sortDir={notesSortDir}>
                {LABELS.DESCRIPTION_LABEL}
              </NxTableCell>
              <NxTableCell chevron/>
            </NxTableRow>
          </NxTableHead>
          <NxTableBody isLoading={isLoading} error={error} emptyMessage={LABELS.EMPTY_MESSAGE}>
            {data.map(({name, format, notes}) => (
                <NxTableRow key={name} onClick={() => onEdit(name)} isClickable>
                  <NxTableCell>{name}</NxTableCell>
                  <NxTableCell>{format}</NxTableCell>
                  <NxTableCell>{notes}</NxTableCell>
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
