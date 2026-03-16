import { useQueryClient } from '@tanstack/react-query';
import { useCallback } from 'react';
import { useT } from '../i18n';
import { describeError } from '../ui/ErrorState';
import { useToast } from '../ui/Toaster';
import { openFresh } from './browser';
import { expectSignOut, signOut, takeExpectedSignOut } from './session';

export function useSignOut(): () => Promise<void> {
  const t = useT();
  const toast = useToast();
  const client = useQueryClient();
  return useCallback(async () => {
    expectSignOut();
    try {
      await signOut();
    } catch (error) {
      takeExpectedSignOut();
      toast.error(t('shell.user.signOutFailed'), describeError(t, error));
      return;
    }
    client.clear();
    openFresh('/');
  }, [client, t, toast]);
}
