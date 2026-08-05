

export type OnboardingStep = { type: string };

export type WizardStateValue =
  | 'loading'
  | 'stepReady'
  | 'setupComplete'
  | 'verifying'
  | 'done'
  | 'issueOccurred'
  | 'skipped';

export type WizardErrorKind = 'stillRequired' | 'network';

export interface WizardMachineContext {
  steps: OnboardingStep[];
  currentIndex: number;
  errorMessage: string | null;
  errorKind: WizardErrorKind | null;
}

export type WizardEvent =
  | { type: 'GET_STARTED' }
  | { type: 'SUBMIT' }
  | { type: 'STEP_ADVANCED' }
  | { type: 'STEP_FAILED'; error: string }
  | { type: 'SKIP' };
