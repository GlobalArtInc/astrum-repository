

import adminRouteNames from './adminRouteNames';
import browseRouteNames from './browseRouteNames';
import userRouteNames from './userRouteNames';
import { PreviewRouteNames } from '@globalart/astrum-ui-plugin';
import { mergeDeepRight } from 'ramda';

// Classic-UI and plugin-specific route names layered on top of the shared
// preview ROUTE_NAMES from astrum-ui-plugin. Preview-only additions belong in
// astrum-ui-plugin/src/frontend/src/components/preview/constants/RouteNames.ts
// so both plugins pick them up automatically.
export const ROUTE_NAMES = mergeDeepRight(PreviewRouteNames, {
  ADMIN: { ...adminRouteNames },
  BROWSE: { ...browseRouteNames },
  USER: { ...userRouteNames },
  LOGIN: 'login'
});
