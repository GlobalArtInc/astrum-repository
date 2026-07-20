
import React from 'react';
import {useMachine} from '@xstate/react';

import {ListMachineUtils} from '@globalart/astrum-ui-plugin';
import {
  NxButton,
  NxFilterInput,
  NxTable,
  NxTableBody,
  NxTableCell,
  NxTableHead,
  NxTableRow,
  NxTooltip,
  NxTile,
} from '@globalart/react-shared-components';

import {faClock} from '@fortawesome/free-solid-svg-icons';

import {
  ContentBody,
  PageActions,
  PageHeader,
  PageTitle
} from '@globalart/astrum-ui-plugin';
import {HelpTile} from '@globalart/astrum-ui-plugin';

import Machine from './TasksListMachine';
import {canCreateTask} from './TasksHelper';

import UIStrings from '../../../../constants/UIStrings';

const {TASKS: {LIST: LABELS}} = UIStrings;
const {COLUMNS} = LABELS;

export default function TasksList({onCreate, onEdit}) {
  const [state, send] = useMachine(Machine, {devTools: true});
  const isLoading = state.matches('loading');
  const {data, error, filter: filterText} = state.context;

  const nameSortDir = ListMachineUtils.getSortDirection('name', state.context);
  const typeNameSortDir = ListMachineUtils.getSortDirection('typeName', state.context);
  const statusDescriptionSortDir = ListMachineUtils.getSortDirection('statusDescription', state.context);
  const scheduleSortDir = ListMachineUtils.getSortDirection('schedule', state.context);
  const nextRunSortDir = ListMachineUtils.getSortDirection('nextRun', state.context);
  const lastRunSortDir = ListMachineUtils.getSortDirection('lastRun', state.context);
  const lastRunResultSortDir = ListMachineUtils.getSortDirection('lastRunResult', state.context);

  const sortByName = () => send({type: 'SORT_BY_NAME'});
  const sortByTypeName = () => send({type: 'SORT_BY_TYPE_NAME'});
  const sortByStatusDescription = () => send({type: 'SORT_BY_STATUS_DESCRIPTION'});
  const sortBySchedule = () => send({type: 'SORT_BY_SCHEDULE'});
  const sortByNextRun = () => send({type: 'SORT_BY_NEXT_RUN'});
  const sortByLastRun = () => send({type: 'SORT_BY_LAST_RUN'});
  const sortByLastRunResult = () => send({type: 'SORT_BY_LAST_RUN_RESULT'});

  const filter = (value) => send({type: 'FILTER', filter: value});
  const canCreate = canCreateTask();

  function create() {
    if (canCreate) {
      onCreate();
    }
  }

  return <div className="as-tasks">
    <PageHeader>
      <PageTitle
          icon={UIStrings.TASKS.MENU.icon}
          text={UIStrings.TASKS.MENU.text}
          description={UIStrings.TASKS.MENU.description}
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
    <ContentBody className="as-tasks-list">
      <NxTile>
        <NxTile.Header>
          <NxTile.HeaderActions>
            <NxFilterInput
                id="filter"
                onChange={filter}
                value={filterText}
                placeholder={UIStrings.FILTER}
            />
          </NxTile.HeaderActions>
        </NxTile.Header>
        <NxTile.Content>
          <NxTable>
            <NxTableHead>
              <NxTableRow>
                <NxTableCell onClick={sortByName} isSortable sortDir={nameSortDir}>{COLUMNS.NAME}</NxTableCell>
                <NxTableCell onClick={sortByTypeName} isSortable sortDir={typeNameSortDir}>{COLUMNS.TYPE}</NxTableCell>
                <NxTableCell onClick={sortByStatusDescription} isSortable sortDir={statusDescriptionSortDir}>{COLUMNS.STATUS}</NxTableCell>
                <NxTableCell onClick={sortBySchedule} isSortable sortDir={scheduleSortDir}>{COLUMNS.SCHEDULE}</NxTableCell>
                <NxTableCell onClick={sortByNextRun} isSortable sortDir={nextRunSortDir}>{COLUMNS.NEXT_RUN}</NxTableCell>
                <NxTableCell onClick={sortByLastRun} isSortable sortDir={lastRunSortDir}>{COLUMNS.LAST_RUN}</NxTableCell>
                <NxTableCell onClick={sortByLastRunResult} isSortable sortDir={lastRunResultSortDir}>{COLUMNS.LAST_RESULT}</NxTableCell>
                <NxTableCell chevron/>
              </NxTableRow>
            </NxTableHead>
            <NxTableBody isLoading={isLoading} error={error} emptyMessage={LABELS.EMPTY_LIST}>
              {data.map((task) => (
                  <NxTableRow key={task.id} onClick={() => onEdit(encodeURIComponent(task.id))} isClickable>
                    <NxTableCell>{task.name}</NxTableCell>
                    <NxTableCell>{task.typeName}</NxTableCell>
                    <NxTableCell>{task.statusDescription}</NxTableCell>
                    <NxTableCell className="capitalize-schedule">{task.schedule}</NxTableCell>
                    <NxTableCell>{task.nextRun}</NxTableCell>
                    <NxTableCell>{task.lastRun}</NxTableCell>
                    <NxTableCell>{task.lastRunResult}</NxTableCell>
                    <NxTableCell chevron/>
                  </NxTableRow>
              ))}
            </NxTableBody>
          </NxTable>
        </NxTile.Content>
      </NxTile>
      <HelpTile header={LABELS.HELP.TITLE} body={LABELS.HELP.TEXT}/>
    </ContentBody>
  </div>;
}
