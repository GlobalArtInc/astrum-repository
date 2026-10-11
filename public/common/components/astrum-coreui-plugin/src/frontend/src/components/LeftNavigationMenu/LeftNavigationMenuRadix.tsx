
import React from 'react';
import { Box, Button, Flex, IconButton, ScrollArea, Separator, Text } from '@radix-ui/themes';
import {
  ChevronDown,
  ChevronUp,
  FileCode,
  FlaskConical,
  FolderOpen,
  HardDriveUpload,
  LayoutDashboard,
  type LucideIcon,
  Repeat,
  Search,
  Settings,
  ShieldAlert,
  Tags,
  X,
} from 'lucide-react';
import { useIsActive, useSref } from '@uirouter/react';
import { NavItemBox, PreviewUIContext, useContextAwareRouteName } from '@globalart/astrum-ui-plugin';
import { ROUTE_NAMES } from '../../routerConfig/routeNames/routeNames';
import { useLeftNavigationMenu, useSearchCollapsibleNav } from './useLeftNavigationMenu';
import { requestSwitchToClassic } from './navigationDrawer';
import LanguageMenuRadix from '../GlobalHeader/LanguageMenuRadix';
import HelpMenuRadix from '../GlobalHeader/HelpMenuRadix';

import './LeftNavigationMenuRadix.scss';
import UIStrings from '../../constants/UIStrings';

const {NAVIGATION, HEADER} = UIStrings.SHELL;

/** Props for {@link SearchNavItem}. */
interface SearchNavItemProps {
  name: string;
  text: string;
  icon?: LucideIcon | null;
  selectedState?: string;
  params?: Record<string, unknown>;
  isCollapsed?: boolean;
}

/**
 * Local NavItem for the Classic UI search submenu.
 *
 * Genuinely component-local variant (Classic UI only): unlike the shared
 * Preview UI `NavItem` in `preview/shared/Navigation/`, it has no
 * tooltip/external-link handling and is always rendered expanded. It is kept
 * local on purpose so no Preview UI primitive leaks into the Classic UI render
 * path. Router/context hooks (`useSref`/`useIsActive`/`useContextAwareRouteName`)
 * are leaf-level bindings, not business logic.
 */
function SearchNavItem({ name, text, icon: Icon, selectedState, params = {}, isCollapsed }: SearchNavItemProps) {
  // Hooks must be called before any conditional returns (React Rules of Hooks)
  const contextAwareName = useContextAwareRouteName(name) || name;
  const contextAwareSelectedState = useContextAwareRouteName(selectedState);

  const sref = useSref(contextAwareName, params);
  const isActive = useIsActive(contextAwareSelectedState || contextAwareName);

  // Defensive check: ensure name is always a valid string for useSref
  if (!name) {
    console.warn('[SearchNavItem] name prop is required but received:', name);
    return null;
  }

  return (
    <a
      href={sref.href}
      onClick={sref.onClick}
      className={`guide-nav-item ${isActive ? 'guide-nav-item--active' : ''}`}
    >
      <Flex align="center" justify={isCollapsed ? 'center' : 'start'} gap="3" px="3" py="2">
        <span className="guide-nav-item__icon">{Icon && <Icon size={18} />}</span>
        {!isCollapsed && <span className="guide-nav-item__text">{text}</span>}
      </Flex>
    </a>
  );
}

/** Props for {@link SearchFormatItem}. */
export interface SearchFormatItemProps {
  routeName: string;
  label: string;
}

/**
 * Search format child item.
 * Visibility is determined by the hook layer via availableFormats from the REST API.
 */
function SearchFormatItem({ routeName, label }: SearchFormatItemProps) {
  return <SearchNavItem name={routeName} text={label} icon={null} params={{ keyword: null }} isCollapsed={false} />;
}

/** Props for {@link SearchCollapsibleNav}. */
export interface SearchCollapsibleNavProps {
  isCollapsed: boolean;
}

/**
 * Collapsible "Search" navigation for the Classic UI.
 *
 * Presentation only: expansion, format filtering, auto-expand and the ExtJS
 * `BrowseableFormats` sync all live in {@link useSearchCollapsibleNav}. The
 * parent entry (chevron + toggle) is a component-local variant of a nav item,
 * so it is not replaced by the shared `NavItem`.
 */
