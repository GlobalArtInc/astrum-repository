

import React from 'react';
import {
  AsButton,
  AsFontAwesomeIcon,
  AsStatefulNavigationDropdown,
  AsNavigationDropdown,
  AsTextLink,
  AsH4,
  useToggle,
  AsDropdown
} from '@globalart/astrum-react-shared-components';
import { ExtJS, useIsVisible } from '@globalart/astrum-ui-plugin';
import { faUser } from '@fortawesome/free-solid-svg-icons';
import { useRouter, useSref } from '@uirouter/react';
import useHasUser from '../../hooks/useHasUser';
import { ROUTE_NAMES } from '../../routerConfig/routeNames/routeNames';
import UIStrings from '../../constants/UIStrings';

export default function LoginAndUserButton() {
  const signInTitle = "Log In"
  const hasUser = useHasUser();
  const router = useRouter();

  if (hasUser) {
    return <ProfileMenu />
  } else {
    return <AsButton
        title={signInTitle}
        aria-label={signInTitle}
        variant="icon-only" onClick={onSignInClick}
        data-analytics-id="as-global-header-login"
    >
      <AsFontAwesomeIcon icon={faUser} />
    </AsButton>
  }

  function onSignInClick() {
    // Keep original requested URL and then encode to Base64
    const url = router.urlService.url();
    if (url) {
      const returnTo = btoa(`#${url}`);
      router.stateService.go(ROUTE_NAMES.LOGIN, { returnTo });
    } else {
      router.stateService.go(ROUTE_NAMES.LOGIN);
    }
  }
}

function ProfileMenu() {
  const { USER } = ROUTE_NAMES;

  const userTokenRouteName = USER.USER_TOKEN;
  const userNugetApiTokenName = USER.NUGETAPITOKEN;

  const userId = ExtJS.useState(() => ExtJS.state().getValue('user')?.id);

  const [isOpen, onToggleCollapse] = useToggle(false);

  const router = useRouter();

  const userTokenRouteState = router.stateRegistry.get(userTokenRouteName);
  const isUserTokenEnabled = useIsVisible(userTokenRouteState.data.visibilityRequirements);

  const userNugetApiTokenState = router.stateRegistry.get(userNugetApiTokenName);
  const isUserNugetApiTokenEnabled = useIsVisible(userNugetApiTokenState.data.visibilityRequirements);

  const { href: userAccountHref } = useSref(USER.ACCOUNT);
  const { href: userNugetApiTokenHref } = useSref(USER.NUGETAPITOKEN);
  const { href: userTokenHref } = useSref(userTokenRouteName);

  return (
      <AsStatefulNavigationDropdown
          isOpen={isOpen}
          onToggleCollapse={onToggleCollapse}
          icon={faUser}
          title="Account settings, tokens and keys"
          data-analytics-id="as-global-header-profile-menu"
      >
        <AsNavigationDropdown.MenuHeader>
          <AsH4>
            <AsFontAwesomeIcon className="as-global-header-profile-menu-user-icon" icon={faUser}/>
            {userId}
          </AsH4>
        </AsNavigationDropdown.MenuHeader>

        <AsTextLink
            href={userAccountHref}
            className="as-dropdown-link"
            data-analytics-id="as-global-header-profile-menu-my-profile"
        >
          {UIStrings.USER_ACCOUNT.MENU.text}
        </AsTextLink>

        {isUserNugetApiTokenEnabled && (
        <AsTextLink
          href={userNugetApiTokenHref}
          className='as-dropdown-link'
          data-analytics-id='as-global-header-profile-menu-nuget-api-key'
        >
          NuGet API Key
        </AsTextLink>
      )}

        { isUserTokenEnabled
            ? (<AsTextLink
                href={userTokenHref}
                className="as-dropdown-link"
                data-analytics-id="as-global-header-profile-menu-user-token"
            >
              User Token
            </AsTextLink>)
            : null
        }

        <AsDropdown.Divider />

        <button onClick={onSingOutClick} className="as-dropdown-button">
          Log Out
        </button>
      </AsStatefulNavigationDropdown>);

  function onSingOutClick() {
    // Reset to light mode on logout since Default UI doesn't support dark mode
    // This ensures consistent state when user logs back in
    try {
      localStorage.setItem('astrum-theme-preference', 'light');
      document.documentElement.setAttribute('data-theme', 'light');
    } catch (e) {
      // Ignore localStorage errors
    }
    ExtJS.signOut();
  }
}
