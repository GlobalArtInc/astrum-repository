

import UIStrings from '../../../constants/UIStrings';
import { DirectoryList, DirectoryPage, ExtJS } from '@globalart/astrum-ui-plugin';
import React from 'react';
import { ROUTE_NAMES } from '../../../routerConfig/routeNames/routeNames';
import { isExtjsCapabilitiesEnabled, isReactCapabilitiesEnabled } from '@globalart/astrum-ui-plugin';

export default function AdminSystemDirectoryPage() {
  const ADMIN = ROUTE_NAMES.ADMIN;

  return (
      <DirectoryPage
          routeName={ADMIN.SYSTEM.DIRECTORY}
          text={UIStrings.ADMIN_SYSTEM_DIRECTORY.MENU.text}
          description={UIStrings.ADMIN_SYSTEM_DIRECTORY.MENU.description}
      >
        <DirectoryList>
          <DirectoryList.DirectoryListItem
              data-analytics-id="as-admin-system-directory-api-lnk"
              text={UIStrings.API.MENU.text}
              description={UIStrings.API.MENU.description}
              routeName={ADMIN.SYSTEM.API.ROOT}
          />

          {isExtjsCapabilitiesEnabled() &&
            <DirectoryList.DirectoryListItem
                data-analytics-id="as-admin-system-directory-capabilities-lnk"
                text={UIStrings.CAPABILITIES.MENU.text}
                description={UIStrings.CAPABILITIES.MENU.description}
                routeName={ADMIN.SYSTEM.CAPABILITIES_EXTJS.ROOT}
                params={{ id: null }}
            />
          }

          {isReactCapabilitiesEnabled() &&
            <DirectoryList.DirectoryListItem
                data-analytics-id="as-admin-system-directory-capabilities-lnk"
                text={UIStrings.CAPABILITIES.MENU.text}
                description={UIStrings.CAPABILITIES.MENU.description}
                routeName={ADMIN.SYSTEM.CAPABILITIES.LIST}
                params={{ id: null }}
            />
          }

          {ExtJS.state().getValue('astrum.previewui.enabled') &&
            <DirectoryList.DirectoryListItem
                data-analytics-id="as-admin-system-directory-previewui-lnk"
                text={UIStrings.PREVIEW_UI_SETTINGS.MENU.text}
                description={UIStrings.PREVIEW_UI_SETTINGS.MENU.description}
                routeName={ADMIN.SYSTEM.PREVIEWUI.ROOT}
            />
          }

          <DirectoryList.DirectoryListItem
              data-analytics-id="as-admin-system-directory-email-server-lnk"
              text={UIStrings.EMAIL_SERVER.MENU.text}
              description={UIStrings.EMAIL_SERVER.MENU.description}
              routeName={ADMIN.SYSTEM.EMAILSERVER.ROOT}
          />


          <DirectoryList.DirectoryListItem
              data-analytics-id="as-admin-system-directory-http-lnk"
              text={UIStrings.HTTP.MENU.text}
              description={UIStrings.HTTP.MENU.description}
              routeName={ADMIN.SYSTEM.HTTP.ROOT}
          />


          <DirectoryList.DirectoryListItem
              data-analytics-id="as-admin-system-directory-nodes-lnk"
              text={UIStrings.NODES.MENU.text}
              description={UIStrings.NODES.MENU.description}
              routeName={ADMIN.SYSTEM.NODES.ROOT}
          />

          <DirectoryList.DirectoryListItem
              data-analytics-id="as-admin-system-directory-tasks-lnk"
              text={UIStrings.TASKS.MENU.text}
              description={UIStrings.TASKS.MENU.description}
              routeName={ADMIN.SYSTEM.TASKS.ROOT}
              params={{ taskId: null }}
          />

        </DirectoryList>
      </DirectoryPage>);
}
