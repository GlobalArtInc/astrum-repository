

import React, { useState, useEffect, useCallback, useMemo, useRef } from 'react';
import {Flex, Box, IconButton} from '@radix-ui/themes';
import {Tooltip} from '@globalart/astrum-ui-plugin';
import {RefreshCw, Menu, Search} from 'lucide-react';
import {
  ExtJS,
  handleExtJsUnsavedChanges,
  hasUnsavedChanges,
  useSideNavbarOpenState,
  heritageToPreviewPath,
  setLastUsedUi,
  useIsCompactViewport,
  useIsPhone
} from '@globalart/astrum-ui-plugin';
import {useRouter} from '@uirouter/react';
import {useTheme} from '../../contexts/ThemeContext';

import {getHeaderLogo, HEADER_MARK} from './headerLogo';

import SearchRadix from './SearchRadix';
import SearchLayer from './SearchLayer';
import SystemStatusRadix from './SystemStatusRadix';
import ThemeSwitcher from '../ThemeSwitcher/ThemeSwitcher';
import HelpMenuRadix from './HelpMenuRadix';
import LanguageMenuRadix from './LanguageMenuRadix';
import LoginAndUserButtonRadix from './LoginAndUserButtonRadix';
import {refreshReactPage} from '../../routerConfig/routerUtils';
import {setNavigationDrawerOpen, useNavigationDrawerOpen} from '../LeftNavigationMenu/navigationDrawer';

import './GlobalHeaderRadix.scss';
import UIStrings from '../../constants/UIStrings';

const {HEADER} = UIStrings.SHELL;

/**
 * Hash paths outside the Classic UI that the Classic redirect must leave alone: the sign-in page (an
 * anonymous visitor sent there would otherwise bounce between it and the dashboard) and the 404 page.
 */
const SHARED_PATHS = ['login', '404'];

