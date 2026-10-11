
import {createRouter, isGlobalArtDevMode, MissingRoutePage} from '@globalart/astrum-ui-plugin';
import {ROUTE_NAMES} from './routeNames/routeNames';
import {loginRoutes} from "./routes/loginRoutes";
import {previewAdminRoutes, globalartInternalTestRoutes} from './routes/previewAdminRoutes';
import {previewBrowseRoutes} from './routes/previewBrowseRoutes';
import {previewUserRoutes} from './routes/previewUserRoutes';
import {getInitialRoute} from './getInitialRoute';

export function getRouter() {
  const initialRoute = getInitialRoute();

  // The Classic UI is retired: its routes are no longer registered, and Astrum One owns the root of the hash.
  // Classic URLs are mapped to their Astrum One pages by the URL rules of createRouter.
  const menuRoutes = [
    ...loginRoutes,
    ...previewBrowseRoutes,
    ...previewAdminRoutes,
    ...previewUserRoutes,

    // GLOBALART INTERNAL TEST PAGES — standalone (no Settings sidebar) at preview.test* level
    ...(isGlobalArtDevMode() ? globalartInternalTestRoutes : []),
  ];

  const missingRoute = {
    name: ROUTE_NAMES.MISSING_ROUTE,
    url: '404',
    component: MissingRoutePage,
    data: {
      visibilityRequirements: {}
    }
  };

  const router = createRouter({initialRoute, menuRoutes, missingRoute});

  return router;
}
