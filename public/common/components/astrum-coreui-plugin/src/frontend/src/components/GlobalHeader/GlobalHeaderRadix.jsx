

import React, { useState, useEffect, useCallback, useMemo, useRef } from 'react';
import {Flex, Box, IconButton, Button, Dialog, Text, Popover, TextArea, Heading} from '@radix-ui/themes';
import {Tooltip} from '@globalart/astrum-ui-plugin';
import {RefreshCw, Repeat, Menu} from 'lucide-react';
import {ExtJS, handleExtJsUnsavedChanges, hasUnsavedChanges, restClient, useSideNavbarOpenState} from '@globalart/astrum-ui-plugin';
import {useRouter} from '@uirouter/react';
import {useTheme} from '../../contexts/ThemeContext';

import proLogo from "../../../../art/logos/logo-pro-edition-header.svg";
import proDarkLogo from "../../../../art/logos/logo-pro-edition-header-dark.svg";
import ceLogo from "../../../../art/logos/logo-community-edition-header.svg";
import ceDarkLogo from "../../../../art/logos/logo-community-edition-header-dark.svg";
import coreLogo from "../../../../art/logos/logo-core-edition-header.svg";
import coreDarkLogo from "../../../../art/logos/logo-core-edition-header-dark.svg";

import SearchRadix from './SearchRadix';
import SystemStatusRadix from './SystemStatusRadix';
import ThemeSwitcher from '../ThemeSwitcher/ThemeSwitcher';
import HelpMenuRadix from './HelpMenuRadix';
import LanguageMenuRadix from './LanguageMenuRadix';
import LoginAndUserButtonRadix from './LoginAndUserButtonRadix';
import {refreshReactPage} from '../../routerConfig/routerUtils';
import {getHeritageEquivalent} from './previewHeritageNavigation';

import './GlobalHeaderRadix.scss';
import UIStrings from '../../constants/UIStrings';

const COMMUNITY = "COMMUNITY";
const PRO = "PRO";
const {HEADER} = UIStrings.SHELL;
const MAX_FEEDBACK_LENGTH = 500;

