

import type React from 'react';

import AnonymousAccessStep from './AnonymousAccessStep';
import ChangePasswordStep from './ChangePasswordStep';
import EulaStep from './steps/EulaStep/EulaStep';

export type StepComponent = React.FC;

// Maps onboarding step type (from the server) to its React component. To add a
// new step, import its component and add an entry to this literal.
export const stepRegistry: Record<string, StepComponent> = {
  ChangeAdminPassword: ChangePasswordStep,
  ConfigureAnonymousAccess: AnonymousAccessStep,
  CommunityEula: EulaStep,
};
