

import React from 'react';
import { ExtJS, useIsVisible } from '@globalart/astrum-ui-plugin';
import { faCircle, faBell } from "@fortawesome/free-solid-svg-icons";
import { NxButton, NxFontAwesomeIcon } from '@globalart/react-shared-components';
import { useRouter } from '@uirouter/react';

export default function SystemStatus() {
  const supportStatusStateIdentifier = 'admin.support.status';
  const healthChecksFailed = ExtJS.useState(() => ExtJS.state().getValue('health_checks_failed', false));
  const router = useRouter();

  const visibilityRequirements =
      router.stateRegistry.get(supportStatusStateIdentifier)
          ?.data
          ?.visibilityRequirements;

  const isVisibleValue = useIsVisible(visibilityRequirements);

  // Cloud / preview may not register admin.support.status; avoid console noise and render nothing.
  if (!visibilityRequirements || !isVisibleValue) {
    return null;
  }

  return (
      <NxButton
          onClick={onClick}
          title="System Status"
          aria-label="System Status"
          variant="icon-only"
          data-analytics-id="as-global-header-system-status"
      >
        {healthChecksFailed
            ? (<span
                role="alert" aria-label="system status -- unhealthy" className="fa-layers fa-fw">
                <NxFontAwesomeIcon icon={faBell}/>
                <NxFontAwesomeIcon
                    className="as-health-check-alert-circle"
                    icon={faCircle}
                    transform="shrink-8 up-5 right-5"
                />
              </span>)
            : <span role="status" aria-label="system status -- healthy">
                <NxFontAwesomeIcon icon={faBell}/>
                </span>
        }
      </NxButton>
  )

  function onClick() {
    router.stateService.go(supportStatusStateIdentifier);
  }
}