export default function GlobalHeaderRadix() {
  const router = useRouter();
  const {effectiveTheme, setTheme} = useTheme();
  
  // Use useStatus().edition which is more reliable than state().getEdition()
  const status = ExtJS.useStatus();
  const edition = status?.edition;
  const version = status?.version;
  
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
  const [isDefaultToPreviewUi, setDefaultToPreviewUi] = useState(false);
  const [isDisableLegacyUi, setDisableLegacyUi] = useState(false);
  const [isDisableSwitchFeedback, setDisableSwitchFeedback] = useState(false);
  const [heritageUnavailableOpen, setHeritageUnavailableOpen] = useState(false);
  const [uiSwitchPopoverOpen, setUiSwitchPopoverOpen] = useState(false);
  const [classicUiFeedback, setClassicUiFeedback] = useState('');
  const useClassicUiButtonRef = useRef(null);

  const isFeedbackTooLong = classicUiFeedback.length > MAX_FEEDBACK_LENGTH;

  useEffect(() => {
    try {
      const state = ExtJS.state();
      if (!state) return;

      const disableLegacyUi = state.getValue('disableLegacyUi', false) ?? false;
      setDisableLegacyUi(disableLegacyUi);
      setDefaultToPreviewUi(state.getValue('defaultToPreviewUi', false) ?? false);
      setDisableSwitchFeedback(state.getValue('disableSwitchFeedback', false) ?? false);

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
  
  // Redirect logic for rollout controls (Sprint 17)
  useEffect(() => {
    if (!canAccessPreviewUi) return;

    const cleanPath = currentPath.replace(/^\//, '');
    const landingPages = ['', '/', 'browse/welcome'];
    const isLandingPage = landingPages.includes(cleanPath);

    // 1. HARD KILL SWITCH: Check disableLegacyUi first
    if (isDisableLegacyUi && !isPreviewUI) {
      // Force redirect to preview equivalent or welcome
      let newPath = 'preview/browse/welcome';

      // Try to find equivalent mapping if it's not a landing page
      if (!isLandingPage) {
        if (cleanPath.startsWith('admin/')) {
          newPath = 'preview/' + cleanPath;
        } else if (cleanPath === 'user/account' || cleanPath.startsWith('user/account/')) {
          // User Account is a standalone page — not inside the Settings layout
          newPath = 'preview/user/account';
        } else if (cleanPath === 'user/NuGetApiToken' || cleanPath.startsWith('user/NuGetApiToken')) {
          newPath = 'preview/user/nugetapitoken';
        } else if (cleanPath === 'user/usertoken' || cleanPath.startsWith('user/usertoken')) {
          newPath = 'preview/user/usertoken';
        } else if (cleanPath.startsWith('user/')) {
          newPath = 'preview/user/' + cleanPath.replace('user/', '');
        } else {
          newPath = 'preview/' + cleanPath;
        }
      }

      // Use hash-based navigation for Preview UI
      const hashPath = `#${newPath}`;
      window.location.hash = hashPath;
      return;
    }

    // 2. SOFT DEFAULT: Check defaultToPreviewUi second
    if (isDefaultToPreviewUi && !isPreviewUI && isLandingPage) {
      const userRequestedLegacy = sessionStorage.getItem('user_requested_legacy') === 'true';
      if (!userRequestedLegacy) {
        // Use hash-based navigation for Preview UI
        const hashPath = '#preview/browse/welcome';
        window.location.hash = hashPath;
      }
    }
  }, [canAccessPreviewUi, isDisableLegacyUi, isDefaultToPreviewUi, isPreviewUI, currentPath]);

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
  
  const switchToClassicUI = useCallback(() => {
    const heritagePath = getHeritageEquivalent(currentPath);
    if (heritagePath === null) {
      setHeritageUnavailableOpen(true);
      setUiSwitchPopoverOpen(false);
      return;
    }

    if (!isDisableSwitchFeedback) {
      restClient.post('/service/rest/internal/ui/switch-feedback', {
        edition: edition ?? 'Unknown',
        version: version ?? 'Unknown',
        feedback: classicUiFeedback.trim().slice(0, 500),
      }).catch(() => {});
    }

    sessionStorage.setItem('user_requested_legacy', 'true');
    setTheme('light');
    setUiSwitchPopoverOpen(false);
    setClassicUiFeedback('');
    window.location.hash = heritagePath;
  }, [currentPath, classicUiFeedback, isDisableSwitchFeedback, setTheme, edition, version]);

  // Always land on the Preview dashboard rather than attempting to map the
  // current Classic route to its Preview equivalent. heritageToPreviewPath has
  // a generic fallback that prefixes "preview/" to unmapped paths, which can
  // produce routes that don't exist in Preview UI and result in 404s.
  const switchToAstrumOneUI = useCallback(() => {
    window.location.hash = '#preview/browse/welcome';
    sessionStorage.removeItem('user_requested_legacy');
  }, []);

  const handleUiSwitchPopoverOpenChange = useCallback((open) => {
    setUiSwitchPopoverOpen(open);
    if (!open) {
      setClassicUiFeedback('');
    }
  }, []);

  const handlePopoverOpenAutoFocus = useCallback((e) => {
    e.preventDefault();
    // Use requestAnimationFrame so the button is mounted when we focus
    requestAnimationFrame(() => {
      useClassicUiButtonRef.current?.focus();
    });
  }, []);

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
    const isDark = effectiveTheme === 'dark';
    
    if (edition === COMMUNITY) {
      return isDark ? ceDarkLogo : ceLogo;
    } else if (edition === PRO) {
      return isDark ? proDarkLogo : proLogo;
    } else {
      return isDark ? coreDarkLogo : coreLogo;
    }
  }


  const [isSidebarOpen] = useSideNavbarOpenState(false);
  const onSidebarToggle = useCallback(() => {
    window.dispatchEvent(new CustomEvent('as-sidebar-toggle'));
  }, []);

  return (
    <div className="as-global-header-radix as-global-header-2" data-testid="global-header">
      <Box asChild className="as-global-header-radix__sticky-bar">
        <Flex align="center" justify="between" gap="4" px="4" className="as-global-header-radix__row">
          {/* Left: Sidebar toggle + Logo */}
          <Flex align="center" gap="4" className="as-global-header-radix__side">
            <Tooltip content={isSidebarOpen ? HEADER.COLLAPSE_SIDEBAR : HEADER.EXPAND_SIDEBAR}>
              <IconButton
                variant="outline"
                size="2"
                color="gray"
                aria-label={isSidebarOpen ? HEADER.COLLAPSE_SIDEBAR : HEADER.EXPAND_SIDEBAR}
                onClick={onSidebarToggle}
              >
                <Menu size={16} />
              </IconButton>
            </Tooltip>
            <Box asChild>
              <a href={`${contextPath}#${isPreviewUI ? 'preview/browse/welcome' : 'browse/welcome'}`} title={HEADER.HOME}>
                <img
                  src={getLogo()}
                  alt={HEADER.LOGO_ALT}
                  className="as-global-header-radix__logo"
                />
              </a>
            </Box>
          </Flex>

          {/* Center: Search Bar */}
          <Box className="as-global-header-radix__search">
            <SearchRadix isPreviewUI={isPreviewUI} />
          </Box>

          {/* Right: Utility buttons */}
          <Flex align="center" gap="3" className="as-global-header-radix__side">
            {canAccessPreviewUi && !isDisableLegacyUi && (
              isPreviewUI ? (
                isDisableSwitchFeedback ? (
                  <Tooltip content={HEADER.SWITCH_TO_CLASSIC}>
                    <Button
                      variant="soft"
                      color="green"
                      className="as-global-header-radix__clickable"
                      aria-label={HEADER.SWITCH_TO_CLASSIC}
                      title={HEADER.SWITCH_TO_CLASSIC}
                      data-testid="ui-toggle-button"
                      data-analytics-id="as-global-switch-classic"
                      onClick={switchToClassicUI}
                    >
                      <Flex align="center" gap="2">
                        <Repeat size={16} aria-hidden />
                        {HEADER.SWITCH_TO_CLASSIC}
                      </Flex>
                    </Button>
                  </Tooltip>
                ) : (
                  <Popover.Root open={uiSwitchPopoverOpen} onOpenChange={handleUiSwitchPopoverOpenChange}>
                    <Tooltip content={uiSwitchPopoverOpen ? undefined : HEADER.SWITCH_TO_CLASSIC}>
                      <Popover.Trigger asChild>
                        <Button
                          variant="soft"
                          color="green"
                          className="as-global-header-radix__clickable"
                          aria-label={HEADER.SWITCH_TO_CLASSIC}
                          aria-haspopup="dialog"
                          title={HEADER.SWITCH_TO_CLASSIC}
                          data-testid="ui-toggle-button"
                          data-analytics-id="as-global-switch-classic"
                        >
                          <Flex align="center" gap="2">
                            <Repeat size={16} aria-hidden />
                            {HEADER.SWITCH_TO_CLASSIC}
                          </Flex>
                        </Button>
                      </Popover.Trigger>
                    </Tooltip>
                    <Popover.Content
                      width="360px"
                      aria-labelledby="ui-switch-popover-title"
                      aria-describedby="ui-switch-popover-desc"
                      onOpenAutoFocus={handlePopoverOpenAutoFocus}
                    >
                      <Flex direction="column" gap="4">
                        <Box>
                          <Heading id="ui-switch-popover-title" size="3" mb="2">{HEADER.SWITCH_FEEDBACK.TITLE}</Heading>
                          <Text id="ui-switch-popover-desc" size="2" color="gray" as="p">
                            {HEADER.SWITCH_FEEDBACK.DESCRIPTION}
                          </Text>
                        </Box>
                        <Flex direction="column" gap="3">
                          <Flex direction="column" gap="1">
                            <TextArea
                              id="ui-feedback-textarea"
                              placeholder={HEADER.SWITCH_FEEDBACK.PLACEHOLDER}
                              value={classicUiFeedback}
                              onChange={(e) => setClassicUiFeedback(e.target.value)}
                              rows={3}
                              aria-label={HEADER.SWITCH_FEEDBACK.LABEL}
                              maxLength={MAX_FEEDBACK_LENGTH + 1}
                              color={isFeedbackTooLong ? 'red' : undefined}
                            />
                            {isFeedbackTooLong && (
                              <Text size="1" color="red">
                                {HEADER.SWITCH_FEEDBACK.TOO_LONG(MAX_FEEDBACK_LENGTH, classicUiFeedback.length - MAX_FEEDBACK_LENGTH)}
                              </Text>
                            )}
                          </Flex>
                          <Button
                            ref={useClassicUiButtonRef}
                            variant="soft"
                            color="green"
                            size="2"
                            className={isFeedbackTooLong ? 'as-global-header-radix__clickable--disabled' : 'as-global-header-radix__clickable'}
                            onClick={switchToClassicUI}
                            disabled={isFeedbackTooLong}
                            aria-label={HEADER.SWITCH_FEEDBACK.USE_CLASSIC}
                            title={HEADER.SWITCH_FEEDBACK.USE_CLASSIC}
                            data-analytics-id="as-dialog-use-classic"
                          >
                            {HEADER.SWITCH_FEEDBACK.USE_CLASSIC}
                          </Button>
                        </Flex>
                      </Flex>
                    </Popover.Content>
                  </Popover.Root>
                )
              ) : (
                <Tooltip content={HEADER.SWITCH_TO_PREVIEW}>
                  <Button
                    variant="soft"
                    color="green"
                    onClick={switchToAstrumOneUI}
                    className="as-global-header-radix__clickable"
                    aria-label={HEADER.SWITCH_TO_PREVIEW}
                    title={HEADER.SWITCH_TO_PREVIEW}
                    data-testid="ui-toggle-button"
                    data-analytics-id="as-global-switch-preview"
                  >
                    <Flex align="center" gap="2">
                      <Repeat size={16} aria-hidden />
                      {HEADER.SWITCH_TO_PREVIEW}
                    </Flex>
                  </Button>
                </Tooltip>
              )
            )}

            {isPreviewUI && <ThemeSwitcher />}

            <SystemStatusRadix />

            <Tooltip content={HEADER.REFRESH}>
              <IconButton
                variant="outline"
                size="2"
                color="gray"
                aria-label={HEADER.REFRESH}
                data-analytics-id="astrum-header-refresh"
                onClick={onRefreshClick}
              >
                <RefreshCw size={16} />
              </IconButton>
            </Tooltip>

            <LanguageMenuRadix />

            <HelpMenuRadix />

            <LoginAndUserButtonRadix />
          </Flex>
        </Flex>
      </Box>

      {/* Classic UI unavailable – shown when current Preview route has no Classic equivalent */}
      <Dialog.Root open={heritageUnavailableOpen} onOpenChange={(open) => !open && setHeritageUnavailableOpen(false)}>
        <Dialog.Content maxWidth="450px" aria-describedby={undefined}>
          <Dialog.Title>{HEADER.CLASSIC_UNAVAILABLE.TITLE}</Dialog.Title>
          <Text size="2" color="gray" as="p" mt="2">
            {HEADER.CLASSIC_UNAVAILABLE.TEXT}
          </Text>
          <Flex gap="3" mt="4" justify="end">
            <Dialog.Close>
              <Button variant="soft" color="gray" size="2">{HEADER.CLASSIC_UNAVAILABLE.STAY}</Button>
            </Dialog.Close>
            <Button
              variant="solid"
              color="blue"
              size="2"
              onClick={() => {
                setHeritageUnavailableOpen(false);
                sessionStorage.setItem('user_requested_legacy', 'true');
                setTheme('light');
                window.location.hash = 'browse/welcome';
              }}
            >
              {HEADER.CLASSIC_UNAVAILABLE.GO}
            </Button>
          </Flex>
        </Dialog.Content>
      </Dialog.Root>
    </div>
  );
}
