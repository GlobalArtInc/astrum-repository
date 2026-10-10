

import React from 'react';
import { ExtJS } from '@globalart/astrum-ui-plugin';
import { faCircle, faBell } from "@fortawesome/free-solid-svg-icons";
import { AsButton, AsFontAwesomeIcon } from '@globalart/react-shared-components';

// TODO: Properly format the Bell+Circle state and write tests https://globalart.atlassian.net/browse/ASTRUM-46297
export default function HealthCheck() {
  const healthChecksFailed = ExtJS.useState(() => ExtJS.state().getValue('health_checks_failed', false));

  return (
      <AsButton title="System Status" aria-label="System Status" variant="icon-only">

        {healthChecksFailed
            ? (<span className="fa-layers fa-fw">
              <AsFontAwesomeIcon icon={faBell}/>
              <AsFontAwesomeIcon icon={faCircle} transform="shrink-8 up-3 right-3" style={{color: 'red'}}/>
              </span>)
            : <AsFontAwesomeIcon icon={faBell}/>
        }

      </AsButton>
  )
}
