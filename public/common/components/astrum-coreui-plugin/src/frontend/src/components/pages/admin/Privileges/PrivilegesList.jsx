
import React, { useCallback } from 'react';
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
  AsPagination,
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

import PrivilegesListMachine from './PrivilegesListMachine';
import UIStrings from '../../../../constants/UIStrings';
import { useRouter } from '@uirouter/react';
import { ROUTE_NAMES } from '../../../../routerConfig/routeNames/routeNames';

const {PRIVILEGES: {LIST: LABELS}} = UIStrings;
const {COLUMNS} = LABELS;

export default function PrivilegesList() {
  const router = useRouter();
  const onEdit = useCallback(itemId => router.stateService.go(ROUTE_NAMES.ADMIN.SECURITY.PRIVILEGES.EDIT, { itemId }));
  const onCreate = useCallback(() => router.stateService.go(ROUTE_NAMES.ADMIN.SECURITY.PRIVILEGES.CREATE));
  
  const [current, send] = useMachine(PrivilegesListMachine, {devTools: true});
  const isLoading = current.matches('loading');
  const { data, error, filter: filterText, currentPage, pageSize, totalCount } = current.context;

  const nameSortDir = ListMachineUtils.getSortDirection('name', current.context);
  const descriptionSortDir = ListMachineUtils.getSortDirection('description', current.context);
  const typeSortDir = ListMachineUtils.getSortDirection('type', current.context);
  const permissionSortDir = ListMachineUtils.getSortDirection('permission', current.context);

  const sortByName = () => send({type: 'SORT_BY_NAME'});
  const sortByDescription = () => send({type: 'SORT_BY_DESCRIPTION'});
  const sortByType = () => send({type: 'SORT_BY_TYPE'});
  const sortByPermission = () => send({type: 'SORT_BY_PERMISSION'});

  const filter = (value) => send({type: 'FILTER', filter: value});
  const canCreate = ExtJS.checkPermission('astrum:privileges:create');

  // Pagination handlers
  const pageCount = Math.ceil(totalCount / pageSize);
  const changePage = page => send({ type: 'CHANGE_PAGE', page });

  function create() {
    if (canCreate) {
      onCreate();
    }
  }

  return <Page className="as-privileges">
    <PageHeader>
      <PageTitle
          icon={UIStrings.PRIVILEGES.MENU.icon}
          text={UIStrings.PRIVILEGES.MENU.text}
          description={UIStrings.PRIVILEGES.MENU.description}
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
    <ContentBody className="as-privileges-list">
      <HelpTile header={LABELS.HELP.TITLE} body={LABELS.HELP.TEXT}/>
      <Section>
        <SectionToolbar>
          <div className="as-spacer"/>
          <AsFilterInput
              id="filter"
              onChange={filter}
              value={filterText}
              placeholder={UIStrings.FILTER}
          />
        </SectionToolbar>
        <AsTable>
          <AsTableHead>
            <AsTableRow>
              <AsTableCell onClick={sortByName} isSortable sortDir={nameSortDir}>{COLUMNS.NAME}</AsTableCell>
              <AsTableCell onClick={sortByDescription} isSortable sortDir={descriptionSortDir}>{COLUMNS.DESCRIPTION}</AsTableCell>
              <AsTableCell onClick={sortByType} isSortable sortDir={typeSortDir}>{COLUMNS.TYPE}</AsTableCell>
              <AsTableCell onClick={sortByPermission} isSortable sortDir={permissionSortDir}>{COLUMNS.PERMISSION}</AsTableCell>
              <AsTableCell chevron/>
            </AsTableRow>
          </AsTableHead>
          <AsTableBody isLoading={isLoading} error={error} emptyMessage={LABELS.EMPTY_LIST}>
            {data.map(({id, name, description, type, permission}) => (
                <AsTableRow key={id} onClick={() => onEdit(id)} isClickable>
                  <AsTableCell>{name}</AsTableCell>
                  <AsTableCell>{description}</AsTableCell>
                  <AsTableCell>{type}</AsTableCell>
                  <AsTableCell>{permission}</AsTableCell>
                  <AsTableCell chevron/>
                </AsTableRow>
            ))}
          </AsTableBody>
        </AsTable>
        {pageCount > 1 && !isLoading && (
          <div className='astrum-pagination'>
            <AsPagination onChange={changePage} pageCount={pageCount} currentPage={currentPage} />
          </div>
        )}
      </Section>
    </ContentBody>
  </Page>;
}
