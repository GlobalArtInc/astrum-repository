

import React from 'react';
import { ExtJsContainer } from '../../../widgets/ExtJsContainer/ExtJsContainer';
import UIStrings from '../../../../constants/UIStrings';

export default function TasksExtJSWrapper() {
  return <ExtJsContainer
      title={UIStrings.TASKS.MENU.text}
      icon={UIStrings.TASKS.MENU.icon}
      extView="AS.coreui.view.task.TaskFeature"
  />;
}
