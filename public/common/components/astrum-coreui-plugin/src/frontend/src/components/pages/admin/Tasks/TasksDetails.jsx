
import React from 'react';
import {useMachine} from '@xstate/react';

import {
  ContentBody,
  PageHeader,
  PageTitle,
  ValidationUtils
} from '@globalart/astrum-ui-plugin';

import Machine from './TasksFormMachine';
import TasksForm from './TasksForm';
import TasksReadOnly from './TasksReadOnly';
import {canUpdateTask} from './TasksHelper';

import UIStrings from '../../../../constants/UIStrings';

export default function TasksDetails({itemId, onDone}) {
  const hasEditPermissions = canUpdateTask();

  const [state, , service] = useMachine(Machine, {
    context: {
      pristineData: {
        id: decodeURIComponent(itemId),
      }
    },
    actions: {
      onSaveSuccess: onDone,
      onDeleteSuccess: onDone,
    },
    devTools: true,
  });

  const {pristineData: {id}} = state.context;

  const isEdit = ValidationUtils.notBlank(id);
  const showReadOnly = isEdit && !hasEditPermissions;

  return <div className="as-task">
    <PageHeader>
      <PageTitle
          icon={UIStrings.TASKS.MENU.icon}
          text={UIStrings.TASKS.MENU.text}
          description={UIStrings.TASKS.MENU.description}
      />
    </PageHeader>
    <ContentBody className="as-task-form">
      {showReadOnly
          ? <TasksReadOnly service={service} onDone={onDone}/>
          : <TasksForm service={service} onDone={onDone}/>
      }
    </ContentBody>
  </div>;
}
