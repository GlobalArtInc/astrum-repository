
import React from 'react';

import {
  NxTile,
  NxPageTitle,
  NxFontAwesomeIcon,
  NxButton,
  NxButtonBar,
  NxH2,
  NxTooltip,
  NxGrid,
  NxReadOnly,
} from '@globalart/react-shared-components';
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
      <NxTile>
        <NxPageTitle>
          <NxH2>{FORM.SECTIONS.SUMMARY}</NxH2>
          <NxButtonBar>
            <NxTooltip title={!canDelete && UIStrings.PERMISSION_ERROR}>
              <NxButton
                  type="button"
                  className={!canDelete && 'disabled'}
                  onClick={deleteTask}
                  variant="tertiary"
              >
                <NxFontAwesomeIcon icon={faTrash}/>
                <span>{UIStrings.SETTINGS.DELETE_BUTTON_LABEL}</span>
              </NxButton>
            </NxTooltip>
            {runnable && task.enabled &&
                <NxTooltip title={!canRun && UIStrings.PERMISSION_ERROR}>
                  <NxButton
                      type="button"
                      className={!canRun && 'disabled'}
                      onClick={runTask}
                      variant="tertiary"
                  >
                    <NxFontAwesomeIcon icon={faPlay}/>
                    <span>{LABELS.BUTTONS.RUN}</span>
                  </NxButton>
                </NxTooltip>
            }
            {stoppable &&
                <NxTooltip title={!canStop && UIStrings.PERMISSION_ERROR}>
                  <NxButton
                      type="button"
                      className={!canStop && 'disabled'}
                      onClick={stopTask}
                      variant="tertiary"
                  >
                    <NxFontAwesomeIcon icon={faStop}/>
                    <span>{LABELS.BUTTONS.STOP}</span>
                  </NxButton>
                </NxTooltip>
            }
          </NxButtonBar>
        </NxPageTitle>
        <NxTile.Content>
          <NxGrid.Row>
            <NxGrid.Column className="nx-grid-col--33">
              <NxReadOnly>
                <ReadOnlyField label={LABELS.ID} value={task.id}/>
                <ReadOnlyField label={LABELS.NAME} value={task.name}/>
                <ReadOnlyField label={LABELS.TYPE} value={task.typeName}/>
              </NxReadOnly>
            </NxGrid.Column>
            <NxGrid.Column className="nx-grid-col--67">
              <NxReadOnly className="task-details-right-column">
                <ReadOnlyField label={LABELS.STATUS} value={task.statusDescription} item/>
                <ReadOnlyField label={LABELS.NEXT_RUN} value={task.nextRun} item/>
                <ReadOnlyField label={LABELS.LAST_RUN} value={task.lastRun} item/>
                <ReadOnlyField label={LABELS.LAST_RESULT} value={task.lastRunResult} item/>
              </NxReadOnly>
            </NxGrid.Column>
          </NxGrid.Row>
        </NxTile.Content>
      </NxTile>
  );
};
