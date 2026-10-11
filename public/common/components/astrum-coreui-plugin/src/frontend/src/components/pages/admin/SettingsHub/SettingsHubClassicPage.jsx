

import React from 'react';
import { DirectoryList, DirectoryPage, isExtjsCapabilitiesEnabled, isReactCapabilitiesEnabled } from '@globalart/astrum-ui-plugin';
import { ROUTE_NAMES } from '../../../../routerConfig/routeNames/routeNames';
import UIStrings from '../../../../constants/UIStrings';

const sectionHeadingStyle = { margin: '24px 0 8px', fontSize: '18px', fontWeight: 600 };

export default function SettingsHubClassicPage() {
  const ADMIN = ROUTE_NAMES.ADMIN;
  const HUB = UIStrings.SETTINGS_HUB;

  return (
    <DirectoryPage
      routeName={ADMIN.DIRECTORY}
      text={HUB.TITLE}
      description={HUB.DESCRIPTION}
    >
      <h3 style={sectionHeadingStyle}>{HUB.SECURITY.TITLE}</h3>
      <DirectoryList>
        <DirectoryList.DirectoryListItem text={HUB.SECURITY.USERS.TEXT} description={HUB.SECURITY.USERS.DESCRIPTION} routeName={ADMIN.SECURITY.USERS.ROOT} params={{ itemId: null }} />
        <DirectoryList.DirectoryListItem text={HUB.SECURITY.ROLES.TEXT} description={HUB.SECURITY.ROLES.DESCRIPTION} routeName={ADMIN.SECURITY.ROLES.LIST} />
        <DirectoryList.DirectoryListItem text={HUB.SECURITY.PRIVILEGES.TEXT} description={HUB.SECURITY.PRIVILEGES.DESCRIPTION} routeName={ADMIN.SECURITY.PRIVILEGES.LIST} />
        <DirectoryList.DirectoryListItem text={HUB.SECURITY.REALMS.TEXT} description={HUB.SECURITY.REALMS.DESCRIPTION} routeName={ADMIN.SECURITY.REALMS.ROOT} />
        <DirectoryList.DirectoryListItem text={HUB.SECURITY.LDAP.TEXT} description={HUB.SECURITY.LDAP.DESCRIPTION} routeName={ADMIN.SECURITY.LDAP.ROOT} params={{ itemId: null }} />
        <DirectoryList.DirectoryListItem text={HUB.SECURITY.SAML.TEXT} description={HUB.SECURITY.SAML.DESCRIPTION} routeName={ADMIN.SECURITY.SAML.ROOT} />
        <DirectoryList.DirectoryListItem text={HUB.SECURITY.OAUTH2.TEXT} description={HUB.SECURITY.OAUTH2.DESCRIPTION} routeName={ADMIN.SECURITY.OAUTH2.ROOT} />
        <DirectoryList.DirectoryListItem text={HUB.SECURITY.ATLASSIAN_CROWD.TEXT} description={HUB.SECURITY.ATLASSIAN_CROWD.DESCRIPTION} routeName={ADMIN.SECURITY.ATLASSIANCROWD.ROOT} />
        <DirectoryList.DirectoryListItem text={HUB.SECURITY.ANONYMOUS_ACCESS.TEXT} description={HUB.SECURITY.ANONYMOUS_ACCESS.DESCRIPTION} routeName={ADMIN.SECURITY.ANONYMOUS.ROOT} />
        <DirectoryList.DirectoryListItem text={HUB.SECURITY.SSL_CERTIFICATES.TEXT} description={HUB.SECURITY.SSL_CERTIFICATES.DESCRIPTION} routeName={ADMIN.SECURITY.SSLCERTIFICATES.LIST} />
        <DirectoryList.DirectoryListItem text={HUB.SECURITY.USER_TOKENS.TEXT} description={HUB.SECURITY.USER_TOKENS.DESCRIPTION} routeName={ADMIN.SECURITY.USERTOKEN.ROOT} />
      </DirectoryList>

      <h3 style={sectionHeadingStyle}>{HUB.REPOSITORY.TITLE}</h3>
      <DirectoryList>
        <DirectoryList.DirectoryListItem text={HUB.REPOSITORY.REPOSITORIES.TEXT} description={HUB.REPOSITORY.REPOSITORIES.DESCRIPTION} routeName={ADMIN.REPOSITORY.REPOSITORIES.ROOT} params={{ itemId: null }} />
        <DirectoryList.DirectoryListItem text={HUB.REPOSITORY.BLOB_STORES.TEXT} description={HUB.REPOSITORY.BLOB_STORES.DESCRIPTION} routeName={ADMIN.REPOSITORY.BLOBSTORES.LIST} />
        <DirectoryList.DirectoryListItem text={HUB.REPOSITORY.CLEANUP_POLICIES.TEXT} description={HUB.REPOSITORY.CLEANUP_POLICIES.DESCRIPTION} routeName={ADMIN.REPOSITORY.CLEANUPPOLICIES.LIST} />
        <DirectoryList.DirectoryListItem text={HUB.REPOSITORY.ROUTING_RULES.TEXT} description={HUB.REPOSITORY.ROUTING_RULES.DESCRIPTION} routeName={ADMIN.REPOSITORY.ROUTINGRULES.LIST} />
        <DirectoryList.DirectoryListItem text={HUB.REPOSITORY.CONTENT_SELECTORS.TEXT} description={HUB.REPOSITORY.CONTENT_SELECTORS.DESCRIPTION} routeName={ADMIN.REPOSITORY.SELECTORS.LIST} />
        <DirectoryList.DirectoryListItem text={HUB.REPOSITORY.DATA_STORE.TEXT} description={HUB.REPOSITORY.DATA_STORE.DESCRIPTION} routeName={ADMIN.REPOSITORY.DATASTORE.ROOT} />
        <DirectoryList.DirectoryListItem text={HUB.REPOSITORY.PROPRIETARY.TEXT} description={HUB.REPOSITORY.PROPRIETARY.DESCRIPTION} routeName={ADMIN.REPOSITORY.PROPRIETARY.ROOT} />
      </DirectoryList>

      <h3 style={sectionHeadingStyle}>{HUB.SYSTEM.TITLE}</h3>
      <DirectoryList>
        <DirectoryList.DirectoryListItem text={HUB.SYSTEM.TASKS.TEXT} description={HUB.SYSTEM.TASKS.DESCRIPTION} routeName={ADMIN.SYSTEM.TASKS.ROOT} params={{ taskId: null }} />
        <DirectoryList.DirectoryListItem text={HUB.SYSTEM.EMAIL_SERVER.TEXT} description={HUB.SYSTEM.EMAIL_SERVER.DESCRIPTION} routeName={ADMIN.SYSTEM.EMAILSERVER.ROOT} />
        <DirectoryList.DirectoryListItem text={HUB.SYSTEM.HTTP.TEXT} description={HUB.SYSTEM.HTTP.DESCRIPTION} routeName={ADMIN.SYSTEM.HTTP.ROOT} />
        <DirectoryList.DirectoryListItem text={HUB.SYSTEM.API.TEXT} description={HUB.SYSTEM.API.DESCRIPTION} routeName={ADMIN.SYSTEM.API.ROOT} />
        {isExtjsCapabilitiesEnabled() &&
          <DirectoryList.DirectoryListItem text={HUB.SYSTEM.CAPABILITIES.TEXT} description={HUB.SYSTEM.CAPABILITIES.DESCRIPTION} routeName={ADMIN.SYSTEM.CAPABILITIES_EXTJS.ROOT} params={{ id: null }} />
        }
        {isReactCapabilitiesEnabled() &&
          <DirectoryList.DirectoryListItem text={HUB.SYSTEM.CAPABILITIES.TEXT} description={HUB.SYSTEM.CAPABILITIES.DESCRIPTION} routeName={ADMIN.SYSTEM.CAPABILITIES.LIST} />
        }
        <DirectoryList.DirectoryListItem text={HUB.SYSTEM.NODES.TEXT} description={HUB.SYSTEM.NODES.DESCRIPTION} routeName={ADMIN.SYSTEM.NODES.ROOT} />
        <DirectoryList.DirectoryListItem text={HUB.SYSTEM.NOTIFICATIONS.TEXT} description={HUB.SYSTEM.NOTIFICATIONS.DESCRIPTION} routeName={ADMIN.SYSTEM.NOTIFICATIONS.ROOT} />
        <DirectoryList.DirectoryListItem text={HUB.SYSTEM.ASTRUM_ONE_UI.TEXT} description={HUB.SYSTEM.ASTRUM_ONE_UI.DESCRIPTION} routeName={ADMIN.SYSTEM.PREVIEWUI.ROOT} />
      </DirectoryList>

      <h3 style={sectionHeadingStyle}>{HUB.SUPPORT.TITLE}</h3>
      <DirectoryList>
        <DirectoryList.DirectoryListItem text={HUB.SUPPORT.SYSTEM_INFORMATION.TEXT} description={HUB.SUPPORT.SYSTEM_INFORMATION.DESCRIPTION} routeName={ADMIN.SUPPORT.SYSTEMINFORMATION.ROOT} />
        <DirectoryList.DirectoryListItem text={HUB.SUPPORT.LOGS.TEXT} description={HUB.SUPPORT.LOGS.DESCRIPTION} routeName={ADMIN.SUPPORT.LOGS.ROOT} params={{ itemId: null }} />
        <DirectoryList.DirectoryListItem text={HUB.SUPPORT.LOGGING_CONFIGURATION.TEXT} description={HUB.SUPPORT.LOGGING_CONFIGURATION.DESCRIPTION} routeName={ADMIN.SUPPORT.LOGGING.LIST} />
        <DirectoryList.DirectoryListItem text={HUB.SUPPORT.METRIC_HEALTH.TEXT} description={HUB.SUPPORT.METRIC_HEALTH.DESCRIPTION} routeName={ADMIN.SUPPORT.STATUS.ROOT} params={{ itemId: null }} />
        <DirectoryList.DirectoryListItem text={HUB.SUPPORT.SUPPORT_ZIP.TEXT} description={HUB.SUPPORT.SUPPORT_ZIP.DESCRIPTION} routeName={ADMIN.SUPPORT.SUPPORTZIP.ROOT} />
        <DirectoryList.DirectoryListItem text={HUB.SUPPORT.SUPPORT_REQUEST.TEXT} description={HUB.SUPPORT.SUPPORT_REQUEST.DESCRIPTION} routeName={ADMIN.SUPPORT.SUPPORTREQUEST.ROOT} />
      </DirectoryList>
    </DirectoryPage>
  );
}
