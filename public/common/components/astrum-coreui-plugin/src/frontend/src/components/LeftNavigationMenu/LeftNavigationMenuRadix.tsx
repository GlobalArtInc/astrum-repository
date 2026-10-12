
import React from 'react';
import { Box, Button, Flex, IconButton, ScrollArea, Separator, Text } from '@radix-ui/themes';
import {
  FileCode,
  FlaskConical,
  FolderOpen,
  HardDriveUpload,
  LayoutDashboard,
  Search,
  Settings,
  ShieldAlert,
  Tags,
  X,
} from 'lucide-react';
import { NavItemBox } from '@globalart/astrum-ui-plugin';
import { ROUTE_NAMES } from '../../routerConfig/routeNames/routeNames';
import { useLeftNavigationMenu } from './useLeftNavigationMenu';
import LanguageMenuRadix from '../GlobalHeader/LanguageMenuRadix';
import HelpMenuRadix from '../GlobalHeader/HelpMenuRadix';

import './LeftNavigationMenuRadix.scss';
import UIStrings from '../../constants/UIStrings';

const {NAVIGATION, HEADER} = UIStrings.SHELL;

/** Props for {@link SidebarFooter}. */
export interface SidebarFooterProps {
  version: string | null;
}

/**
 * Footer of the expanded menu: the product version, in both UIs. The theme is switched from the header.
 */
export function SidebarFooter({ version }: SidebarFooterProps) {
  if (!version) {
    return null;
  }

  return (
    <Box className="guide-sidebar__footer" data-testid="left-nav-footer">
      <Text as="div" size="1" className="guide-sidebar__version" data-testid="left-nav-version">
        {NAVIGATION.PRODUCT_VERSION(version)}
      </Text>
    </Box>
  );
}

/** Props for {@link DrawerFooter}. */
export interface DrawerFooterProps {
  version: string | null;
}

/**
 * Footer of the menu drawer on phones and tablets: what the narrow header has no room for (language and help)
 * and the product version.
 */
export function DrawerFooter({ version }: DrawerFooterProps) {
  return (
    <Flex direction="column" gap="3" className="guide-sidebar__drawer-footer" data-testid="left-nav-drawer-footer">
      <Flex gap="2" className="guide-sidebar__drawer-tools">
        <LanguageMenuRadix />
        <HelpMenuRadix />
      </Flex>
      {version && (
        <Text as="div" size="1" className="guide-sidebar__version">
          {NAVIGATION.PRODUCT_VERSION(version)}
        </Text>
      )}
    </Flex>
  );
}

/**
 * Modern Left Navigation Menu - SUPER UX with Radix UI.
 *
 * Self-hosted distribution: Dashboard, Search, Browse, Upload, Tags, Malware
 * Risk, API, Settings. Presentation only; derived state comes from
 * {@link useLeftNavigationMenu}.
 *
 * On phones and tablets Astrum One shows it as a drawer over the page with a scrim, opened from the header.
 */
export default function LeftNavigationMenuRadix() {
  const { BROWSE, ADMIN } = ROUTE_NAMES;
  const {
    isCollapsed: isRailCollapsed,
    adminInitialRouteName,
    isSettingsVisible,
    showTestHub,
    isProEdition,
    version,
    logo,
    isDrawer,
    isDrawerOpen,
    closeDrawer,
    onDrawerClick,
    drawerRef,
  } = useLeftNavigationMenu();
  const isCollapsed = isRailCollapsed && !isDrawer;

  let className = 'guide-sidebar';
  if (isDrawer) {
    className += isDrawerOpen ? ' guide-sidebar--drawer guide-sidebar--drawer-open' : ' guide-sidebar--drawer';
  }
  else if (isCollapsed) {
    className += ' guide-sidebar--collapsed';
  }

  return (
    <>
      {isDrawerOpen && (
        <div className="guide-sidebar__scrim" aria-hidden="true" data-testid="left-nav-scrim" onClick={closeDrawer} />
      )}
      <nav
        ref={drawerRef}
        id="as-main-menu"
        className={className}
        aria-label={NAVIGATION.MAIN_MENU}
        data-testid="left-nav"
        onClick={isDrawer ? onDrawerClick : undefined}
      >
        {isDrawer && (
          <Flex align="center" justify="between" gap="2" className="guide-sidebar__drawer-header">
            <img src={logo} alt={HEADER.LOGO_ALT} className="guide-sidebar__drawer-logo" />
            <IconButton
              variant="ghost"
              color="gray"
              size="3"
              className="guide-sidebar__drawer-close"
              aria-label={HEADER.CLOSE_MENU}
              data-drawer-autofocus
              onClick={closeDrawer}
            >
              <X size={20} aria-hidden />
            </IconButton>
          </Flex>
        )}
        <ScrollArea className="guide-sidebar__nav" scrollbars="vertical">
          <Box className="guide-sidebar__nav-content">
            <Flex direction="column" gap="1">
              {/* Dashboard */}
              <NavItemBox name={BROWSE.WELCOME.ROOT} text={NAVIGATION.DASHBOARD} icon={LayoutDashboard} isCollapsed={isCollapsed} />

              {/* Search */}
              <NavItemBox
                name={BROWSE.SEARCH.UNIFIED}
                visibilityRoute={BROWSE.SEARCH.ROOT}
                text={NAVIGATION.SEARCH}
                icon={Search}
                selectedState={BROWSE.SEARCH.ROOT}
                isCollapsed={isCollapsed}
              />

              {/* Browse */}
              <NavItemBox name={BROWSE.BROWSE.ROOT} text={NAVIGATION.BROWSE} icon={FolderOpen} isCollapsed={isCollapsed} />

              {/* Upload */}
              <NavItemBox
                name={BROWSE.UPLOAD.LIST}
                visibilityRoute={BROWSE.UPLOAD.ROOT}
                text={NAVIGATION.UPLOAD}
                icon={HardDriveUpload}
                selectedState={BROWSE.UPLOAD.ROOT}
                isCollapsed={isCollapsed}
              />

              {/* Tags — Pro feature: hidden in CE mode */}
              {isProEdition && (
                <NavItemBox
                  name={BROWSE.TAGS.ROOT}
                  text={NAVIGATION.TAGS}
                  icon={Tags}
                  isCollapsed={isCollapsed}
                  params={{ itemId: null }}
                />
              )}


              {/* API */}
              <NavItemBox name={BROWSE.API.ROOT} text={NAVIGATION.API} icon={FileCode} isCollapsed={isCollapsed} />

              {/* Admin Section */}
              {isSettingsVisible && adminInitialRouteName && (
                <>
                  <Box className="guide-sidebar__divider">
                    <Separator size="4" />
                  </Box>
                  <NavItemBox
                    name={ADMIN.SETTINGS}
                    selectedState={ADMIN.DIRECTORY}
                    text={NAVIGATION.SETTINGS}
                    icon={Settings}
                    isCollapsed={isCollapsed}
                  />
                </>
              )}

              {/* GLOBALART INTERNAL: Test Hub — below Settings */}
              {showTestHub && (
                <NavItemBox name="preview.test" text="Test Hub" icon={FlaskConical} isCollapsed={isCollapsed} />
              )}
            </Flex>
          </Box>
        </ScrollArea>
        {isDrawer ? (
          <DrawerFooter version={version} />
        ) : (
          !isCollapsed && <SidebarFooter version={version} />
        )}
      </nav>
    </>
  );
}
