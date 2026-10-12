
import React from 'react';
import {
  AsSystemNotice,
} from '@globalart/astrum-react-shared-components';

import './SystemNotices.scss';
import UpgradeAlert from './UpgradeAlert/UpgradeAlert';

/**
 * The banners of the upgrade of a cluster. The recovery mode banner is rendered by the SystemAlerts host of
 * astrum-ui-plugin.
 */
export default function SystemNotices() {
  return <AsSystemNotice.Container className="as-system-notices">
    <UpgradeAlert />
  </AsSystemNotice.Container>;
}
