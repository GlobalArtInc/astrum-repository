

import React, { useEffect } from 'react';
import { UIView, useCurrentStateAndParams, useRouter } from '@uirouter/react';
import { Page } from '../../layout';
import { NxH1, NxP, NxTile } from '@globalart/react-shared-components';
import { RouteNames } from '../../../constants/RouteNames';
import { isVisible } from '../../../interface/NavigationUtils';

export function DirectoryPage({ routeName, text, description, children, ...attr }) {
  const { state } = useCurrentStateAndParams();
  const router = useRouter();

  useEffect(() => {
    if (state.name === routeName) {
      const allStates = router.stateRegistry.get();
      const visibleChildRoutes = allStates.filter(
        (childState) =>
          childState.name.startsWith(`${routeName}.`) &&
          !childState?.data?.visibilityRequirements?.ignoreForMenuVisibilityCheck &&
          isVisible(childState.data?.visibilityRequirements)
      );

      if (visibleChildRoutes.length === 0) {
        router.stateService.go(RouteNames.MISSING_ROUTE);
      }
    }
  }, [state.name, routeName, router]);

  if (state.name !== routeName) {
    return <UIView />;
  }

  return (
    <Page {...attr}>
      <NxH1>{text}</NxH1>

      <NxP>{description}</NxP>

      <NxTile>
        <NxTile.Content>{children}</NxTile.Content>
      </NxTile>
    </Page>
  );
}
