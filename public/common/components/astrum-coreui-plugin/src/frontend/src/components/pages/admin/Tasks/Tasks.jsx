
import React from 'react';

import {Detail, Master, MasterDetail} from '@globalart/astrum-ui-plugin';

import TasksList from './TasksList';
import TasksDetails from './TasksDetails';

import './Tasks.scss';

// Always render the React Tasks experience (ExtJS wrapper removed as part of migration)
export default function Tasks() {
  return (
    <MasterDetail path="admin/system/tasks">
      <Master>
        <TasksList />
      </Master>
      <Detail>
        <TasksDetails />
      </Detail>
    </MasterDetail>
  );
}
