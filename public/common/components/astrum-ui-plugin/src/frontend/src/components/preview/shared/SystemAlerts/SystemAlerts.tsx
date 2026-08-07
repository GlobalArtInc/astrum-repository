

import React from 'react';
import { Box } from '@radix-ui/themes';

import { useIsPreviewUI } from '../Navigation';
import { RecoveryModeAlert } from './RecoveryModeAlert';

import './SystemAlerts.scss';

export interface SystemAlertsProps {
  /** Additional CSS class. */
  className?: string;
}

/**
 * SystemAlerts - Application-scoped banner host for the Preview UI.
 *
 * Renders only in Preview UI (gated by useIsPreviewUI). Mounted once per app
 * shell (coreui / cloudui) above the global header.
 *
 * Composes self-gating alert children (each decides its own visibility),
 * mirroring the classic SystemNotices pattern. Add new system banners here.
 */
export function SystemAlerts({ className = '' }: SystemAlertsProps): React.ReactElement | null {
  const isPreview = useIsPreviewUI();

  if (!isPreview) {
    return null;
  }

  return (
    <Box className={`as-system-alerts ${className}`.trim()} data-testid="as-system-alerts">
      <RecoveryModeAlert />
    </Box>
  );
}

export default SystemAlerts;
