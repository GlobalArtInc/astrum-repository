

import React from 'react';
import {useIsPreviewUI, SystemAlerts} from '@globalart/astrum-ui-plugin';

import SystemNotices from './SystemNotices';
import './SystemNoticesSwitch.scss';

export default function SystemNoticesSwitch() {
  const isPreviewUI = useIsPreviewUI();

  return (
    <div className="as-notices-stack">
      {isPreviewUI && <SystemAlerts />}
      <SystemNotices isPreviewUI={isPreviewUI} />
    </div>
  );
}
