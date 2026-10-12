

import React from 'react';
import { Box } from '@radix-ui/themes';

import { RecoveryModeAlert } from './RecoveryModeAlert';

import './SystemAlerts.scss';

export interface SystemAlertsProps {
  /** Additional CSS class. */
  className?: string;
}

/**
 * SystemAlerts - Application-scoped banner host of Astrum One.
 *
 * Mounted once per app shell (coreui / cloudui) above the global header.
 *
 * Composes self-gating alert children (each decides its own visibility).
 * Add new system banners here.
 */
export function SystemAlerts({ className = '' }: SystemAlertsProps): React.ReactElement {
  return (
    <Box className={`as-system-alerts ${className}`.trim()} data-testid="as-system-alerts">
      <RecoveryModeAlert />
    </Box>
  );
}

export default SystemAlerts;
