import { isVisible } from '@globalart/astrum-ui-plugin';
import useFilteredRoutes from "../../hooks/useFilteredRoutes";
import { ROUTE_NAMES } from "../../routerConfig/routeNames/routeNames";

/**
 * Hook to check if any page of an Astrum One settings section is visible.
 * Uses useFilteredRoutes which internally subscribes to permission change events
 * via ExtJS.useVisiblityWithChanges.
 *
 * Only the pages of the section count, the states right under it: their requirements guard everything below
 * them, which declares none of its own.
 */
function useIsSectionVisible(section) {
  const depth = section.split('.').length + 1;
  // useFilteredRoutes internally uses ExtJS.useVisiblityWithChanges, which subscribes to
  // Permissions#changed, State#changed, and State#userchanged events.
  // The isVisible check inside the filter is re-evaluated when these events fire.
  const visibleRoutes = useFilteredRoutes((state) => {
    const requirements = state.data?.visibilityRequirements;
    return state.name.startsWith(`${section}.`) &&
      state.name.split('.').length === depth &&
      Boolean(requirements) &&
      !requirements.ignoreForMenuVisibilityCheck &&
      isVisible(requirements);
  });

  return visibleRoutes.length > 0;
}

/**
 * The first Astrum One settings section the user may open, or null when there is none.
 */
export function useDefaultAdminRouteName() {
  const { ADMIN } = ROUTE_NAMES;

  const repositorySection = `preview.${ADMIN.REPOSITORY.DIRECTORY}`;
  const securitySection = `preview.${ADMIN.SECURITY.DIRECTORY}`;
  const systemSection = `preview.${ADMIN.SYSTEM.DIRECTORY}`;
  const supportSection = `preview.${ADMIN.SUPPORT.DIRECTORY}`;

  const repoVisible = useIsSectionVisible(repositorySection);
  const securityVisible = useIsSectionVisible(securitySection);
  const systemVisible = useIsSectionVisible(systemSection);
  const supportVisible = useIsSectionVisible(supportSection);

  if (repoVisible) {
    return repositorySection;
  }
  if (securityVisible) {
    return securitySection;
  }
  if (systemVisible) {
    return systemSection;
  }
  if (supportVisible) {
    return supportSection;
  }

  return null;
}
