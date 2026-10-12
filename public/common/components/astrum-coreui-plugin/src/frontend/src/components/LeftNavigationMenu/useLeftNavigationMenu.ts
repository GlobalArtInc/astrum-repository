
import { ExtJS, isGlobalArtDevMode, useSideNavbarOpenState, useTheme } from '@globalart/astrum-ui-plugin';
import { getHeaderLogo } from '../GlobalHeader/headerLogo';
import { useDefaultAdminRouteName } from './useDefaultAdminRouteName';
import { type NavigationDrawer, useNavigationDrawer } from './navigationDrawer';

/**
 * Layer 2 (Integration) hook for the left navigation menu of Astrum One.
 *
 * All business logic for `LeftNavigationMenuRadix` lives here: derived state and side effects.
 * The presentation components in `LeftNavigationMenuRadix.tsx` consume this hook and contain no
 * `useEffect`/`useMemo`/data-fetching of their own.
 */

/** Return value of {@link useLeftNavigationMenu}. */
export interface UseLeftNavigationMenuResult extends NavigationDrawer {
  /** Whether the sidebar is collapsed. */
  isCollapsed: boolean;
  /** First visible admin route, or `null` when no admin section is visible. */
  adminInitialRouteName: string | null;
  /** Whether the "Settings" entry should be shown. */
  isSettingsVisible: boolean;
  /** Whether the GlobalArt-internal "Test Hub" entry should be shown. */
  showTestHub: boolean;
  /** Whether the instance runs the Pro edition (gates Pro-only entries like Tags). */
  isProEdition: boolean;
  /** Version of the running Astrum Repository, or `null` while it is unknown. */
  version: string | null;
  /** Logo shown at the top of the drawer, for the edition and theme. */
  logo: string;
}

/**
 * Integration hook for the top-level left navigation menu.
 *
 * Owns the derived state the presentation component needs: collapsed state,
 * the default admin route, whether the Settings entry should be visible, and
 * the version shown in the footer.
 * The menu is expanded until the user collapses it; that choice is remembered.
 */
export function useLeftNavigationMenu(): UseLeftNavigationMenuResult {
  const [isOpen] = useSideNavbarOpenState(true);
  const status = ExtJS.useStatus?.();
  const version: string | null = status?.version || null;
  const { effectiveTheme } = useTheme();
  const drawer = useNavigationDrawer();
  const adminInitialRouteName = useDefaultAdminRouteName();
  const isSettingsVisible = !!adminInitialRouteName;

  // Test Hub is visible when the GLOBALART_INTERNAL build flag or localStorage flag is set
  const showTestHub = isGlobalArtDevMode();
  const isProEdition = ExtJS.isProEdition?.() ?? false;

  return {
    isCollapsed: !isOpen,
    adminInitialRouteName,
    isSettingsVisible,
    showTestHub,
    isProEdition,
    version,
    logo: getHeaderLogo(status?.edition, effectiveTheme === 'dark'),
    ...drawer,
  };
}
