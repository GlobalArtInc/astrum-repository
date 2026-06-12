
import {ExtJS, resolveDefaultLandingRoute, USER_REQUESTED_LEGACY_KEY} from '@globalart/astrum-ui-plugin';
import {ROUTE_NAMES} from './routeNames/routeNames';

/**
 * Determines the initial route based on anonymous access and Preview UI settings.
 *
 * bootstrapFromREST() seeds window.NX.State before this runs, so
 * ExtJS.state().getValue() returns real data. The try/catch is a safety net in
 * case REST bootstrap failed.
 *
 * When anonymous access is enabled, honors the "Default to Astrum One UI" setting
 * (gated by anonymous Preview access and the user_requested_legacy session flag)
 * so anonymous users land directly on the Preview dashboard instead of flashing
 * Classic first.
 */
export function getInitialRoute() {
  try {
    const state = ExtJS.state();
    if (state && typeof state.getValue === 'function') {
      const anonUser = state.getValue('anonymousUsername');
      // Only use ExtJS state if it returned a real value (not the fallback null)
      if (anonUser !== null && anonUser !== undefined) {
        if (!anonUser) {
          return ROUTE_NAMES.LOGIN;
        }
        return resolveDefaultLandingRoute({
          defaultToPreviewUi: state.getValue('defaultToPreviewUi', false) ?? false,
          canAccessPreviewUi: state.getValue('anonymousEnabled', false) ?? false,
          userRequestedLegacy: sessionStorage.getItem(USER_REQUESTED_LEGACY_KEY) === 'true'
        });
      }
    }
  }
  catch {
    // ExtJS not ready
  }
  return ROUTE_NAMES.BROWSE.WELCOME.ROOT;
}
