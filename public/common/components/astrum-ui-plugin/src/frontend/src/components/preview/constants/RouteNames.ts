

/**
 * Preview ROUTE_NAMES dictionary.
 *
 * Layers preview-specific route names over the base astrum-ui-plugin RouteNames.
 * Preview components should import from here (directly, or as PreviewRouteNames
 * from the @globalart/astrum-ui-plugin package barrel) so they remain portable
 * across plugins.
 *
 * Preview-only entries live under PREVIEW_* keys so they don't collide with
 * classic entries of the same feature name in each plugin's browseRouteNames.js.
 *
 * Each plugin's routerConfig/routeNames/routeNames.js merges these on top of
 * its own plugin-specific admin/browse/user route names. Plugin-specific
 * classic route names MUST NOT be added here — they belong in the plugin's
 * own browseRouteNames/adminRouteNames/userRouteNames.
 */
import { mergeDeepRight } from 'ramda';
import { RouteNames as BaseRouteNames } from '../../../constants/RouteNames';

export const ROUTE_NAMES = mergeDeepRight(BaseRouteNames, {
  BROWSE: {
    WELCOME: {
      ROOT: 'browse.welcome',
      TITLE: 'Dashboard',
    },
  },
});
