
import React from 'react';
import { Box, Flex, ScrollArea, Separator, Text } from '@radix-ui/themes';
import {
  ChevronDown,
  ChevronUp,
  FileCode,
  FlaskConical,
  FolderOpen,
  HardDriveUpload,
  LayoutDashboard,
  type LucideIcon,
  Search,
  Settings,
  ShieldAlert,
  Tags,
} from 'lucide-react';
import { useIsActive, useSref } from '@uirouter/react';
import { NavItemBox, PreviewUIContext, useContextAwareRouteName } from '@globalart/astrum-ui-plugin';
import { ROUTE_NAMES } from '../../routerConfig/routeNames/routeNames';
import { useLeftNavigationMenu, useSearchCollapsibleNav } from './useLeftNavigationMenu';

import './LeftNavigationMenuRadix.scss';
import UIStrings from '../../constants/UIStrings';

const {NAVIGATION} = UIStrings.SHELL;

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

/**
 * Modern Left Navigation Menu - SUPER UX with Radix UI.
 *
 * Self-hosted distribution: Dashboard, Search, Browse, Upload, Tags, Malware
 * Risk, API, Settings. Presentation only; derived state comes from
 * {@link useLeftNavigationMenu}.
 */
export default function LeftNavigationMenuRadix() {
  const { BROWSE, ADMIN } = ROUTE_NAMES;
  const {
    isCollapsed,
    isPreviewUI,
    adminInitialRouteName,
    isSettingsVisible,
    showTestHub,
    isProEdition,
    version,
  } = useLeftNavigationMenu();


  return (
    <PreviewUIContext.Provider value={isPreviewUI}>
      <nav
        className={`guide-sidebar ${isCollapsed ? 'guide-sidebar--collapsed' : ''}`}
        aria-label={NAVIGATION.MAIN_MENU}
        data-testid="left-nav"
      >
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
        {!isCollapsed && <SidebarFooter version={version} />}
      </nav>
    </PreviewUIContext.Provider>
  );
}
