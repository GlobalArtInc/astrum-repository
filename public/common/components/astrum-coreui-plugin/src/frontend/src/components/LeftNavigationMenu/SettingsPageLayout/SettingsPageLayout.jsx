
import React, { Suspense } from 'react';
import { UIView } from '@uirouter/react';
import { RouteLoadingFallback } from '@globalart/astrum-ui-plugin';

import SettingsSidebar from './SettingsSidebar';

export default function SettingsPageLayout() {
  return (
    <div className="as-settings-layout">
      <SettingsSidebar />
      <Suspense fallback={<RouteLoadingFallback />}>
        <UIView />
      </Suspense>
    </div>
  );
}
