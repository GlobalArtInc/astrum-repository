

import React from 'react';
import {ExtJS} from '@globalart/astrum-ui-plugin';
import {
  NxSystemNotice,
} from '@globalart/react-shared-components';

import './SystemNotices.scss';
import UpgradeAlert from './UpgradeAlert/UpgradeAlert';
import RecoveryModeAlert from './RecoveryModeAlert/RecoveryModeAlert';

export default function SystemNotices ({isPreviewUI = false} = {}) {
  const recoveryModeEnabled = ExtJS.useState(() => ExtJS.state().getValue('recovery.mode.enabled'));

  // In Preview UI the recovery banner is rendered separately by the Radix
  // SystemAlerts host, so skip the classic RecoveryModeAlert here to avoid a
  // duplicate. All other banners render as before.
  const showClassicRecoveryAlert = recoveryModeEnabled && !isPreviewUI;

  return <NxSystemNotice.Container className="as-system-notices">
    {showClassicRecoveryAlert ? <RecoveryModeAlert /> : <UpgradeAlert />}
  </NxSystemNotice.Container>;
}
