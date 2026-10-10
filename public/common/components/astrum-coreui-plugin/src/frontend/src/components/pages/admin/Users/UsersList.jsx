
import React from 'react';
import {useMachine} from '@xstate/react';

import {ExtJS, ListMachineUtils} from '@globalart/astrum-ui-plugin';
import {
  AsButton,
  AsFilterInput,
  AsTable,
  AsTableBody,
  AsTableCell,
  AsTableHead,
  AsTableRow,
  AsTooltip,
  AsFormSelect,
} from '@globalart/react-shared-components';

import {faUsers} from '@fortawesome/free-solid-svg-icons';

import {
  ContentBody,
  PageActions,
  PageHeader,
  PageTitle,
  Section
} from '@globalart/astrum-ui-plugin';
import {HelpTile} from '@globalart/astrum-ui-plugin';

import './Users.scss';

import Machine from './UsersListMachine';
import UIStrings from '../../../../constants/UIStrings';

const {USERS: {LIST: LABELS}} = UIStrings;
const {COLUMNS} = LABELS;

export default function UsersList({onCreate, onEdit, initialSourceFilter = 'default', onSourceFilterChange}) {
  const [current, send] = useMachine(Machine, {
    devTools: true,
    context: {sourceFilter: initialSourceFilter},
  });
  const isLoading = current.matches('loading') || current.matches('loadingSources');
  const {data, sources = {}, error, filter, sourceFilter} = current.context;

  const userIdSortDir = ListMachineUtils.getSortDirection('userId', current.context);
  const realmSortDir = ListMachineUtils.getSortDirection('realm', current.context);
  const firstNameSortDir = ListMachineUtils.getSortDirection('firstName', current.context);
  const lastNameSortDir = ListMachineUtils.getSortDirection('lastName', current.context);
  const emailSortDir = ListMachineUtils.getSortDirection('email', current.context);
  const statusSortDir = ListMachineUtils.getSortDirection('status', current.context);

  const sortById = () => send({type: 'SORT_BY_USER_ID'});
  const sortByRealms = () => send({type: 'SORT_BY_REALM'});
  const sortByFirstName = () => send({type: 'SORT_BY_FIRST_NAME'});
  const sortByLastName = () => send({type: 'SORT_BY_LAST_NAME'});
  const sortByEmail = () => send({type: 'SORT_BY_EMAIL'});
  const sortByStatus = () => send({type: 'SORT_BY_STATUS'});

  const onUserIdFilterChange = (value) => send({type: 'FILTER', filter: value});
  const handleSourceFilterChange = (value) => {
    send({type: 'FILTER_BY_SOURCE', filter: value});
    onSourceFilterChange?.(value);
  };
  const canCreate = ExtJS.checkPermission('astrum:users:create');

  const create = () => {
    if (canCreate) {
      onCreate();
    }
  };

  const onClickRow = (realm, userId) => onEdit(`${encodeURIComponent(realm)}/${encodeURIComponent(userId)}`);

  return <div className="as-users">
    <PageHeader>
      <PageTitle
          icon={UIStrings.USERS.MENU.icon}
          text={UIStrings.USERS.MENU.text}
          description={UIStrings.USERS.MENU.description}
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
    <ContentBody className="as-users-list">
      <Section>
        <AsTable>
          <AsTableHead>
            <AsTableRow>
              <AsTableCell onClick={sortById} isSortable sortDir={userIdSortDir}>{COLUMNS.USER_ID}</AsTableCell>
              <AsTableCell onClick={sortByRealms} isSortable sortDir={realmSortDir}>{COLUMNS.REALM}</AsTableCell>
              <AsTableCell onClick={sortByFirstName} isSortable sortDir={firstNameSortDir}>{COLUMNS.FIRST_NAME}</AsTableCell>
              <AsTableCell onClick={sortByLastName} isSortable sortDir={lastNameSortDir}>{COLUMNS.LAST_NAME}</AsTableCell>
              <AsTableCell onClick={sortByEmail} isSortable sortDir={emailSortDir}>{COLUMNS.EMAIL}</AsTableCell>
              <AsTableCell onClick={sortByStatus} isSortable sortDir={statusSortDir}>{COLUMNS.STATUS}</AsTableCell>
              <AsTableCell chevron/>
            </AsTableRow>
            <AsTableRow isFilterHeader>
              <AsTableCell>
                <AsFilterInput
                    id="userIdFilter"
                    onChange={onUserIdFilterChange}
                    value={filter}
                    placeholder={LABELS.FILTER_PLACEHOLDER}
                />
              </AsTableCell>
              <AsTableCell>
                <AsFormSelect
                    id="userSourceFilter"
                    name="userSourceFilter"
                    value={sourceFilter}
                    onChange={handleSourceFilterChange}
                >
                  {Object.values(sources).map(({id, name}) => (
                      <option key={id} value={id}>{name}</option>
                  ))}
                </AsFormSelect>
              </AsTableCell>
              <AsTableCell />
              <AsTableCell />
              <AsTableCell />
              <AsTableCell />
              <AsTableCell />
            </AsTableRow>
          </AsTableHead>
          <AsTableBody isLoading={isLoading} error={error} emptyMessage={LABELS.EMPTY_LIST}>
            {data.map(({userId, realm, firstName, lastName, email, status}) => (
                <AsTableRow key={userId} onClick={() => onClickRow(realm, userId)} isClickable>
                  <AsTableCell>{userId}</AsTableCell>
                  <AsTableCell>{sources[realm].name}</AsTableCell>
                  <AsTableCell>{firstName}</AsTableCell>
                  <AsTableCell>{lastName}</AsTableCell>
                  <AsTableCell>{email}</AsTableCell>
                  <AsTableCell className="capitalize-status">{status}</AsTableCell>
                  <AsTableCell chevron/>
                </AsTableRow>
            ))}
          </AsTableBody>
        </AsTable>
      </Section>
      <HelpTile header={LABELS.HELP.TITLE} body={LABELS.HELP.TEXT}/>
    </ContentBody>
  </div>;
}
