

import React, { useEffect } from 'react';

import { useWizard } from './useWizard';
import UIStrings from './UIStrings';

import './Welcome.scss';
import welcomeLogoUrl from '../../art/logos/astrum-mark.svg';

export default function Welcome(): JSX.Element {
  const { registerStep, getStarted } = useWizard();
  const { LOGO_ALT, TITLE, DESCRIPTION } = UIStrings.ONBOARDING_WIZARD.WELCOME;

  useEffect(() => {
    registerStep({ valid: true, onSubmit: () => getStarted() });
  }, [registerStep, getStarted]);

  return (
    <div className="onboarding-wizard__welcome">
      <img
        src={welcomeLogoUrl}
        alt={LOGO_ALT}
        className="onboarding-wizard__welcome-logo"
        width={64}
        height={64}
        decoding="async"
      />
      <h1 className="onboarding-wizard__welcome-title">{TITLE}</h1>
      <p className="onboarding-wizard__welcome-description">{DESCRIPTION}</p>
    </div>
  );
}
