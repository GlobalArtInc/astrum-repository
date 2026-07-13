

import React, { useState, useEffect, useCallback, useMemo } from "react";

import {
  NxButton,
  NxFontAwesomeIcon,
  NxGlobalHeader2,
} from "@globalart/react-shared-components";
import { faSync } from "@fortawesome/free-solid-svg-icons";
import { ExtJS, handleExtJsUnsavedChanges } from '@globalart/astrum-ui-plugin';
import proLogo from "../../../../art/logos/logo-pro-edition-header.svg";
import proDarkLogo from "../../../../art/logos/logo-pro-edition-header-dark.svg";
import ceLogo from "../../../../art/logos/logo-community-edition-header.svg";
import ceDarkLogo from "../../../../art/logos/logo-community-edition-header-dark.svg";
import coreLogo from "../../../../art/logos/logo-core-edition-header.svg";
import coreDarkLogo from "../../../../art/logos/logo-core-edition-header-dark.svg";
import { useRouter } from '@uirouter/react';
import {Sparkles, ArrowLeft} from 'lucide-react';

import './Globalheader.scss';
import HelpMenu from './HelpMenu';
import LoginAndUserButton from './LogInAndUserProfileMenu';
import Search from './Search';
import SystemStatus from './SystemStatus';
import { ThemeSelector } from '@globalart/astrum-ui-plugin/src/frontend/src';
import { refreshReactPage } from '../../routerConfig/routerUtils';
import ThemeSwitcher from '../ThemeSwitcher/ThemeSwitcher';

export default function GlobalHeader() {
  const COMMUNITY = "COMMUNITY";
  const PRO = "PRO";

  const router = useRouter();
  // Use useStatus().edition which is more reliable than state().getEdition()
  const status = ExtJS.useStatus();
  const edition = status?.edition;

  const refreshTitle = "Refresh";

  const showThemeSelector = window.location.search.includes('showThemeSelector');

  const contextPath = ExtJS.useState(() => ExtJS.state().getValue('astrum-context-path', ''));

  // Use useUser for authentication state
  const user = ExtJS.useUser();
  const isLoggedIn = user?.authenticated === true;
  const isAnonymous = !isLoggedIn;

  // Preview UI access flags
  const [previewUiFlags, setPreviewUiFlags] = useState({ anonymous: false, loggedIn: false });

  useEffect(() => {
    try {
      const state = ExtJS.state();
      if (state) {
        setPreviewUiFlags({
          anonymous: state.getValue('anonymousEnabled', false) ?? false,
          loggedIn: state.getValue('loggedInEnabled', false) ?? false
        });
      }
    } catch (e) {
      // State not available
    }
  }, [user]);

  const canAccessPreviewUi = useMemo(() => {
    if (isAnonymous) {
      return previewUiFlags.anonymous;
    }
    return previewUiFlags.loggedIn;
  }, [isAnonymous, previewUiFlags]);

  // Phase 4.0: UI Toggle between DEFAULT and PREVIEW
  // Use state to track hash changes and trigger re-renders
  const [currentPath, setCurrentPath] = useState(() => window.location.hash.substring(1));
  
  // Listen for hash changes to update the toggle button state
  useEffect(() => {
    const handleHashChange = () => {
      setCurrentPath(window.location.hash.substring(1));
    };
    
    window.addEventListener('hashchange', handleHashChange);
    return () => window.removeEventListener('hashchange', handleHashChange);
  }, []);
  
  const isPreviewUI = currentPath.startsWith('preview');
  
  const toggleUI = useCallback(() => {
    let newPath;
    if (isPreviewUI) {
      // Remove 'preview/' prefix (e.g., 'preview/browse/welcome' → 'browse/welcome')
      newPath = currentPath.replace(/^preview\//, '');
    } else {
      // Add 'preview/' prefix (e.g., 'browse/welcome' → 'preview/browse/welcome')
      // Remove any leading slash first
      const cleanPath = currentPath.replace(/^\//, '');
      newPath = 'preview/' + cleanPath;
    }
    window.location.hash = newPath;
  }, [currentPath, isPreviewUI]);

  return (
      <NxGlobalHeader2
          homeHref={ contextPath || "/"}
          logoProps={getLogoProps()}
          className="as-global-header"
      >
        <Search />

        <SystemStatus />

        <NxButton
            title={refreshTitle}
            aria-label={refreshTitle}
            variant="icon-only"
            onClick={onRefreshClick}
            data-analytics-id="as-global-header-refresh-button"
        >
          <NxFontAwesomeIcon icon={faSync} />
        </NxButton>

        {/* ThemeSwitcher - only shown when IN Preview UI (dark mode not supported in Default UI) */}
        {isPreviewUI && canAccessPreviewUi && <ThemeSwitcher />}

        {canAccessPreviewUi && (
          <NxButton
            variant="tertiary"
            onClick={toggleUI}
            className="ui-toggle-button"
            data-analytics-id="as-global-header-ui-toggle"
            title={isPreviewUI ? "Switch to Classic UI" : "Switch to Astrum One UI"}
          >
            {isPreviewUI ? (
              <>
                <ArrowLeft size={16} />
                <span>Classic UI</span>
              </>
            ) : (
              <>
                <Sparkles size={16} />
                <span>Astrum One UI</span>
              </>
            )}
          </NxButton>
        )}

        <HelpMenu />

        <LoginAndUserButton />

        {showThemeSelector && <ThemeSelector />}
      </NxGlobalHeader2>);

  function onRefreshClick() {
    // Try to get the ExtJS Menu controller
    const menuCtrl =
      window.Ext && Ext.getApplication && Ext.getApplication().getController
        ? Ext.getApplication().getController('Menu')
        : null;

    handleExtJsUnsavedChanges(menuCtrl, () => {
      if (ExtJS.isExtJsRendered()) {
        ExtJS.refresh();
      } else {
        refreshReactPage(router);
      }
    });
  }

  function getLogoProps() {
    return {
      lightPath: getLogo(),
      darkPath: getDarkLogo(), // Needs to be replaced with a true dark mode logo before we can enable dark mode
      altText: 'Astrum Repository'
    }
  }

  function getLogo() {
    return edition === COMMUNITY ? ceLogo
        : edition === PRO ? proLogo
        : coreLogo; // Core or catch all for unknown
  }

  function getDarkLogo() {
    return edition === COMMUNITY ? ceDarkLogo
        : edition === PRO ? proDarkLogo
            : coreDarkLogo; // Core or catch all for unknown
  }

}
