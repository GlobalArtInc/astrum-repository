

import React from 'react';
import { DirectoryList, DirectoryPage } from '@globalart/astrum-ui-plugin';
import { ROUTE_NAMES } from '../../../routerConfig/routeNames/routeNames';
import UIStrings from '../../../constants/UIStrings';

export default function AdminSupportDirectoryPage() {
  return (
      <DirectoryPage
          routeName={ROUTE_NAMES.ADMIN.SUPPORT.DIRECTORY}
          text={UIStrings.ADMIN_SUPPORT_DIRECTORY.MENU.text}
          description={UIStrings.ADMIN_SUPPORT_DIRECTORY.MENU.description}
      >
        <DirectoryList>
          <DirectoryList.DirectoryListItem
              data-analytics-id="as-admin-support-directory-logging-lnk"
              text={UIStrings.LOGGING.MENU.text}
              description={UIStrings.LOGGING.MENU.description}
              routeName={ROUTE_NAMES.ADMIN.SUPPORT.LOGGING.LIST}
              params={{ itemId: null }}
          />

          <DirectoryList.DirectoryListItem
              data-analytics-id="as-admin-support-directory-logs-lnk"
              text={UIStrings.LOGS.MENU.text}
              description={UIStrings.LOGS.MENU.description}
              routeName={ROUTE_NAMES.ADMIN.SUPPORT.LOGS.ROOT}
              params={{ itemId: null }}
          />

          <DirectoryList.DirectoryListItem
              data-analytics-id="as-admin-support-directory-recovery-lnk"
              text={UIStrings.RECOVERY_MODE.MENU.text}
              description={UIStrings.RECOVERY_MODE.MENU.description}
              routeName={ROUTE_NAMES.ADMIN.SUPPORT.RECOVERY.ROOT}
          />

          <DirectoryList.DirectoryListItem
              data-analytics-id="as-admin-support-directory-status-lnk"
              text={UIStrings.METRIC_HEALTH.MENU.text}
              description={UIStrings.METRIC_HEALTH.MENU.description}
              routeName={ROUTE_NAMES.ADMIN.SUPPORT.STATUS.ROOT}
              params={{ itemId: null }}
          />

          <DirectoryList.DirectoryListItem
              data-analytics-id="as-admin-support-directory-supportreqeuest-lnk"
              text={UIStrings.SUPPORT_REQUEST.MENU.text}
              description={UIStrings.SUPPORT_REQUEST.MENU.description}
              routeName={ROUTE_NAMES.ADMIN.SUPPORT.SUPPORTREQUEST.ROOT}
          />

          <DirectoryList.DirectoryListItem
              data-analytics-id="as-admin-support-directory-supportzip-lnk"
              text={UIStrings.SUPPORT_ZIP.MENU.text}
              description={UIStrings.SUPPORT_ZIP.MENU.description}
              routeName={ROUTE_NAMES.ADMIN.SUPPORT.SUPPORTZIP.ROOT}
          />

          <DirectoryList.DirectoryListItem
              data-analytics-id="as-admin-support-directory-system-info-lnk"
              text={UIStrings.SYSTEM_INFORMATION.MENU.text}
              description={UIStrings.SYSTEM_INFORMATION.MENU.description}
              routeName={ROUTE_NAMES.ADMIN.SUPPORT.SYSTEMINFORMATION.ROOT}
          />
        </DirectoryList>
      </DirectoryPage>);
}
