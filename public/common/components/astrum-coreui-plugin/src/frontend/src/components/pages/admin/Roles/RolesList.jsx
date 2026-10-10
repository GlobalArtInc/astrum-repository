
import React from 'react';
import {useMachine} from '@xstate/react';

import {
  ContentBody,
  ExtJS,
  ListMachineUtils,
  PageActions,
  PageHeader,
  PageTitle,
  Permissions,
  Section,
  SectionToolbar,
  Page,
} from '@globalart/astrum-ui-plugin';
import {
  AsButton,
  AsFilterInput,
  AsTable,
  AsTableBody,
  AsTableCell,
  AsTableHead,
  AsTableRow,
  AsTooltip,
  AsInfoAlert,
} from '@globalart/react-shared-components';

import {HelpTile} from '@globalart/astrum-ui-plugin';

import RolesListMachine from './RolesListMachine';
import UIStrings from '../../../../constants/UIStrings';

import {useRouter} from '@uirouter/react';
import {ROUTE_NAMES} from '../../../../routerConfig/routeNames/routeNames';

const ADMIN = ROUTE_NAMES.ADMIN;

const {ROLES: {LIST: LABELS}} = UIStrings;
const {COLUMNS} = LABELS;

export default function RolesList() {
  const router = useRouter();
  const onEdit = (itemId) => router.stateService.go(ADMIN.SECURITY.ROLES.EDIT, {itemId});
  const onCreate = () => router.stateService.go(ADMIN.SECURITY.ROLES.CREATE);

  const [state, send] = useMachine(RolesListMachine, {devTools: true});
  const isLoading = state.matches('loading');
  const {data, error, filter: filterText, defaultRole: {roleId, roleName, capabilityId} = {}} = state.context;

  const idSortDir = ListMachineUtils.getSortDirection('id', state.context);
  const nameSortDir = ListMachineUtils.getSortDirection('name', state.context);
  const descriptionSortDir = ListMachineUtils.getSortDirection('description', state.context);

  const sortById = () => send({type: 'SORT_BY_ID'});
  const sortByName = () => send({type: 'SORT_BY_NAME'});
  const sortByDescription = () => send({type: 'SORT_BY_DESCRIPTION'});

  const filter = (value) => send({type: 'FILTER', filter: value});
  const canCreate = ExtJS.checkPermission(Permissions.ROLES.CREATE);

  function create() {
    if (canCreate) {
      onCreate();
    }
  }

  return <Page className="as-roles">
    <PageHeader>
      <PageTitle
          icon={UIStrings.ROLES.MENU.icon}
          text={UIStrings.ROLES.MENU.text}
          description={UIStrings.ROLES.MENU.description}
      />
      <PageActions>
        <AsTooltip
            title={!canCreate && UIStrings.PERMISSION_ERROR}
            placement="bottom"
        >
          <AsButton
              type="button"
              variant="primary"
              className={!canCreate && 'disabled'}
              onClick={create}
          >
            {LABELS.CREATE_BUTTON}
          </AsButton>
        </AsTooltip>
      </PageActions>
    </PageHeader>
    <ContentBody className="as-roles-list">
      {roleId && roleName &&
          <AsInfoAlert>
            {LABELS.ALERT.DEFAULT_ROLE(roleId, roleName)}
            <br/>
            {capabilityId && LABELS.ALERT.CAPABILITY(capabilityId)}
          </AsInfoAlert>
      }
      <Section>
        <SectionToolbar>
          <div className="as-spacer"/>
          <AsFilterInput
              id="filter"
              onChange={filter}
              value={filterText}
              placeholder={UIStrings.FILTER}/>
        </SectionToolbar>
        <AsTable>
          <AsTableHead>
            <AsTableRow>
              <AsTableCell onClick={sortById} isSortable sortDir={idSortDir}>{COLUMNS.ID}</AsTableCell>
              <AsTableCell onClick={sortByName} isSortable sortDir={nameSortDir}>{COLUMNS.NAME}</AsTableCell>
              <AsTableCell onClick={sortByDescription} isSortable sortDir={descriptionSortDir}>{COLUMNS.DESCRIPTION}</AsTableCell>
              <AsTableCell chevron/>
            </AsTableRow>
          </AsTableHead>
          <AsTableBody isLoading={isLoading} error={error} emptyMessage={LABELS.EMPTY_LIST}>
            {data.map(({id, name, description}) => (
                <AsTableRow key={id} onClick={() => onEdit(encodeURIComponent(id))} isClickable>
                  <AsTableCell>{id}</AsTableCell>
                  <AsTableCell>{name}</AsTableCell>
                  <AsTableCell>{description}</AsTableCell>
                  <AsTableCell chevron/>
                </AsTableRow>
            ))}
          </AsTableBody>
        </AsTable>
      </Section>
      <HelpTile header={LABELS.HELP.TITLE} body={LABELS.HELP.TEXT}/>
    </ContentBody>
  </Page>;
}
