
import React, { useState } from 'react';
import { Theme, Card, Flex, Heading, Text, Box } from '@radix-ui/themes';
import { ExtJS } from '../../../interface/ExtJS';
import UIStrings from "../../../constants/UIStrings";
import LoginLayout from "../../layout/LoginLayout";
import AnonymousAccess from "./AnonymousAccess";
import InitialPasswordInfo from "./InitialPasswordInfo";
import LocalLogin from "./LocalLogin";
import SsoLogin from "./SsoLogin";

const { LOGIN_TITLE, LOGIN_SUBTITLE, SSO_DIVIDER_LABEL } = UIStrings;

import "./LoginPage.scss";
import '@radix-ui/themes/styles.css';
import markOnlyLogoUrl from '../../../art/logos/astrum-mark.svg';

const localAuthenticationRealms = [
  "ldapRealmEnabled",
  "userTokenRealmEnabled",
  "localAuthRealmEnabled",
  "crowdRealmEnabled",
];
const ssoAuthenticationRealms = ["samlEnabled", "oauth2Enabled"];

/**
 * Login page component that renders within LoginLayout.
 * Displays a welcome message and login form matching the design specification.
 * @param {Object} logoConfig - Logo configuration passed to LoginLayout
 * @param {React.ReactNode} [headerActions] - Controls shown at the end of the header
 */
export default function LoginPage({ logoConfig, headerActions }) {
  const [generalError, setGeneralError] = useState(null);

  const extStateReady = !!(window.AS?.State?.getValue);
  const getState = (key, fallback = null) => {
    try {
      return ExtJS.state().getValue?.(key, fallback) ?? fallback;
    } catch {
      return fallback;
    }
  };

  const isCloudEnvironment = !!getState("isCloud", false);
  // Allow local dev to test cloud deployments without SSO infrastructure
  const isLocalDev = window.location.hostname === 'localhost' || window.location.hostname === '127.0.0.1';
  const showContinueWithoutLogin = !!getState("anonymousUsername", false);

  const showSSOLogin = extStateReady
    ? ssoAuthenticationRealms.some((realm) => getState(realm, false))
    : false;

  const showLocalLogin = (!isCloudEnvironment || isLocalDev) && (
    extStateReady
      ? localAuthenticationRealms.some((realm) => getState(realm, false))
      : true // default to showing local login when ExtJS state isn't ready yet
  );

  const adminPasswordFilePath = getState("admin.password.file");
  const onboardingRequired = getState("onboarding.required", false);

  const showInitialPasswordPathInfo =
    !!adminPasswordFilePath && !isCloudEnvironment && onboardingRequired;


  return (
    <LoginLayout logoConfig={logoConfig} headerActions={headerActions}>
      <Theme appearance="light" accentColor="blue" hasBackground={false} className="login-theme-wrapper">
        <Card className="login-card" data-testid="login-tile" size="4">
        <Flex direction="column" gap="5" className="login-card__content">
          {/* Header with mark logo */}
          <Flex direction="column" align="center" gap="4" className="login-card__header">
            <img
              src={markOnlyLogoUrl}
              alt=""
              className="login-card__mark"
              width={48}
              height={48}
              decoding="async"
            />
            <Heading as="h1" size="6" weight="medium" className="login-card__title">
              {LOGIN_TITLE}
            </Heading>
            <Text size="3" color="gray" className="login-card__subtitle">
              {LOGIN_SUBTITLE}
            </Text>
          </Flex>

          {/* Error Message */}
          {generalError && (
            <Flex role="alert" className="login-error" align="center" justify="between" gap="2">
              <Text size="2" color="red">{generalError}</Text>
              <button
                type="button"
                aria-label={UIStrings.CLOSE}
                className="login-error__close"
                onClick={() => setGeneralError(null)}
              >
                ×
              </button>
            </Flex>
          )}

          {/* Initial Password Info */}
          {showInitialPasswordPathInfo && (
            <InitialPasswordInfo passwordFilePath={adminPasswordFilePath} />
          )}

          {/* SSO Login */}
          {showSSOLogin && <SsoLogin autoFocus={!showLocalLogin} />}

          {/* Divider - only when both SSO and local login are visible */}
          {showSSOLogin && showLocalLogin && (
            <Flex align="center" gap="3" className="login-divider">
              <Box className="login-divider__line" style={{flex: 1}} />
              <Text size="2" color="gray">{SSO_DIVIDER_LABEL}</Text>
              <Box className="login-divider__line" style={{flex: 1}} />
            </Flex>
          )}

          {/* Local Login */}
          {showLocalLogin && (
            <LocalLogin primaryButton={!showSSOLogin} onError={setGeneralError} />
          )}

          {/* Anonymous Access */}
          {showContinueWithoutLogin && (
            <>
              <Box className="login-divider__line" mt="2" style={{width: '100%'}} />
              <AnonymousAccess />
            </>
          )}
        </Flex>
      </Card>
      </Theme>
    </LoginLayout>
  );
}
