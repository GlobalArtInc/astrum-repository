import adminRouteNames from './adminRouteNames';
import browseRouteNames from './browseRouteNames';
import { PreviewRouteNames } from '@globalart/astrum-ui-plugin';
import { mergeDeepRight } from 'ramda';

// Astrum One state names of the shell, layered on top of the shared preview ROUTE_NAMES from
// astrum-ui-plugin. Names that pages of both plugins need belong in
// astrum-ui-plugin/src/frontend/src/components/preview/constants/RouteNames.ts.
export const ROUTE_NAMES = mergeDeepRight(PreviewRouteNames, {
  ADMIN: { ...adminRouteNames },
  BROWSE: { ...browseRouteNames },
  LOGIN: 'login'
});
