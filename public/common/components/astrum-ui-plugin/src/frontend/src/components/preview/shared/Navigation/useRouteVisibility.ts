
import { useRouter } from '@uirouter/react';
import { useIsVisible } from '../../../../interface/NavigationUtils';
import { useContextAwareRouteName } from './useContextAwareRouteName';

// Sentinel value to indicate route doesn't exist (distinct from undefined visibilityRequirements)
const ROUTE_NOT_FOUND = Symbol('ROUTE_NOT_FOUND');

/**
 * Hook to resolve route state and visibilityRequirements from a route.
 * Returns ROUTE_NOT_FOUND if the route doesn't exist, or the requirements (which may be undefined).
 */
function useRouteVisibilityRequirements(baseName: string) {
  const router = useRouter();
  const contextAwareName = useContextAwareRouteName(baseName) || '';

  try {
    // Try context-aware name first, fall back to base name
    let state = router.stateRegistry.get(contextAwareName);
    if (!state) {
      state = router.stateRegistry.get(baseName);
    }

    if (!state) {
      return ROUTE_NOT_FOUND; // Route doesn't exist
    }

    const data = state?.data || {};
    // May be undefined if route has no visibility requirements (meaning always visible)
    return data.visibilityRequirements;
  } catch {
    return ROUTE_NOT_FOUND;
  }
}

/**
 * Hook to check if a route is visible based on permissions, editions, etc.
 * Uses the reactive useIsVisible hook from astrum-ui-plugin which subscribes to
 * Permissions#changed, State#changed, and State#userchanged events.
 *
 * @param baseName - Base route name to check visibility for
 * @returns boolean indicating if the route is visible
 */
export function useRouteVisibility(baseName: string): boolean {
  const visibilityRequirements = useRouteVisibilityRequirements(baseName);
  const routeExists = visibilityRequirements !== ROUTE_NOT_FOUND;

  // useIsVisible handles permission/state change events reactively
  // If route has no requirements (undefined), useIsVisible returns true
  const isRouteVisible = useIsVisible(routeExists ? visibilityRequirements : undefined);

  // Route must exist to be visible. Missing routes return false (fail closed).
  return routeExists ? isRouteVisible : false;
}
