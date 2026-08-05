

import { useCallback, useEffect, useState } from 'react';

import { useAnonymousApi } from '../preview/pages/settings/security/anonymous/useAnonymousApi';
import { useWizard } from './useWizard';

const ANONYMOUS_USER_ID = 'anonymous';
const ANONYMOUS_REALM_NAME = 'AstrumAuthorizingRealm';
const DEFAULT_SECURE_DISABLED = false;

export interface UseAnonymousAccessStepResult {
  enabled: boolean;
  onSelect: (value: boolean) => void;
  onFormSubmit: (event: React.FormEvent) => void;
}

export function useAnonymousAccessStep(): UseAnonymousAccessStepResult {
  const { registerStep, submit } = useWizard();
  const { fetchSettings, saveSettings } = useAnonymousApi();
  const [enabled, setEnabled] = useState<boolean>(DEFAULT_SECURE_DISABLED);

  useEffect(() => {
    let cancelled = false;
    fetchSettings()
      .then((settings) => {
        if (!cancelled) {
          setEnabled(settings.enabled);
        }
      })
      .catch((error) => {
        console.warn('[AnonymousAccessStep] Failed to fetch anonymous settings', error);
      });
    return () => {
      cancelled = true;
    };
  }, [fetchSettings]);

  const onSelect = useCallback((value: boolean) => {
    setEnabled(value);
  }, []);

  const onSubmit = useCallback(async () => {
    await saveSettings({
      userId: ANONYMOUS_USER_ID,
      realmName: ANONYMOUS_REALM_NAME,
      enabled,
    });
  }, [saveSettings, enabled]);

  useEffect(() => {
    registerStep({ valid: true, onSubmit });
  }, [registerStep, onSubmit]);

  const onFormSubmit = useCallback(
    (event: React.FormEvent) => {
      event.preventDefault();
      submit();
    },
    [submit]
  );

  return { enabled, onSelect, onFormSubmit };
}
