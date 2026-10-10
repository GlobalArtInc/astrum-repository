

import React, {useState, useEffect} from 'react';
import {DropdownMenu, Flex, IconButton, Text} from '@radix-ui/themes';
import {Tooltip, usePortalContainer} from '@globalart/astrum-ui-plugin';
import {User, Key, LogOut} from 'lucide-react';
import {ExtJS, useIsVisible} from '@globalart/astrum-ui-plugin';
import {useRouter, useSref} from '@uirouter/react';
import useHasUser from '../../hooks/useHasUser';
import {ROUTE_NAMES} from '../../routerConfig/routeNames/routeNames';
import UIStrings from '../../constants/UIStrings';
import {useTheme} from '../../contexts/ThemeContext';
import {restClient} from '../../utils/api';

import './LoginAndUserButtonRadix.scss';

/**
 * Single component with all hooks at top - no conditional component switching.
 * Fixes "Rendered fewer hooks" when hasUser changes during logout.
 */
export default function LoginAndUserButtonRadix() {
  const portalContainer = usePortalContainer();
  const hasUser = useHasUser();
  const router = useRouter();
  const {USER} = ROUTE_NAMES;
  const {setTheme} = useTheme();

  const userTokenRouteName = USER.USER_TOKEN;
  const userNugetApiTokenName = USER.NUGETAPITOKEN;
  const [userId, setUserId] = useState(null);

  useEffect(() => {
    try {
      const state = ExtJS.state();
      if (state) {
        setUserId(state.getValue('user')?.id || null);
      }
    } catch (e) {
      setUserId(null);
    }
  }, []);

  const userTokenRouteState = router.stateRegistry.get(userTokenRouteName);
  const isUserTokenEnabled = useIsVisible(userTokenRouteState?.data?.visibilityRequirements);
  const userNugetApiTokenState = router.stateRegistry.get(userNugetApiTokenName);
  const isUserNugetApiTokenEnabled = useIsVisible(userNugetApiTokenState?.data?.visibilityRequirements);

  // Match GlobalHeaderRadix: Preview vs Classic is determined by the URL hash, not only
  // whether preview states are registered (fixes user menu linking to #user/... in Preview UI).
  const [hashPath, setHashPath] = useState(() => {
    if (typeof window === 'undefined') {
      return '';
    }
    return window.location.hash.replace(/^#/, '').split('?')[0] || '';
  });
  useEffect(() => {
    const syncFromLocation = () => {
      setHashPath(window.location.hash.replace(/^#/, '').split('?')[0] || '');
    };
    syncFromLocation();
    window.addEventListener('hashchange', syncFromLocation);
    window.addEventListener('popstate', syncFromLocation);
    return () => {
      window.removeEventListener('hashchange', syncFromLocation);
      window.removeEventListener('popstate', syncFromLocation);
    };
  }, []);
  const isPreviewUI = hashPath === 'preview' || hashPath.startsWith('preview/');

  const userAccountState = isPreviewUI ? 'preview.user.account' : USER.ACCOUNT;
  const nugetApiTokenState = isPreviewUI ? 'preview.user.nugetapitoken' : USER.NUGETAPITOKEN;
  const userTokenState = isPreviewUI ? 'preview.user.usertoken' : userTokenRouteName;
  const {href: userAccountHref} = useSref(userAccountState);
  const {href: userNugetApiTokenHref} = useSref(nugetApiTokenState);
  const {href: userTokenHref} = useSref(userTokenState);

  const onSignInClick = () => {
    const url = router.urlService.url();
    if (url) {
      const returnTo = btoa(`#${url}`);
      router.stateService.go(ROUTE_NAMES.LOGIN, {returnTo});
    } else {
      router.stateService.go(ROUTE_NAMES.LOGIN);
    }
  };

  function onSignOutClick() {
    // Reset to light mode on logout - Classic UI doesn't support dark mode.
    // Apply synchronously to DOM and localStorage BEFORE redirect so we never
    // flash dark theme on login/Classic UI pages.
    if (typeof document !== 'undefined') {
      document.documentElement.setAttribute('data-theme', 'light');
      document.body.classList.remove('dark');
      document.body.classList.add('light');
    }
    try {
      localStorage.setItem('astrum-theme-preference', 'light');
    } catch (e) {
      /* ignore */
    }
    setTheme('light');

    ExtJS.signOut();
  }

  if (!hasUser) {
    return (
      <Tooltip content={UIStrings.SHELL.ACCOUNT.LOG_IN}>
        <IconButton
          variant="outline"
          size="2"
          color="gray"
          aria-label={UIStrings.SHELL.ACCOUNT.LOG_IN}
          onClick={onSignInClick}
        >
          <User size={16} />
        </IconButton>
      </Tooltip>
    );
  }

  return (
    <DropdownMenu.Root>
      <Tooltip content={UIStrings.SHELL.ACCOUNT.MY_ACCOUNT}>
        <DropdownMenu.Trigger>
          <IconButton variant="outline" size="2" color="gray" aria-label={UIStrings.SHELL.ACCOUNT.MY_ACCOUNT} data-testid="user-menu" data-analytics-id="as-header-user-menu">
            <User size={16} />
          </IconButton>
        </DropdownMenu.Trigger>
      </Tooltip>

      <DropdownMenu.Content align="end" container={portalContainer} color="gray" variant="soft">
        <DropdownMenu.Label>
          <Text size="2" as="span" weight="bold" className="as-login-and-user-button__user-id">
            {userId}
          </Text>
        </DropdownMenu.Label>
        <DropdownMenu.Separator />

        <DropdownMenu.Item asChild>
          <a href={userAccountHref} className="as-internal-link">
            <Text size="2" color="gray">{UIStrings.USER_ACCOUNT.MENU.text}</Text>
          </a>
        </DropdownMenu.Item>

        {isUserNugetApiTokenEnabled && (
          <DropdownMenu.Item asChild>
            <a href={userNugetApiTokenHref} className="as-internal-link">
              <Text size="2" color="gray">{UIStrings.SHELL.ACCOUNT.NUGET_API_KEY}</Text>
            </a>
          </DropdownMenu.Item>
        )}

        {isUserTokenEnabled && (
          <DropdownMenu.Item asChild>
            <a href={userTokenHref} className="as-internal-link">
              <Flex align="center" justify="start" gap="3">
                <Text size="2">{UIStrings.SHELL.ACCOUNT.USER_TOKEN}</Text>
                <Key size={16} />
              </Flex>
            </a>
          </DropdownMenu.Item>
        )}

        <DropdownMenu.Separator />

        <DropdownMenu.Item onClick={onSignOutClick} data-analytics-id="as-header-sign-out">
          <Flex align="center" justify="start" gap="3">
            <LogOut size={16} />
            <Text size="2">{UIStrings.SHELL.ACCOUNT.LOG_OUT}</Text>
          </Flex>
        </DropdownMenu.Item>
      </DropdownMenu.Content>
    </DropdownMenu.Root>
  );
}

