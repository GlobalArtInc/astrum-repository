import type { Area } from '../../app/area';
import { ADMINISTRATOR, OnboardingWatcher } from './watcher';

export const onboardingArea: Area = {
  id: 'onboarding',
  routes: [
    { path: 'onboarding', load: () => import('./OnboardingPage'), need: ADMINISTRATOR, requiresUser: true },
  ],
  watchers: [OnboardingWatcher],
};
