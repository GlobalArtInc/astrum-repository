
import React, { Suspense, useEffect, useLayoutEffect } from 'react';
import DOMPurify from 'dompurify';
import { createRoot } from 'react-dom/client';
import { UIRouter, UIView, useRouter } from '@uirouter/react';
import { ExtJS, UnsavedChangesModal, PreviewUnsavedDialog, RouteLoadingFallback, ToastProvider, useToast, bootstrapFromREST, SessionExpiryModal, useSessionExpiry, TooltipContainerProvider as NuiTooltipContainerProvider, OnboardingWizardMount } from '@globalart/astrum-ui-plugin';
import LeftNavigationMenuRadix from './components/LeftNavigationMenu/LeftNavigationMenuRadix';
import { Theme } from '@radix-ui/themes';
import { TooltipContainerProvider } from './components/shared/Tooltip/TooltipContainerContext';
import { getRouter } from './routerConfig/routerConfig';
import { ROUTE_NAMES } from './routerConfig/routeNames/routeNames';
import GlobalHeaderRadix from './components/GlobalHeader/GlobalHeaderRadix';
import { ThemeProvider, useTheme } from './contexts/ThemeContext';

import './App.scss';
import './components/shared/Toast.scss';
import SystemNoticesSwitch from './components/widgets/SystemStatusAlerts/SystemNoticesSwitch';
import { useRedirectOnLogout } from './hooks/useRedirectOnLogout';
import usePreventPushStateOnHash from './hooks/usePreventPushStateOnHash';

// WHOLE_DOCUMENT allows DOMPurify to parse <head> and preserve <style> tags that customers
// place there; FORCE_BODY returns only body content so no <html>/<head> wrapper leaks into the DOM.
const BRANDING_SANITIZE_OPTIONS = Object.freeze({
  WHOLE_DOCUMENT: true,
  FORCE_BODY: true,
});

/**
 * Global session-expired modal (REST 401). Mounted for all routes so interceptors can always notify.
 */
function SessionExpiryHost() {
  const { isExpired, message, hideExpiryModal } = useSessionExpiry();
  return (
    <SessionExpiryModal
      isOpen={isExpired}
      onClose={hideExpiryModal}
      message={message}
    />
  );
}

/**
 * AppWithRadixTheme - Connects ThemeProvider to Radix Theme.
 * Keeps the Radix-based UI improvements (GlobalHeaderRadix, LeftNavigationMenuRadix)
 * while using ExtJS for session/auth/state management.
 */
function AppWithRadixTheme() {
  const {effectiveTheme} = useTheme();

  return (
    <Theme appearance={effectiveTheme} accentColor="green" grayColor="slate" radius="medium">
      <TooltipContainerProvider>
        <NuiTooltipContainerProvider>
          <ToastProvider>
            <SessionExpiryHost />
            <App />
          </ToastProvider>
        </NuiTooltipContainerProvider>
      </TooltipContainerProvider>
    </Theme>
  );
}

export function App() {
  useRedirectOnLogout();
  usePreventPushStateOnHash();
  const {effectiveTheme} = useTheme();
  const toast = useToast();
  useLayoutEffect(() => {
    window.__astrumToast = toast;
    return () => { delete window.__astrumToast; };
  }, [toast]);
  const router = useRouter();
  const currentStateName = router.globals.$current.name || '';

  const isLoginRoute = currentStateName === ROUTE_NAMES.LOGIN;

  // Read branding from ExtJS state (available because we wait for ExtJS before rendering)
  const branding = ExtJS.state().getValue('branding');

  const headerEnabled = branding?.headerEnabled;
  const headerHtml = branding?.headerHtml;
  const footerEnabled = branding?.footerEnabled;
  const footerHtml = branding?.footerHtml;

  // Render minimal layout for login route
  if (isLoginRoute) {
    return <UIView />;
  }

  // Render standard layout for all other routes
  return (
    <>
      <SystemNoticesSwitch />

      {headerEnabled && (
        <div
          className="as-branding-header"
          data-testid="as-branding-header"
          dangerouslySetInnerHTML={{ __html: DOMPurify.sanitize(headerHtml, BRANDING_SANITIZE_OPTIONS) }}
        />
      )}

      <GlobalHeaderRadix />

      <LeftNavigationMenuRadix />

      <OnboardingWizardMount />

      <UnsavedChangesModal/>

      <PreviewUnsavedDialog />

      <div className="as-main-content">
        <Theme appearance={effectiveTheme} accentColor="blue" grayColor="slate" radius="medium">
          <Suspense fallback={<RouteLoadingFallback />}>
            <UIView />
          </Suspense>
        </Theme>
      </div>

      {footerEnabled && (
        <div
          className="as-branding-footer"
          data-testid="as-branding-footer"
          dangerouslySetInnerHTML={{ __html: DOMPurify.sanitize(footerHtml, BRANDING_SANITIZE_OPTIONS) }}
        />
      )}
    </>
  );
}

/**
 * Phase 2: React-First Bootstrap with REST Auth (ASTRUM-52583)
 *
 * 1. ExtJS.waitForExtJs() now returns as soon as ExtJS app initializes
 *    (does NOT wait for permissions to load)
 * 2. React renders immediately with empty/REST-bootstrap permissions
 * 3. Permissions load asynchronously in background
 * 4. Components update via permission change events
 *
 * This enables UI interactivity within ~2 seconds even for users with 400+ roles,
 * where permission loading can take 10-15 seconds.
 */
function renderApp() {
  if (window.__asAppRendered) {
    return;
  }
  window.__asAppRendered = true;

  try {
    const router = getRouter();
    const el = document.createElement('div');
    el.className = 'as-page astrum-page';
    document.body.appendChild(el);
    const root = createRoot(el);

    root.render(
    <ThemeProvider>
      <UIRouter router={router}>
        <AppWithRadixTheme />
      </UIRouter>
    </ThemeProvider>
  );
  } catch (err) {
    window.__asAppRendered = false;
  }
}

/**
 * Phase 2: React Shell with Non-Blocking Bootstrap (ASTRUM-52583)
 *
 * Boot sequence uses ExtJS.waitForExtJs() which now returns immediately
 * when ExtJS initializes (not waiting for permissions).
 *
 * Fallbacks in ExtJS.js handle permission checks during loading:
 *   - ExtJS.checkPermission() reads from __asRestBootstrap when AS.Permissions isn't ready
 *   - Components can use ExtJS.arePermissionsReady() to check loading state
 *
 * This enables immediate UI rendering even for users with 400+ roles
 * where permission loading can take 10-15 seconds on cold cache.
 */
// Build identifier - check in browser console: window.__asBuild
window.__asBuild = typeof __AS_BUILD_COMMIT__ !== 'undefined' ? __AS_BUILD_COMMIT__ : 'dev';

/** Timeout (ms) before falling back to REST bootstrap when ExtJS fails to load */
const EXTJS_TIMEOUT_MS = 15000;

let extJsResolved = false;

function bootstrapAndRender() {
  if (extJsResolved) return;
  extJsResolved = true;
  renderApp();
}

ExtJS.waitForExtJs(() => bootstrapAndRender());

// Fallback: if ExtJS never loads (e.g. app.js 404, script error), bootstrap from REST and render
setTimeout(() => {
  if (extJsResolved) return;
  bootstrapFromREST()
    .then(() => bootstrapAndRender())
    .catch((err) => {
      bootstrapAndRender();
    });
}, EXTJS_TIMEOUT_MS);
