

import React, {useCallback} from 'react';
import {IconButton, Box} from '@radix-ui/themes';
import {Bell} from 'lucide-react';
import {useRouter} from '@uirouter/react';

import {ExtJS} from '../../../../interface/ExtJS';
import {useIsVisible} from '../../../../interface/NavigationUtils';
import {useIsPreviewUI} from '../Navigation';
import {useUnreadStatusFailure} from '../hooks';
import {Tooltip} from '../Tooltip/TooltipContainerContext';
import SharedStrings from '../../constants/SharedStrings';

/**
 * Global-header bell that routes to the appropriate system-status page for the
 * user's current UI (Classic vs. Preview) and shows a red "unread" dot when
 * health checks are failing.
 *
 * `SystemStatusRadix` (coreui) and `SystemStatus` (cloud) are thin wrappers
 * around this component — they differ in the root class name, the analytics
 * attribute the cloud build attaches to its icon button, and the look of the
 * button (the self-hosted header draws it like its other bordered controls)
 * and its localized labels.
 */
export default function SystemStatusBell({
  className,
  dataAnalyticsId,
  buttonClassName,
  variant = 'ghost',
  size,
  label = SharedStrings.SYSTEM_STATUS_BELL.SYSTEM_STATUS,
  unhealthyLabel = 'System status -- unhealthy',
}) {
  const isPreviewUI = useIsPreviewUI();
  const supportStatusStateIdentifier = isPreviewUI
    ? 'preview.admin.support.metrichealth'
    : 'admin.support.status';
  const healthChecksFailed = ExtJS.useState(() =>
    ExtJS.state().getValue('health_checks_failed', false),
  );
  const {showDot, markAcknowledged} = useUnreadStatusFailure(healthChecksFailed);
  const router = useRouter();

  const visibilityRequirements = router.stateRegistry.get(supportStatusStateIdentifier)?.data
    ?.visibilityRequirements;

  const isVisibleValue = useIsVisible(visibilityRequirements);

  const onClick = useCallback(() => {
    markAcknowledged();
    router.stateService.go(supportStatusStateIdentifier);
  }, [markAcknowledged, router.stateService, supportStatusStateIdentifier]);

  if (!visibilityRequirements || !isVisibleValue) {
    return null;
  }

  const currentLabel = showDot ? unhealthyLabel : label;

  return (
    <Box className={className}>
      <Tooltip content={currentLabel}>
        <IconButton
          variant={variant}
          size={size}
          className={buttonClassName}
          onClick={onClick}
          aria-label={currentLabel}
          {...(dataAnalyticsId ? {'data-analytics-id': dataAnalyticsId} : {})}
        >
          <Bell size={18} aria-hidden />
        </IconButton>
      </Tooltip>
      {showDot && <Box className={`${className}__badge`} aria-hidden="true" />}
    </Box>
  );
}
