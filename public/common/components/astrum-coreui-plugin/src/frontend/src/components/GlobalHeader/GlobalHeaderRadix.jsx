

import React, { useState, useCallback, useMemo } from 'react';
import {Flex, Box, IconButton} from '@radix-ui/themes';
import {Tooltip} from '@globalart/astrum-ui-plugin';
import {RefreshCw, Menu, Search} from 'lucide-react';
import {
  ExtJS,
  hasUnsavedChanges,
  useSideNavbarOpenState,
  PREVIEW_WELCOME_HASH,
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

export default function GlobalHeaderRadix() {
  const router = useRouter();
  const {effectiveTheme} = useTheme();
  
  // Use useStatus().edition which is more reliable than state().getEdition()
  const status = ExtJS.useStatus();
  const edition = status?.edition;
  
  // Context path (static, safe to read any time)
  const contextPath = useMemo(() => {
    try {
      const state = ExtJS.state();
      return state?.getValue('astrum-context-path', '') || '';
    } catch {
      return '';
    }
  }, []);

  // Phones and tablets get a header of icons: the menu is a drawer, the search opens full screen, and the controls
  // that do not fit move into the drawer.
  const isCompact = useIsCompactViewport();
  const isPhoneViewport = useIsPhone();
  const isPhone = isCompact && isPhoneViewport;
  const isDrawerOpen = useNavigationDrawerOpen();
  const [isSearchLayerOpen, setSearchLayerOpen] = useState(false);
  const openSearchLayer = useCallback(() => setSearchLayerOpen(true), []);
  const closeSearchLayer = useCallback(() => setSearchLayerOpen(false), []);

  const onRefreshClick = () => {
    // React dirty state is handled here; ExtJS warnBeforeNavigate has no backing controller (ASTRUM-53775).
    if (hasUnsavedChanges() && !window.confirm('You have unsaved changes. Refreshing will discard them. Continue?')) {
      return;
    }
    refreshReactPage(router);
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
              href={`${contextPath}${PREVIEW_WELCOME_HASH}`}
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
              <SearchRadix />
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

            <ThemeSwitcher className="as-global-header-radix__icon-button" />

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
