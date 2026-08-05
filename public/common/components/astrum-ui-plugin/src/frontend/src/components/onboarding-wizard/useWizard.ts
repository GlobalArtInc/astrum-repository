

import { useContext } from 'react';

import { WizardContext } from './WizardContext';
import type { UseWizardResult } from './WizardContext';

export function useWizard(): UseWizardResult {
  const context = useContext(WizardContext);
  if (context === null) {
    throw new Error('useWizard must be used within a WizardProvider');
  }
  return context;
}
