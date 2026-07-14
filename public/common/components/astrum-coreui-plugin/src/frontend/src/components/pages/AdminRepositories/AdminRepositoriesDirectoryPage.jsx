

import React from 'react';
import { DirectoryList, DirectoryPage } from '@globalart/astrum-ui-plugin';
import UIStrings from '../../../constants/UIStrings';
import { ROUTE_NAMES } from '../../../routerConfig/routeNames/routeNames';

export default function AdminRepositoriesDirectoryPage() {
  const ADMIN = ROUTE_NAMES.ADMIN;

  return (
      <DirectoryPage
        routeName={ROUTE_NAMES.ADMIN.REPOSITORY.DIRECTORY}
        text={UIStrings.REPOSITORY_DIRECTORY.MENU.text}
        description={UIStrings.REPOSITORY_DIRECTORY.MENU.description}
      >
        <DirectoryList>
          <DirectoryList.DirectoryListItem
              data-analytics-id="as-admin-repository-directory-repositories-lnk"
              text={UIStrings.REPOSITORIES.MENU.text}
              description={UIStrings.REPOSITORIES.MENU.description}
              routeName={ADMIN.REPOSITORY.REPOSITORIES.ROOT}
              params={{ itemId: null }}
          />

          <DirectoryList.DirectoryListItem
              data-analytics-id="as-admin-repository-directory-blobstores-lnk"
              text={UIStrings.BLOB_STORES.MENU.text}
              description={UIStrings.BLOB_STORES.MENU.description}
              routeName={ADMIN.REPOSITORY.BLOBSTORES.LIST}
          />

          <DirectoryList.DirectoryListItem
              data-analytics-id="as-admin-repository-directory-datastore-lnk"
              text={UIStrings.DATASTORE_CONFIGURATION.MENU.text}
              description={UIStrings.DATASTORE_CONFIGURATION.MENU.description}
              routeName={ADMIN.REPOSITORY.DATASTORE.ROOT}
          />

          <DirectoryList.DirectoryListItem
              data-analytics-id="as-admin-repository-directory-proprietary-lnk"
              {...UIStrings.PROPRIETARY_REPOSITORIES.MENU}
              routeName={ADMIN.REPOSITORY.PROPRIETARY.ROOT}
          />

          <DirectoryList.DirectoryListItem
              data-analytics-id="as-admin-repository-directory-selectors-lnk"
              text={UIStrings.CONTENT_SELECTORS.MENU.text}
              description={UIStrings.CONTENT_SELECTORS.MENU.description}
              routeName={ADMIN.REPOSITORY.SELECTORS.LIST}
              params={{ itemId: null }}
          />

          <DirectoryList.DirectoryListItem
              data-analytics-id="as-admin-repository-directory-cleanup-policies-lnk"
              text={UIStrings.CLEANUP_POLICIES.MENU.text}
              description={UIStrings.CLEANUP_POLICIES.MENU.description}
              routeName={ADMIN.REPOSITORY.CLEANUPPOLICIES.LIST}
              params={{ itemId: null }}
          />

          <DirectoryList.DirectoryListItem
              data-analytics-id="as-admin-repository-directory-routingrules-lnk"
              text={UIStrings.ROUTING_RULES.MENU.text}
              description={UIStrings.ROUTING_RULES.MENU.description}
              routeName={ADMIN.REPOSITORY.ROUTINGRULES.LIST}
              params={{ itemId: null }}
          />
        </DirectoryList>
      </DirectoryPage>);
}
