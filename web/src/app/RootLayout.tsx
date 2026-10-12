import { Theme } from '@radix-ui/themes';
import { useEffect, useLayoutEffect } from 'react';
import { Outlet, useLocation, useNavigate } from 'react-router';
import { LOGIN_PATH } from '../auth/returnTo';
import { SessionWatcher } from '../auth/SessionWatcher';
import { useTheme } from '../shell/theme';
import { ConfirmProvider } from '../ui/ConfirmDialog';
import { Toaster } from '../ui/Toaster';
import { legacyPath } from './legacy';

export function AppTheme({
  forceLight = false,
  children,
}: {
  forceLight?: boolean;
  children: React.ReactNode;
}) {
  const { appearance } = useTheme();
  const shown = forceLight ? 'light' : appearance;
  useLayoutEffect(() => {
    document.documentElement.dataset.theme = shown;
  }, [shown]);
  return (
    <Theme appearance={shown} accentColor="blue" grayColor="slate" radius="medium" panelBackground="solid">
      {children}
    </Theme>
  );
}

function useLegacyRedirects() {
  const navigate = useNavigate();
  useEffect(() => {
    const follow = () => {
      const target = legacyPath(window.location.hash);
      if (target) {
        navigate(target, { replace: true });
      }
    };
    window.addEventListener('hashchange', follow);
    return () => window.removeEventListener('hashchange', follow);
  }, [navigate]);
}

export function RootLayout() {
  const { pathname } = useLocation();
  useLegacyRedirects();
  return (
    <AppTheme forceLight={pathname === LOGIN_PATH}>
      <Toaster>
        <ConfirmProvider>
          <Outlet />
          <SessionWatcher />
        </ConfirmProvider>
      </Toaster>
    </AppTheme>
  );
}