export function SearchCollapsibleNav({ isCollapsed }: SearchCollapsibleNavProps) {
  const {
    isExpanded,
    isParentActive,
    parentHref,
    navigateToSearch,
    handleParentClick,
    customRouteName,
    visibleFormats,
  } = useSearchCollapsibleNav(isCollapsed);

  if (isCollapsed) {
    return (
      <Box>
        <a href={parentHref} onClick={navigateToSearch} className="guide-nav-item">
          <Flex align="center" justify="center" gap="3" px="3" py="2">
            <span className="guide-nav-item__icon">
              <Search size={18} />
            </span>
          </Flex>
        </a>
      </Box>
    );
  }

  return (
    <Box>
      <div className={`search-collapsible ${isExpanded ? 'search-collapsible--expanded' : ''}`}>
        <a
          href={parentHref}
          onClick={handleParentClick}
          aria-expanded={isExpanded}
          className={`guide-nav-item ${isParentActive ? 'guide-nav-item--active' : ''}`}
        >
          <Flex align="center" justify="space-between" gap="3" px="3" py="2">
            <Flex align="center" gap="3">
              <span className="guide-nav-item__icon">
                <Search size={18} />
              </span>
              <span className="guide-nav-item__text">{NAVIGATION.SEARCH}</span>
            </Flex>
            <span className="search-collapsible__chevron">
              {isExpanded ? <ChevronUp size={16} /> : <ChevronDown size={16} />}
            </span>
          </Flex>
        </a>

        {isExpanded && (
          <Box className="search-collapsible__submenu">
            <SearchNavItem
              name={customRouteName}
              text={NAVIGATION.CUSTOM_SEARCH}
              icon={null}
              params={{ keyword: null }}
              isCollapsed={false}
            />
            {visibleFormats.map((format) => (
              <SearchFormatItem key={format.id} routeName={format.routeName} label={format.label} />
            ))}
          </Box>
        )}
      </div>
    </Box>
  );
}

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
  canSwitchToClassic: boolean;
  onSwitchToClassic: () => void;
}

/**
 * Footer of the menu drawer on phones and tablets: what the narrow header has no room for (language, help, the
 * switch to the Classic UI) and the product version.
 */
export function DrawerFooter({ version, canSwitchToClassic, onSwitchToClassic }: DrawerFooterProps) {
  return (
    <Flex direction="column" gap="3" className="guide-sidebar__drawer-footer" data-testid="left-nav-drawer-footer">
      <Flex gap="2" className="guide-sidebar__drawer-tools">
        <LanguageMenuRadix />
        <HelpMenuRadix />
      </Flex>
      {canSwitchToClassic && (
        <Button
          variant="outline"
          color="gray"
          size="3"
          className="guide-sidebar__drawer-classic"
          data-testid="left-nav-switch-classic"
          onClick={onSwitchToClassic}
        >
          <Repeat size={16} aria-hidden />
          {HEADER.SWITCH_TO_CLASSIC}
        </Button>
      )}
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
    isPreviewUI,
    adminInitialRouteName,
    isSettingsVisible,
    showTestHub,
    isProEdition,
    version,
    logo,
    canSwitchToClassic,
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

  const switchToClassic = () => {
    closeDrawer();
    requestSwitchToClassic();
  };

  return (
    <PreviewUIContext.Provider value={isPreviewUI}>
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

              {/* Search - Preview UI: unified single link; Classic UI: collapsible format submenu */}
              {isPreviewUI ? (
                <NavItemBox
                  name={BROWSE.SEARCH.UNIFIED}
                  visibilityRoute={BROWSE.SEARCH.ROOT}
                  text={NAVIGATION.SEARCH}
                  icon={Search}
                  selectedState={BROWSE.SEARCH.ROOT}
                  isCollapsed={isCollapsed}
                />
              ) : (
                <SearchCollapsibleNav isCollapsed={isCollapsed} />
              )}

              {/* Browse */}
              <NavItemBox
                name={isPreviewUI ? 'preview.browse.browse' : BROWSE.BROWSE.ROOT}
                text={NAVIGATION.BROWSE}
                icon={FolderOpen}
                isCollapsed={isCollapsed}
                params={isPreviewUI ? {} : { repo: null }}
              />

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
              {isPreviewUI && (
                <NavItemBox name="preview.browse.api" text={NAVIGATION.API} icon={FileCode} isCollapsed={isCollapsed} />
              )}

              {/* Admin Section */}
              {isSettingsVisible && adminInitialRouteName && (
                <>
                  <Box className="guide-sidebar__divider">
                    <Separator size="4" />
                  </Box>
                  <NavItemBox
                    name={isPreviewUI ? 'preview.settings' : adminInitialRouteName}
                    selectedState={ADMIN.DIRECTORY}
                    text={NAVIGATION.SETTINGS}
                    icon={Settings}
                    isCollapsed={isCollapsed}
                  />
                </>
              )}

              {/* GLOBALART INTERNAL: Test Hub — below Settings, Preview UI only */}
              {showTestHub && (
                <NavItemBox name="preview.test" text="Test Hub" icon={FlaskConical} isCollapsed={isCollapsed} />
              )}
            </Flex>
          </Box>
        </ScrollArea>
        {isDrawer ? (
          <DrawerFooter version={version} canSwitchToClassic={canSwitchToClassic} onSwitchToClassic={switchToClassic} />
        ) : (
          !isCollapsed && <SidebarFooter version={version} />
        )}
      </nav>
    </PreviewUIContext.Provider>
  );
}
