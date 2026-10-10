

import React, { useState } from 'react';
import { useCurrentStateAndParams } from '@uirouter/react';
import { AsFilterInput } from '@globalart/astrum-react-shared-components';
import {ExtJS, useIsVisible, handleExtJsUnsavedChanges} from '@globalart/astrum-ui-plugin';
import { useRouter } from '@uirouter/react';
import {ROUTE_NAMES} from "../../routerConfig/routeNames/routeNames";

export default function Search() {
  const [searchValue, setSearchValue] = useState("");
  const router = useRouter();
  const { state} = useCurrentStateAndParams();

  const searchRoute = router.stateRegistry.get(ROUTE_NAMES.BROWSE.SEARCH.GENERIC);
  const isVisible = useIsVisible(searchRoute?.data?.visibilityRequirements);

  if (!isVisible) {
    return null;
  }

  return (<AsFilterInput
      placeholder="Search components"
      value={searchValue}
      onChange={setSearchValue}
      searchIcon
      onKeyPress={onSearchKeyPress}
      inputAttributes={{
        title: "Search components"
      }}
      data-analytics-id="as-global-header-search-input"
  />);

  async function onSearchKeyPress(key) {
    if (key === 'Enter') {
      // Try to get the ExtJS Menu controller
      const menuCtrl =
        window.Ext && Ext.getApplication && Ext.getApplication().getController
          ? Ext.getApplication().getController('Menu')
          : null;
      handleExtJsUnsavedChanges(menuCtrl, async () => {
        if (state.name !== ROUTE_NAMES.BROWSE.SEARCH.GENERIC) {
          await router.stateService.go(ROUTE_NAMES.BROWSE.SEARCH.GENERIC, { keyword: `=keyword=${encodeURIComponent(searchValue)}` });
        } else {
          ExtJS.search(searchValue);
        }
      });
    }
  }
}