export default function GlobalHeaderRadix() {
  const router = useRouter();
  const {effectiveTheme, setTheme} = useTheme();
  
  // Use useStatus().edition which is more reliable than state().getEdition()
  const status = ExtJS.useStatus();
  const edition = status?.edition;
  
  // Use useUser for authentication state - this is a stable hook
  const user = ExtJS.useUser();
  const isLoggedIn = user?.authenticated === true;
  const isAnonymous = !isLoggedIn;

  // Context path (static, safe to read any time)
  const contextPath = useMemo(() => {
    try {
      const state = ExtJS.state();
      return state?.getValue('astrum-context-path', '') || '';
    } catch {
      return '';
    }
  }, []);

  // Cached access flag — single effect handles all auth states.
  //
  // Rules:
  //   Logged-in:  grant when ExtJS flags confirm access; NEVER revoke
  //               while isLoggedIn is true (immune to flag flaps).
  //   Anonymous:  re-evaluate every time (no caching — anonymous state
  //               is stable, no login race condition).
  //   Logout→Anon: compute anonymous access in the SAME effect that
  //               handles logout so there's one setState, no flash.
  const [canAccessPreviewUi, setCanAccessPreviewUi] = useState(false);
  useEffect(() => {
    try {
      const state = ExtJS.state();
      if (!state) return;

      if (isLoggedIn) {
        // Logged-in path: only upgrade false→true, never downgrade.
        if (!canAccessPreviewUi) {
          const loggedInEnabled = state.getValue('loggedInEnabled', false) ?? false;
          if (loggedInEnabled) {
            setCanAccessPreviewUi(true);
          }
        }
      } else {
        // Anonymous path (includes fresh page load AND logout→anon).
        // Always re-evaluate — no caching for anonymous.
        const anonymousEnabled = state.getValue('anonymousEnabled', false) ?? false;
        setCanAccessPreviewUi(anonymousEnabled);
      }
    } catch {
      if (!isLoggedIn) {
        setCanAccessPreviewUi(false);
      }
    }
  }, [isLoggedIn, user, canAccessPreviewUi]);

  // Phase 4.0: UI Toggle between DEFAULT and PREVIEW
  // Both UIs use hash-based routing (e.g., #browse/welcome or #preview/browse/welcome)
  const [currentPath, setCurrentPath] = useState(() => {
    const hash = window.location.hash.substring(1);
    return hash || '';
  });

  // Listen for hash changes (entire app uses hash-based routing)
  useEffect(() => {
    const handleLocationChange = () => {
      const hash = window.location.hash.substring(1);
      setCurrentPath(hash || '');
    };

    window.addEventListener('hashchange', handleLocationChange);
    window.addEventListener('popstate', handleLocationChange);
    return () => {
      window.removeEventListener('hashchange', handleLocationChange);
      window.removeEventListener('popstate', handleLocationChange);
    };
  }, []);

  const isPreviewUI = currentPath.startsWith('preview');

  // Phones and tablets get a header of icons in Astrum One: the menu is a drawer, the search opens full screen, and
  // the controls that do not fit move into the drawer. The Classic UI keeps its desktop header at any width.
  const isCompactViewport = useIsCompactViewport();
  const isPhoneViewport = useIsPhone();
  const isCompact = isPreviewUI && isCompactViewport;
  const isPhone = isCompact && isPhoneViewport;
  const isDrawerOpen = useNavigationDrawerOpen();
  const [isSearchLayerOpen, setSearchLayerOpen] = useState(false);
  const openSearchLayer = useCallback(() => setSearchLayerOpen(true), []);
  const closeSearchLayer = useCallback(() => setSearchLayerOpen(false), []);
  
  // The Classic UI is retired: every Classic URL, including old bookmarks and links in emails, opens the
  // matching Astrum One page, or the dashboard when Astrum One has no such page.
  useEffect(() => {
    if (!canAccessPreviewUi || isPreviewUI) return;
    const cleanPath = currentPath.replace(/^\//, '');
    if (SHARED_PATHS.includes(cleanPath.split('?')[0])) return;
    window.location.hash = `#${heritageToPreviewPath(cleanPath)}`;
  }, [canAccessPreviewUi, isPreviewUI, currentPath]);

  useEffect(() => {
    if (isPreviewUI && canAccessPreviewUi) {
      setLastUsedUi('preview');
    }
  }, [isPreviewUI, canAccessPreviewUi]);

  // Track whether access was ever granted so we don't redirect on
  // the very first render (before the grant effect has run).
  const accessEverGranted = useRef(false);
  if (canAccessPreviewUi) {
    accessEverGranted.current = true;
  }

  // Redirect out of Preview UI only when access transitions from
  // granted → revoked while the user is still authenticated.
  // On logout, useRedirectOnLogout handles the redirect to login — we must not
  // interfere by stripping preview/ (which creates invalid classic routes like #settings).
  useEffect(() => {
    if (isPreviewUI && !canAccessPreviewUi && accessEverGranted.current && isLoggedIn) {
      setTheme('light');
      const defaultPath = currentPath.replace(/^preview\//, '') || 'browse/welcome';
      window.location.hash = defaultPath;
    }
  }, [isPreviewUI, canAccessPreviewUi, currentPath, setTheme, isLoggedIn]);
  
  const onRefreshClick = () => {
    if (!ExtJS.isExtJsRendered()) {
      // Preview UI: ExtJS warnBeforeNavigate crashes when there is no backing ExtJS
      // controller, so handle React dirty state directly here (ASTRUM-53775).
      if (hasUnsavedChanges() && !window.confirm('You have unsaved changes. Refreshing will discard them. Continue?')) {
        return;
      }
      refreshReactPage(router);
      return;
    }

    const menuCtrl =
      window.Ext && Ext.getApplication && Ext.getApplication().getController
        ? Ext.getApplication().getController('Menu')
        : null;

    handleExtJsUnsavedChanges(menuCtrl, () => {
      ExtJS.refresh();
    });
  };

  function getLogo() {
    return isPhone ? HEADER_MARK : getHeaderLogo(edition, effectiveTheme === 'dark');
  }


  const [isSidebarOpen] = useSideNavbarOpenState(true);
  const onSidebarToggle = useCallback(() => {
    window.dispatchEvent(new CustomEvent('as-sidebar-toggle'));
  }, []);
  const onDrawerToggle = useCallback(() => setNavigationDrawerOpen(!isDrawerOpen), [isDrawerOpen]);

  let menuLabel = isSidebarOpen ? HEADER.COLLAPSE_SIDEBAR : HEADER.EXPAND_SIDEBAR;
  if (isCompact) {
    menuLabel = isDrawerOpen ? HEADER.CLOSE_MENU : HEADER.OPEN_MENU;
  }

  let headerClassName = 'as-global-header-radix as-global-header-2';
  if (isCompact) {
    headerClassName += isPhone ? ' as-global-header-radix--compact as-global-header-radix--phone' : ' as-global-header-radix--compact';
  }

  return (
    <div className={headerClassName} data-testid="global-header">
      <Box asChild className="as-global-header-radix__sticky-bar">
        <Flex align="center" justify="between" gap="4" className="as-global-header-radix__row">
          {/* Left: Sidebar toggle + Logo */}
          <Flex align="center" gap="3" className="as-global-header-radix__side">
            <Tooltip content={menuLabel}>
              <IconButton
                variant="outline"
                size="3"
                color="gray"
                className="as-global-header-radix__icon-button"
                aria-label={menuLabel}
                aria-expanded={isCompact ? isDrawerOpen : isSidebarOpen}
                aria-controls={isCompact ? 'as-main-menu' : undefined}
                data-testid="menu-toggle"
                onClick={isCompact ? onDrawerToggle : onSidebarToggle}
              >
                <Menu size={18} aria-hidden />
              </IconButton>
            </Tooltip>
            <a
              href={`${contextPath}#${isPreviewUI ? 'preview/browse/welcome' : 'browse/welcome'}`}
              title={HEADER.HOME}
              className="as-global-header-radix__home"
            >
              <img
                src={getLogo()}
                alt={HEADER.LOGO_ALT}
                className="as-global-header-radix__logo"
              />
            </a>
          </Flex>

          {/* Center: Search Bar */}
          {!isCompact && (
            <Box className="as-global-header-radix__search">
              <SearchRadix isPreviewUI={isPreviewUI} />
            </Box>
          )}

          {/* Right: Utility buttons */}
          <Flex align="center" gap="2" className="as-global-header-radix__side as-global-header-radix__actions">
            {isCompact && (
              <IconButton
                variant="outline"
                size="3"
                color="gray"
                className="as-global-header-radix__icon-button"
                aria-label={HEADER.OPEN_SEARCH}
                aria-haspopup="dialog"
                aria-expanded={isSearchLayerOpen}
                data-testid="search-layer-open"
                onClick={openSearchLayer}
              >
                <Search size={18} aria-hidden />
              </IconButton>
            )}

            {!isCompact && (
              <Tooltip content={HEADER.REFRESH}>
                <IconButton
                  variant="outline"
                  size="3"
                  color="gray"
                  className="as-global-header-radix__icon-button"
                  aria-label={HEADER.REFRESH}
                  data-analytics-id="astrum-header-refresh"
                  onClick={onRefreshClick}
                >
                  <RefreshCw size={18} aria-hidden />
                </IconButton>
              </Tooltip>
            )}

            {isPreviewUI && <ThemeSwitcher className="as-global-header-radix__icon-button" />}

            {!isPhone && <LanguageMenuRadix />}

            <SystemStatusRadix />

            {!isPhone && <HelpMenuRadix />}

            <LoginAndUserButtonRadix />
          </Flex>
        </Flex>
      </Box>

      {isCompact && isSearchLayerOpen && <SearchLayer onClose={closeSearchLayer} />}

    </div>
  );
}
