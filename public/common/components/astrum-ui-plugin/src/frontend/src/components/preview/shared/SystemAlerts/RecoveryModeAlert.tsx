

import React from 'react';
import { useCurrentStateAndParams, useRouter } from '@uirouter/react';
import { ExtJS } from '../../../../interface/ExtJS';

import { SystemAlert } from './SystemAlert';
import SharedStrings from '../../constants/SharedStrings';

/** UI-Router state name for the Preview UI Recovery Mode page. */
const RECOVERY_MODE_STATE = 'preview.admin.support.recoverymode';

const LABELS = {
  TITLE: SharedStrings.RECOVERY_MODE_ALERT.RECOVERY_MODE_ENABLED,
  MESSAGE:
    SharedStrings.RECOVERY_MODE_ALERT.WHILE_RECOVERY_MODE_DATA_REPAIR,
  VIEW_DETAILS: SharedStrings.RECOVERY_MODE_ALERT.VIEW_DETAILS,
};

/**
 * RecoveryModeAlert - Self-gating system banner for the Preview UI.
 *
 * Renders only for administrators while recovery mode is active. State comes
 * from the `recovery.mode.enabled` app-state key (contributed by the backend),
 * read reactively so the banner appears/disappears as the mode toggles.
 *
 * Designed to be composed inside the SystemAlerts host alongside other
 * self-gating alerts, mirroring the classic SystemNotices pattern.
 */
export function RecoveryModeAlert(): React.ReactElement | null {
  const user = ExtJS.useUser ? ExtJS.useUser() : undefined;
  const isAdmin = !!user?.administrator;

  const recoveryModeEnabled = ExtJS.useState?.(() =>
    ExtJS.state()?.getValue?.('recovery.mode.enabled')
  );

  const router = useRouter();

  // Use UI-Router's active state (not raw hash matching) to know if we're on
  // the Recovery Mode page; robust to query params and route renames.
  const { state } = useCurrentStateAndParams();
  const isOnRecoveryModePage = state?.name === RECOVERY_MODE_STATE;

  if (!isAdmin || !recoveryModeEnabled) {
    return null;
  }

  return (
    <SystemAlert
      tier="info"
      title={LABELS.TITLE}
      message={LABELS.MESSAGE}
      // Hide the "View Details" CTA when already on the Recovery Mode page.
      action={
        isOnRecoveryModePage
          ? undefined
          : {
              label: LABELS.VIEW_DETAILS,
              onClick: () => router.stateService.go(RECOVERY_MODE_STATE),
              analyticsId: 'as-recovery-mode-alert-view-details',
            }
      }
      dismissable={false}
    />
  );
}

export default RecoveryModeAlert;
