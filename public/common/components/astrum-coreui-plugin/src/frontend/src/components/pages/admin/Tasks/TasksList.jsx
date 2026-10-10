
import React from 'react';
import {useMachine} from '@xstate/react';

import {ListMachineUtils} from '@globalart/astrum-ui-plugin';
import {
  AsButton,
  AsFilterInput,
  AsTable,
  AsTableBody,
  AsTableCell,
  AsTableHead,
  AsTableRow,
  AsTooltip,
  AsTile,
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
    <ContentBody className="as-tasks-list">
      <AsTile>
        <AsTile.Header>
          <AsTile.HeaderActions>
            <AsFilterInput
                id="filter"
                onChange={filter}
                value={filterText}
                placeholder={UIStrings.FILTER}
            />
          </AsTile.HeaderActions>
        </AsTile.Header>
        <AsTile.Content>
          <AsTable>
            <AsTableHead>
              <AsTableRow>
                <AsTableCell onClick={sortByName} isSortable sortDir={nameSortDir}>{COLUMNS.NAME}</AsTableCell>
                <AsTableCell onClick={sortByTypeName} isSortable sortDir={typeNameSortDir}>{COLUMNS.TYPE}</AsTableCell>
                <AsTableCell onClick={sortByStatusDescription} isSortable sortDir={statusDescriptionSortDir}>{COLUMNS.STATUS}</AsTableCell>
                <AsTableCell onClick={sortBySchedule} isSortable sortDir={scheduleSortDir}>{COLUMNS.SCHEDULE}</AsTableCell>
                <AsTableCell onClick={sortByNextRun} isSortable sortDir={nextRunSortDir}>{COLUMNS.NEXT_RUN}</AsTableCell>
                <AsTableCell onClick={sortByLastRun} isSortable sortDir={lastRunSortDir}>{COLUMNS.LAST_RUN}</AsTableCell>
                <AsTableCell onClick={sortByLastRunResult} isSortable sortDir={lastRunResultSortDir}>{COLUMNS.LAST_RESULT}</AsTableCell>
                <AsTableCell chevron/>
              </AsTableRow>
            </AsTableHead>
            <AsTableBody isLoading={isLoading} error={error} emptyMessage={LABELS.EMPTY_LIST}>
              {data.map((task) => (
                  <AsTableRow key={task.id} onClick={() => onEdit(encodeURIComponent(task.id))} isClickable>
                    <AsTableCell>{task.name}</AsTableCell>
                    <AsTableCell>{task.typeName}</AsTableCell>
                    <AsTableCell>{task.statusDescription}</AsTableCell>
                    <AsTableCell className="capitalize-schedule">{task.schedule}</AsTableCell>
                    <AsTableCell>{task.nextRun}</AsTableCell>
                    <AsTableCell>{task.lastRun}</AsTableCell>
                    <AsTableCell>{task.lastRunResult}</AsTableCell>
                    <AsTableCell chevron/>
                  </AsTableRow>
              ))}
            </AsTableBody>
          </AsTable>
        </AsTile.Content>
      </AsTile>
      <HelpTile header={LABELS.HELP.TITLE} body={LABELS.HELP.TEXT}/>
    </ContentBody>
  </div>;
}
