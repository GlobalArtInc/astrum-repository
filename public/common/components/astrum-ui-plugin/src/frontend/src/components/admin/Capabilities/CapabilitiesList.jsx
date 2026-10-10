
import React, {} from 'react';
import { useMachine } from '@xstate/react';
import { useRouter } from '@uirouter/react';
import classNames from 'classnames';

import {
  AsButton,
  AsFilterInput,
  AsTable,
  AsTableBody,
  AsTableCell,
  AsTableHead,
  AsTableRow,
  AsFontAwesomeIcon
} from '@globalart/astrum-react-shared-components';

import {
  faServer,
  faCheckCircle,
  faExclamationCircle,
  faMinusCircle
} from '@fortawesome/free-solid-svg-icons';

import { ContentBody, Page, PageActions, PageHeader, PageTitle, Section, SectionToolbar } from '../../layout';

import CapabilitiesListMachine from './CapabilitiesListMachine';
import UIStrings from '../../../constants/UIStrings';
import { AdminRouteNames } from '../../../constants/admin/AdminRouteNames';
import ListMachineUtils from '../../../interface/ListMachineUtils';
import './CapabilitiesList.scss';

import ExtJS from '../../../interface/ExtJS';
import Permissions from '../../../constants/Permissions';
import { RouteNames } from '../../../constants/RouteNames';

const ADMIN = RouteNames.ADMIN;

const { CAPABILITIES } = UIStrings;
const { COLUMNS } = CAPABILITIES.LIST;

export default function CapabilitiesList() {
  const router = useRouter();
  const onEdit = id => router.stateService.go(AdminRouteNames.SYSTEM.CAPABILITIES.EDIT, { id });
  const onCreate = () => router.stateService.go(ADMIN.SYSTEM.CAPABILITIES.CREATE);
  const canCreate = ExtJS.checkPermission(Permissions.CAPABILITIES.CREATE);

  const [current, send] = useMachine(CapabilitiesListMachine, { devTools: true });
  const isLoading = current.matches('loading');
  const { data, error, filter, sortField, sortDirection } = current.context;
  const retryHandler = () =>
      send({ type: 'RETRY' });

  const filterHandler = (value) =>
      send({ type: 'FILTER', filter: value });

  const sortByTypeName = () => send({ type: 'SORT_BY_TYPE_NAME' });
  const sortByState = () => send({ type: 'SORT_BY_STATE' });
  const sortByCategory = () => send({ type: 'SORT_BY_CATEGORY' });
  const sortByRepository = () => send({ type: 'SORT_BY_REPOSITORY' });
  const sortByDescription = () => send({ type: 'SORT_BY_DESCRIPTION' });
  const sortByNotes = () => send({ type: 'SORT_BY_NOTES' });

  const showsRepositoryHeader = !!data ?
      !!data.find(entry => entry.tags?.Repository)
      : false;

  return (
    <Page className='as-capabilities'>
      <PageHeader>
        <PageTitle icon={faServer} {...CAPABILITIES.MENU} />
        <PageActions>
          <AsButton data-analytics-id="as-create-capability" variant="primary" onClick={onCreate} disabled={!canCreate}>{CAPABILITIES.LIST.CREATE_BUTTON}</AsButton>
        </PageActions>
      </PageHeader>
      <ContentBody className='as-capabilities-list'>
        <Section>
          <SectionToolbar>
            <div className="as-spacer"/>
            <AsFilterInput placeholder={UIStrings.FILTER} value={filter} onChange={filterHandler} />
          </SectionToolbar>
          <AsTable>
            <AsTableHead>
              <AsTableRow>
                <AsTableCell
                  isSortable
                  sortDir={ListMachineUtils.getSortDirection('typeName', { sortField, sortDirection })}
                  onClick={sortByTypeName}
                >
                  {COLUMNS.TYPE}
                </AsTableCell>
                <AsTableCell
                  isSortable
                  sortDir={ListMachineUtils.getSortDirection('state', { sortField, sortDirection })}
                  onClick={sortByState}
                >
                  {COLUMNS.STATE}
                </AsTableCell>
                <AsTableCell
                  isSortable
                  sortDir={ListMachineUtils.getSortDirection('category', { sortField, sortDirection })}
                  onClick={sortByCategory}
                >
                  {COLUMNS.CATEGORY}
                </AsTableCell>
                { showsRepositoryHeader &&
                  <AsTableCell
                    isSortable
                    sortDir={ListMachineUtils.getSortDirection('repository', { sortField, sortDirection })}
                    onClick={sortByRepository}
                  >
                    {COLUMNS.REPOSITORY}
                  </AsTableCell>
                }
                <AsTableCell
                  isSortable
                  sortDir={ListMachineUtils.getSortDirection('description', { sortField, sortDirection })}
                  onClick={sortByDescription}
                >
                  {COLUMNS.DESCRIPTION}
                </AsTableCell>
                <AsTableCell
                  isSortable
                  sortDir={ListMachineUtils.getSortDirection('notes', { sortField, sortDirection })}
                  onClick={sortByNotes}
                >
                  {COLUMNS.NOTES}
                </AsTableCell>
                <AsTableCell chevron />
              </AsTableRow>
            </AsTableHead>
            <AsTableBody
              isLoading={isLoading}
              error={error}
              emptyMessage={CAPABILITIES.LIST.EMPTY_LIST}
              retryHandler={retryHandler}
            >
              {data.map(({ id, state, typeName, description, notes, tags }) => (
                <AsTableRow isClickable key={id} onClick={() => onEdit(id)}>
                  <AsTableCell>{typeName}</AsTableCell>
                  <StateCell state={state} />
                  <AsTableCell>{tagsToCategoryDisplayText(tags)}</AsTableCell>
                  { showsRepositoryHeader && <AsTableCell>{propertiesToRepositoryDisplayText(tags)}</AsTableCell>}
                  <AsTableCell>{toDisplayText(description)}</AsTableCell>
                  <AsTableCell>{toDisplayText(notes)}</AsTableCell>
                  <AsTableCell chevron />
                </AsTableRow>
              ))}
            </AsTableBody>
          </AsTable>
        </Section>
      </ContentBody>
    </Page>
  );

  function tagsToCategoryDisplayText(tags) {
    return toDisplayText(tags?.Category)
  }

  function propertiesToRepositoryDisplayText(tags) {
    return toDisplayText(tags?.Repository)
  }

  function toDisplayText(text) {
    return text || '-';
  }

}

function StateCell({state}) {
  let icon;

  switch (state) {
    case "active":
      icon = faCheckCircle;
      break;
    case "error":
    case "passive":
      icon = faExclamationCircle;
      break;
    case "disabled":
      icon = faMinusCircle;
      break;
    default:
      icon = null;
  }

  return (
      <AsTableCell className={classNames("capabilities-state", "capabilities-state__" + state)}>
        {icon && <AsFontAwesomeIcon icon={icon} />}
        {state}
      </AsTableCell>
  );
}
