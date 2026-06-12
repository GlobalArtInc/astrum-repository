
import {createRouter, isGlobalArtDevMode, MissingRoutePage} from '@globalart/astrum-ui-plugin';
import {ROUTE_NAMES} from './routeNames/routeNames';
import {browseRoutes} from './routes/browseRoutes';
import {adminRoutes} from './routes/adminRoutes';
import {userRoutes} from './routes/userRoutes';
import {loginRoutes} from "./routes/loginRoutes";
import {previewAdminRoutes, globalartInternalTestRoutes} from './routes/previewAdminRoutes';
import {previewBrowseRoutes} from './routes/previewBrowseRoutes';
import {previewUserRoutes} from './routes/previewUserRoutes';
import {getInitialRoute} from './getInitialRoute';

export function getRouter() {
  const initialRoute = getInitialRoute();

  const menuRoutes = [
    // DEFAULT UI (current - unchanged)
    ...browseRoutes,
    ...adminRoutes,
    ...userRoutes,
    ...loginRoutes,

    // PREVIEW UI (new - Phase 4.0)
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
