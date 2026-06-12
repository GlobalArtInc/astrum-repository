

import './styles/fonts';
import './App';
import { enforceHashRouting } from './hashEnforcement';
import CommunityEulaOnboarding from './components/pages/onboarding/CommunityEulaOnboarding';
import TelemetryWarningBanner from './components/shared/telemetry/TelemetryWarningBanner';
import './styles/extjs.scss';
import { configureAxios, configureDebugLogging, exposeCreateRoot } from '@globalart/astrum-ui-plugin';

enforceHashRouting();

configureAxios();
configureDebugLogging();
exposeCreateRoot();

window.ReactComponents = {
  ...window.ReactComponents,
  CommunityEulaOnboarding,
  TelemetryWarningBanner,
};
