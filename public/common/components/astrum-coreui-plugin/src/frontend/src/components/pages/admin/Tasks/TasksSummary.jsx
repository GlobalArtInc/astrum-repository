
import React from 'react';

import {
  AsTile,
  AsPageTitle,
  AsFontAwesomeIcon,
  AsButton,
  AsButtonBar,
  AsH2,
  AsTooltip,
  AsGrid,
  AsReadOnly,
} from '@globalart/astrum-react-shared-components';
import {
  ReadOnlyField,
} from '@globalart/astrum-ui-plugin';

import {faTrash, faStop, faPlay} from '@fortawesome/free-solid-svg-icons';
import UIStrings from '../../../../constants/UIStrings';
import {canDeleteTask, canRunTask, canStopTask} from './TasksHelper';

const {TASKS: {SUMMARY: LABELS, FORM}} = UIStrings;

export default function TasksSummary({task, runnable, stoppable, onDelete, onRun, onStop}) {
  const canDelete = canDeleteTask();
  const canRun = canRunTask();
  const canStop = canStopTask();

  const deleteTask = () => {
    if (canDelete) {
      onDelete();
    }
  };

  const runTask = () => {
    if (canRun) {
      onRun();
    }
  };

  const stopTask = () => {
    if (canStop) {
      onStop();
    }
  };

  return (
      <AsTile>
        <AsPageTitle>
          <AsH2>{FORM.SECTIONS.SUMMARY}</AsH2>
          <AsButtonBar>
            <AsTooltip title={!canDelete && UIStrings.PERMISSION_ERROR}>
              <AsButton
                  type="button"
                  className={!canDelete && 'disabled'}
                  onClick={deleteTask}
                  variant="tertiary"
              >
                <AsFontAwesomeIcon icon={faTrash}/>
                <span>{UIStrings.SETTINGS.DELETE_BUTTON_LABEL}</span>
              </AsButton>
            </AsTooltip>
            {runnable && task.enabled &&
                <AsTooltip title={!canRun && UIStrings.PERMISSION_ERROR}>
                  <AsButton
                      type="button"
                      className={!canRun && 'disabled'}
                      onClick={runTask}
                      variant="tertiary"
                  >
                    <AsFontAwesomeIcon icon={faPlay}/>
                    <span>{LABELS.BUTTONS.RUN}</span>
                  </AsButton>
                </AsTooltip>
            }
            {stoppable &&
                <AsTooltip title={!canStop && UIStrings.PERMISSION_ERROR}>
                  <AsButton
                      type="button"
                      className={!canStop && 'disabled'}
                      onClick={stopTask}
                      variant="tertiary"
                  >
                    <AsFontAwesomeIcon icon={faStop}/>
                    <span>{LABELS.BUTTONS.STOP}</span>
                  </AsButton>
                </AsTooltip>
            }
          </AsButtonBar>
        </AsPageTitle>
        <AsTile.Content>
          <AsGrid.Row>
            <AsGrid.Column className="as-grid-col--33">
              <AsReadOnly>
                <ReadOnlyField label={LABELS.ID} value={task.id}/>
                <ReadOnlyField label={LABELS.NAME} value={task.name}/>
                <ReadOnlyField label={LABELS.TYPE} value={task.typeName}/>
              </AsReadOnly>
            </AsGrid.Column>
            <AsGrid.Column className="as-grid-col--67">
              <AsReadOnly className="task-details-right-column">
                <ReadOnlyField label={LABELS.STATUS} value={task.statusDescription} item/>
                <ReadOnlyField label={LABELS.NEXT_RUN} value={task.nextRun} item/>
                <ReadOnlyField label={LABELS.LAST_RUN} value={task.lastRun} item/>
                <ReadOnlyField label={LABELS.LAST_RESULT} value={task.lastRunResult} item/>
              </AsReadOnly>
            </AsGrid.Column>
          </AsGrid.Row>
        </AsTile.Content>
      </AsTile>
  );
};
