import {createRouter, MissingRoutePage} from '@globalart/astrum-ui-plugin';
import {ROUTE_NAMES} from '../routerConfig/routeNames/routeNames';
import {browseRoutes} from '../routerConfig/routes/browseRoutes';
import {adminRoutes} from '../routerConfig/routes/adminRoutes';
import {userRoutes} from '../routerConfig/routes/userRoutes';
import {loginRoutes} from '../routerConfig/routes/loginRoutes';

/**
 * A router with the routes of the retired Classic UI, which the application no longer registers. Only the tests
 * of the Classic components still in the code use it, until those components are deleted.
 */
export function getClassicRouter() {
  return createRouter({
    initialRoute: ROUTE_NAMES.BROWSE.WELCOME.ROOT,
    menuRoutes: [...browseRoutes, ...adminRoutes, ...userRoutes, ...loginRoutes],
    missingRoute: {
      name: ROUTE_NAMES.MISSING_ROUTE,
      url: '404',
      component: MissingRoutePage,
      data: {visibilityRequirements: {}},
    },
  });
}
