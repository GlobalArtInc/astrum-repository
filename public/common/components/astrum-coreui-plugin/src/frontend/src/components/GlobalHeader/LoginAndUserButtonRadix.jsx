

import React, {useState, useEffect} from 'react';
import {DropdownMenu, Flex, IconButton, Text} from '@radix-ui/themes';
import {Tooltip, usePortalContainer} from '@globalart/astrum-ui-plugin';
import {User, UserRound, Key, KeyRound, LogOut} from 'lucide-react';
import {ExtJS, useIsVisible} from '@globalart/astrum-ui-plugin';
import {useRouter, useSref} from '@uirouter/react';
import useHasUser from '../../hooks/useHasUser';
import {ROUTE_NAMES} from '../../routerConfig/routeNames/routeNames';
import UIStrings from '../../constants/UIStrings';
import {restClient} from '../../utils/api';

import './LoginAndUserButtonRadix.scss';

/**
 * Up to two letters that stand for the user on the avatar: the first letters of the first two words of the
 * name ("jane.doe" gives "JD"), or its first two letters when it is a single word ("admin" gives "AD").
 */
export function getInitials(name) {
  const words = String(name ?? '').split(/[^\p{L}\p{N}]+/u).filter(Boolean);
  if (words.length === 0) {
    return '';
  }
  const letters = words.length > 1
    ? [...words[0]].slice(0, 1).concat([...words[1]].slice(0, 1))
    : [...words[0]].slice(0, 2);
  return letters.join('').toLocaleUpperCase();
}

/**
 * When the user menu offers the NuGet API key: the user may hold one and the NuGet API key realm is active.
 */
const NUGET_API_TOKEN_VISIBILITY = {
  requiresUser: true,
  permissions: ['astrum:apikey:*'],
  statesEnabled: [{key: 'nugetApiKeyRealmEnabled', defaultValue: false}],
};

/**
 * When the user menu offers the user token: the user token feature is installed, licensed and active, and the
 * user may read their own token.
 */
const USER_TOKEN_VISIBILITY = {
  bundle: 'astrum-usertoken-plugin',
  statesEnabled: [{key: 'userTokenRealmEnabled', defaultValue: false}],
  permissions: ['astrum:usertoken-current:read'],
  editions: ['PRO'],
};

/**
 * Single component with all hooks at top - no conditional component switching.
 * Fixes "Rendered fewer hooks" when hasUser changes during logout.
 */
export default function LoginAndUserButtonRadix() {
  const portalContainer = usePortalContainer();
  const hasUser = useHasUser();
  const user = ExtJS.useUser();
  const router = useRouter();

  const [stateUserId, setStateUserId] = useState(null);

  useEffect(() => {
    try {
      const state = ExtJS.state();
      if (state) {
        setStateUserId(state.getValue('user')?.id || null);
      }
    } catch (e) {
      setStateUserId(null);
    }
  }, []);

  const userId = user?.id || stateUserId;
  const initials = getInitials(userId);

  const isUserTokenEnabled = useIsVisible(USER_TOKEN_VISIBILITY);
  const isUserNugetApiTokenEnabled = useIsVisible(NUGET_API_TOKEN_VISIBILITY);

  const {href: userAccountHref} = useSref('preview.user.account');
  const {href: accessTokensHref} = useSref('preview.user.accesstokens');
  const {href: userNugetApiTokenHref} = useSref('preview.user.nugetapitoken');
  const {href: userTokenHref} = useSref('preview.user.usertoken');

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
    // The theme stays as the user chose it: every page supports both, and the sign-in page renders light anyway
    ExtJS.signOut();
  }

  if (!hasUser) {
    return (
      <Tooltip content={UIStrings.SHELL.ACCOUNT.LOG_IN}>
        <IconButton
          variant="outline"
          size="3"
          color="gray"
          className="as-global-header-radix__icon-button"
          aria-label={UIStrings.SHELL.ACCOUNT.LOG_IN}
          data-testid="user-sign-in"
          onClick={onSignInClick}
        >
          <User size={18} aria-hidden />
        </IconButton>
      </Tooltip>
    );
  }

  return (
    <DropdownMenu.Root>
      <Tooltip content={UIStrings.SHELL.ACCOUNT.MY_ACCOUNT}>
        <DropdownMenu.Trigger>
          <button
            type="button"
            className="as-login-and-user-button__avatar"
            aria-label={UIStrings.SHELL.ACCOUNT.MY_ACCOUNT}
            data-testid="user-menu"
            data-analytics-id="as-header-user-menu"
          >
            {initials ? <span aria-hidden>{initials}</span> : <User size={18} aria-hidden />}
          </button>
        </DropdownMenu.Trigger>
      </Tooltip>

      <DropdownMenu.Content
        align="end"
        container={portalContainer}
        color="gray"
        variant="soft"
        className="as-global-header-radix__menu as-login-and-user-button__menu"
      >
        <DropdownMenu.Label className="as-login-and-user-button__menu-label">
          <Flex align="center" gap="2">
            <span className="as-login-and-user-button__menu-avatar" aria-hidden>{initials}</span>
            <Text size="2" as="span" weight="bold" className="as-login-and-user-button__user-id">
              {userId}
            </Text>
          </Flex>
        </DropdownMenu.Label>
        <DropdownMenu.Separator />

        <DropdownMenu.Item asChild>
          <a href={userAccountHref} className="as-internal-link">
            <Flex align="center" justify="start" gap="3">
              <UserRound size={16} aria-hidden />
              <Text size="2">{UIStrings.USER_ACCOUNT.MENU.text}</Text>
            </Flex>
          </a>
        </DropdownMenu.Item>

        <DropdownMenu.Item asChild>
          <a href={accessTokensHref} className="as-internal-link" data-testid="user-menu-access-tokens">
            <Flex align="center" justify="start" gap="3">
              <KeyRound size={16} aria-hidden />
              <Text size="2">{UIStrings.SHELL.ACCOUNT.ACCESS_TOKENS}</Text>
            </Flex>
          </a>
        </DropdownMenu.Item>

        {isUserNugetApiTokenEnabled && (
          <DropdownMenu.Item asChild>
            <a href={userNugetApiTokenHref} className="as-internal-link">
              <Flex align="center" justify="start" gap="3">
                <Key size={16} aria-hidden />
                <Text size="2">{UIStrings.SHELL.ACCOUNT.NUGET_API_KEY}</Text>
              </Flex>
            </a>
          </DropdownMenu.Item>
        )}

        {isUserTokenEnabled && (
          <DropdownMenu.Item asChild>
            <a href={userTokenHref} className="as-internal-link">
              <Flex align="center" justify="start" gap="3">
                <Key size={16} aria-hidden />
                <Text size="2">{UIStrings.SHELL.ACCOUNT.USER_TOKEN}</Text>
              </Flex>
            </a>
          </DropdownMenu.Item>
        )}

        <DropdownMenu.Separator />

        <DropdownMenu.Item onClick={onSignOutClick} data-analytics-id="as-header-sign-out">
          <Flex align="center" justify="start" gap="3">
            <LogOut size={16} aria-hidden />
            <Text size="2">{UIStrings.SHELL.ACCOUNT.LOG_OUT}</Text>
          </Flex>
        </DropdownMenu.Item>
      </DropdownMenu.Content>
    </DropdownMenu.Root>
  );
}
