

/**
 * Preview ROUTE_NAMES dictionary.
 *
 * Layers Astrum One route names over the base astrum-ui-plugin RouteNames.
 * Preview components should import from here (directly, or as PreviewRouteNames
 * from the @globalart/astrum-ui-plugin package barrel) so they remain portable
 * across plugins.
 *
 * Each plugin's routerConfig/routeNames/routeNames.js merges these on top of
 * the route names only its own shell needs.
 */
import { mergeDeepRight } from 'ramda';
import { RouteNames as BaseRouteNames } from '../../../constants/RouteNames';

export const ROUTE_NAMES = mergeDeepRight(BaseRouteNames, {
  BROWSE: {
    WELCOME: {
      ROOT: 'preview.browse.welcome',
      TITLE: 'Dashboard',
    },
  },
});
