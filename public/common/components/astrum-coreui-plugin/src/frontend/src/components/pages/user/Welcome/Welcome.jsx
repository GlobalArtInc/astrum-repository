import React from 'react';
import {AsPageMain} from '@globalart/astrum-react-shared-components';
import {ExtJS, Permissions, UsageDashboard} from '@globalart/astrum-ui-plugin';

import './Welcome.scss';

export default function Welcome() {
  const user = ExtJS.useUser();
  const canSearch = ExtJS.usePermission(() => ExtJS.checkPermission(Permissions.SEARCH.READ), []);
  const canConfigureCleanup = ExtJS.usePermission(() => ExtJS.checkPermission(Permissions.ADMIN), []) && !!user;

  return (
      <AsPageMain>
        <div className="as-welcome">
          <div className="as-welcome__outreach as-viewport-sized__scrollable">
            <div className="as-welcome__usage">
              <UsageDashboard variant="classic" canSearch={canSearch} canConfigureCleanup={canConfigureCleanup}/>
            </div>
          </div>
        </div>
      </AsPageMain>
  );
}
