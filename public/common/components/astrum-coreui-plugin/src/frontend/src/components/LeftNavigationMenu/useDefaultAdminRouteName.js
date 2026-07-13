

import { isVisible } from '@globalart/astrum-ui-plugin';
import useFilteredRoutes from "../../hooks/useFilteredRoutes";
import { ROUTE_NAMES } from "../../routerConfig/routeNames/routeNames";

/**
 * Hook to check if any route in a section is visible.
 * Uses useFilteredRoutes which internally subscribes to permission change events
 * via ExtJS.useVisiblityWithChanges.
 */
function useIsSectionVisible(section) {
  // useFilteredRoutes internally uses ExtJS.useVisiblityWithChanges, which subscribes to
  // Permissions#changed, State#changed, and State#userchanged events.
  // The isVisible check inside the filter is re-evaluated when these events fire.
  const visibleRoutes = useFilteredRoutes(
    (state) =>
      state.name.startsWith(`${section}.`) &&
      !state?.data?.visibilityRequirements?.ignoreForMenuVisibilityCheck &&
      isVisible(state.data.visibilityRequirements)
  );

  return visibleRoutes.length > 0;
}

export function useDefaultAdminRouteName() {
  const { ADMIN } = ROUTE_NAMES;

  const repoVisible = useIsSectionVisible(ADMIN.REPOSITORY.DIRECTORY);
  const securityVisible = useIsSectionVisible(ADMIN.SECURITY.DIRECTORY);
  const systemVisible = useIsSectionVisible(ADMIN.SYSTEM.DIRECTORY);
  const supportVisible = useIsSectionVisible(ADMIN.SUPPORT.DIRECTORY);

  if (repoVisible) {
    return ADMIN.REPOSITORY.DIRECTORY;
  }
  if (securityVisible) {
    return ADMIN.SECURITY.DIRECTORY;
  }
  if (systemVisible) {
    return ADMIN.SYSTEM.DIRECTORY;
  }
  if (supportVisible) {
    return ADMIN.SUPPORT.DIRECTORY;
  }

  return null;
}

