

import React from 'react';
import { Flex, Link } from '@radix-ui/themes';
import { useRouter } from '@uirouter/react';
import UIStrings from '../../../constants/UIStrings';
import { RouteNames } from "../../../constants/RouteNames";

const { CONTINUE_WITHOUT_LOGIN } = UIStrings;

/**
 * Link component that allows users to continue without logging in when anonymous access is enabled.
 * Properly handles returnTo parameter for redirect after anonymous navigation.
 */
export default function AnonymousAccess() {
  const router = useRouter();

  const handleContinueWithoutLogin = (e) => {
    e.preventDefault();
    try {
      const returnTo = router.globals.params.returnTo;
      if (returnTo) {
        const decodedReturnTo = atob(returnTo);
        // Validate decoded URL is a safe relative path (prevents open redirect)
        if (decodedReturnTo.startsWith('/') || decodedReturnTo.startsWith('#')) {
          router.urlService.url(decodedReturnTo);
        } else {
          router.stateService.go(RouteNames.MISSING_ROUTE);
        }
      } else {
        router.stateService.go(RouteNames.WELCOME);
      }
    } catch (ex) {
      console.warn('redirection unsuccessful: ', ex);
      router.stateService.go(RouteNames.MISSING_ROUTE);
    }
  };

  return (
    <Flex justify="center" pt="3" className="login-footer">
      <Link
        href="#browse/welcome"
        onClick={handleContinueWithoutLogin}
        className="login-continue"
        size="2"
        data-testid="continue-without-login-button"
        data-analytics-id="as-login-anonymous"
      >
        {CONTINUE_WITHOUT_LOGIN}
      </Link>
    </Flex>
  );
}
