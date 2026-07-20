
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
  NxButton,
  NxFilterInput,
  NxTable,
  NxTableBody,
  NxTableCell,
  NxTableHead,
  NxTableRow,
  NxTooltip,
  NxInfoAlert,
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
        <NxTooltip
            title={!canCreate && UIStrings.PERMISSION_ERROR}
            placement="bottom"
        >
          <NxButton
              type="button"
              variant="primary"
              className={!canCreate && 'disabled'}
              onClick={create}
          >
            {LABELS.CREATE_BUTTON}
          </NxButton>
        </NxTooltip>
      </PageActions>
    </PageHeader>
    <ContentBody className="as-roles-list">
      {roleId && roleName &&
          <NxInfoAlert>
            {LABELS.ALERT.DEFAULT_ROLE(roleId, roleName)}
            <br/>
            {capabilityId && LABELS.ALERT.CAPABILITY(capabilityId)}
          </NxInfoAlert>
      }
      <Section>
        <SectionToolbar>
          <div className="as-spacer"/>
          <NxFilterInput
              id="filter"
              onChange={filter}
              value={filterText}
              placeholder={UIStrings.FILTER}/>
        </SectionToolbar>
        <NxTable>
          <NxTableHead>
            <NxTableRow>
              <NxTableCell onClick={sortById} isSortable sortDir={idSortDir}>{COLUMNS.ID}</NxTableCell>
              <NxTableCell onClick={sortByName} isSortable sortDir={nameSortDir}>{COLUMNS.NAME}</NxTableCell>
              <NxTableCell onClick={sortByDescription} isSortable sortDir={descriptionSortDir}>{COLUMNS.DESCRIPTION}</NxTableCell>
              <NxTableCell chevron/>
            </NxTableRow>
          </NxTableHead>
          <NxTableBody isLoading={isLoading} error={error} emptyMessage={LABELS.EMPTY_LIST}>
            {data.map(({id, name, description}) => (
                <NxTableRow key={id} onClick={() => onEdit(encodeURIComponent(id))} isClickable>
                  <NxTableCell>{id}</NxTableCell>
                  <NxTableCell>{name}</NxTableCell>
                  <NxTableCell>{description}</NxTableCell>
                  <NxTableCell chevron/>
                </NxTableRow>
            ))}
          </NxTableBody>
        </NxTable>
      </Section>
      <HelpTile header={LABELS.HELP.TITLE} body={LABELS.HELP.TEXT}/>
    </ContentBody>
  </Page>;
}
