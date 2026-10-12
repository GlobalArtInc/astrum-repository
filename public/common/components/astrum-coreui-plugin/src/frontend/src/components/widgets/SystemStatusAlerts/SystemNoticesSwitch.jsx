
import React from 'react';
import {SystemAlerts} from '@globalart/astrum-ui-plugin';

import SystemNotices from './SystemNotices';
import './SystemNoticesSwitch.scss';

export default function SystemNoticesSwitch() {
  return (
    <div className="as-notices-stack">
      <SystemAlerts />
      <SystemNotices />
    </div>
  );
}
