

import { useCallback, useEffect, useMemo, useRef } from 'react';
import { useMachine } from '@xstate/react';
import Axios from 'axios';
import { ExtJS } from '../../../../interface/ExtJS';
import { useToast } from '../../shared';
import { createRevealTokenMachine } from './shared/revealTokenMachine';

const NUGET_API_KEY_BASE = '/service/rest/internal/nuget-api-key';

function nugetApiKeyUrl(authToken: string): string {
  return `${NUGET_API_KEY_BASE}?authToken=${btoa(authToken)}`;
}

export interface RevealedNuGetApiKey {
  apiKey: string;
}

async function fetchToken(): Promise<RevealedNuGetApiKey> {
  const authToken = await ExtJS.requestAuthenticationToken(
    'Please authenticate to access your NuGet API key.'
  );
  const res = await Axios.get(nugetApiKeyUrl(authToken));
  return { apiKey: res.data?.apiKey ?? res.data };
}

async function deleteToken(): Promise<void> {
  const authToken = await ExtJS.requestAuthenticationToken(
    'Please authenticate to reset your NuGet API key.'
  );
  await Axios.delete(nugetApiKeyUrl(authToken));
}

export interface UseNuGetApiTokenReturn {
  apiKey: string | null;
  showReveal: boolean;
  accessLoading: boolean;
  resetLoading: boolean;
  handleAccess: () => void;
  handleReset: () => void;
  handleCloseReveal: () => void;
}

/**
 * Hook for the NuGet API Key page. Uses the shared revealTokenMachine with only
 * fetchToken + deleteToken configured — no status endpoint, no generate flow.
 */
export function useNuGetApiToken(): UseNuGetApiTokenReturn {
  const toast = useToast();
  const toastRef = useRef(toast);
  toastRef.current = toast;

  const machine = useMemo(
    () =>
      createRevealTokenMachine<null, RevealedNuGetApiKey>({
        id: 'nuget-api-token',
        tokenType: 'nuget',
        services: {
          fetchToken,
          deleteToken,
        },
      }),
    []
  );

  const [state, send] = useMachine(machine);
  const pendingActionRef = useRef<'reveal' | 'delete' | null>(null);
  const wasDeletingRef = useRef(false);
  const prevActionErrorRef = useRef<string | null>(null);

  useEffect(() => {
    const isDeleting = state.matches('deleting');
    if (wasDeletingRef.current && !isDeleting && !state.context.actionError) {
      toastRef.current.success('API key reset. Generate a new one.');
      pendingActionRef.current = null;
    }
    wasDeletingRef.current = isDeleting;
  }, [state]);

  useEffect(() => {
    const err = state.context.actionError;
    if (err && err !== prevActionErrorRef.current) {
      const action = pendingActionRef.current;
      if (action === 'reveal') {
        toastRef.current.error('Failed to retrieve NuGet API key. Please check your credentials.');
      } else if (action === 'delete') {
        toastRef.current.error('Failed to reset NuGet API key. Please check your credentials.');
      }
      pendingActionRef.current = null;
    }
    prevActionErrorRef.current = err;
  }, [state.context.actionError]);

  const isRevealed = state.matches('revealed');
  const apiKey = isRevealed ? state.context.tokenValue?.apiKey ?? null : null;

  const handleAccess = useCallback(() => {
    pendingActionRef.current = 'reveal';
    send({ type: 'REVEAL' });
  }, [send]);

  const handleReset = useCallback(() => {
    pendingActionRef.current = 'delete';
    send({ type: 'DELETE' });
  }, [send]);

  const handleCloseReveal = useCallback(() => {
    send({ type: 'HIDE' });
  }, [send]);

  return {
    apiKey,
    showReveal: isRevealed,
    accessLoading: state.matches('revealing'),
    resetLoading: state.matches('deleting'),
    handleAccess,
    handleReset,
    handleCloseReveal,
  };
}
