import {ExtJS, PREVIEW_WELCOME_ROUTE} from '@globalart/astrum-ui-plugin';
import {ROUTE_NAMES} from './routeNames/routeNames';

/**
 * Determines the initial route based on anonymous access.
 *
 * bootstrapFromREST() seeds window.AS.State before this runs, so
 * ExtJS.state().getValue() returns real data. The try/catch is a safety net in
 * case REST bootstrap failed.
 *
 * The Classic UI is retired, so everyone starts on the Astrum One dashboard, anonymous
 * visitors included; only when anonymous access is disabled does the sign-in page come
 * first. A Classic choice saved in this browser by an earlier release no longer counts.
 */
export function getInitialRoute() {
  try {
    const state = ExtJS.state();
    if (state && typeof state.getValue === 'function') {
      const anonUser = state.getValue('anonymousUsername');
      // Only use ExtJS state if it returned a real value (not the fallback null)
      if (anonUser !== null && anonUser !== undefined && !anonUser) {
        return ROUTE_NAMES.LOGIN;
      }
    }
  }
  catch {
    // ExtJS not ready
  }
  return PREVIEW_WELCOME_ROUTE;
}
