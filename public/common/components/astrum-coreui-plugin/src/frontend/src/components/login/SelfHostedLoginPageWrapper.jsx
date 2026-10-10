
import React from 'react';
import { LoginPage } from '@globalart/astrum-ui-plugin';
import LanguageMenuRadix from '../GlobalHeader/LanguageMenuRadix';

import proLogo from '../../../../art/logos/logo-pro-edition-header.svg';
import ceLogo from '../../../../art/logos/logo-community-edition-header.svg';
import coreLogo from '../../../../art/logos/logo-core-edition-header.svg';

const logoConfig = {
  proLight: proLogo,
  ceLight: ceLogo,
  coreLight: coreLogo,
};

/**
 * Self-hosted login page wrapper with all edition logos configured.
 * Uses the unified LoginPage from astrum-ui-plugin with proper logo configuration.
 */
export default function SelfHostedLoginPageWrapper() {
  return <LoginPage logoConfig={logoConfig} headerActions={<LanguageMenuRadix />} />;
}
