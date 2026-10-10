
import React from 'react';
import {useMachine} from '@xstate/react';

import {faBroom} from '@fortawesome/free-solid-svg-icons';

import ListMachineUtils from '../../../interface/ListMachineUtils';

import {
  AsButton,
  AsFilterInput,
  AsTable,
  AsTableBody,
  AsTableCell,
  AsTableHead,
  AsTableRow,
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
        <AsButton variant="primary" onClick={onCreate}>
          <span>{LABELS.CREATE_BUTTON}</span>
        </AsButton>
      </PageActions>
    </PageHeader>
    <ContentBody>
      <Section className="as-cleanup-policies-list">
        <SectionToolbar>
          <div className="as-spacer" />
          <AsFilterInput
              id="filter"
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
              <AsTableCell onClick={() => send({type: 'SORT_BY_FORMAT'})} isSortable sortDir={formatSortDir}>
                {LABELS.FORMAT_LABEL}
              </AsTableCell>
              <AsTableCell onClick={() => send({type: 'SORT_BY_NOTES'})} isSortable sortDir={notesSortDir}>
                {LABELS.DESCRIPTION_LABEL}
              </AsTableCell>
              <AsTableCell chevron/>
            </AsTableRow>
          </AsTableHead>
          <AsTableBody isLoading={isLoading} error={error} emptyMessage={LABELS.EMPTY_MESSAGE}>
            {data.map(({name, format, notes}) => (
                <AsTableRow key={name} onClick={() => onEdit(name)} isClickable>
                  <AsTableCell>{name}</AsTableCell>
                  <AsTableCell>{format}</AsTableCell>
                  <AsTableCell>{notes}</AsTableCell>
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
