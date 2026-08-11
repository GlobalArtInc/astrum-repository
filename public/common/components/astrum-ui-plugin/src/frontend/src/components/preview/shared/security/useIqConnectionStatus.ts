

/**
 * useIqConnectionStatus - Shared hook for IQ Server connection awareness
 *
 * Used by:
 * - Dashboard (FirewallCard) - Shows connection status
 * - Repository Profile IQ Server tab - Shows error when disconnected
 * - Browse Firewall Report - Shows error when disconnected
 *
 * This hook:
 * 1. Checks if IQ Server is configured (has URL)
 * 2. Auto-tests connection on mount
 * 3. Provides reactive connection status
 */

import { useState, useEffect, useCallback } from 'react';
import Axios from 'axios';

const IQ_API = 'service/rest/v1/iq';
const IQ_VERIFY_API = 'service/rest/internal/ui/iq/verify-connection';

export type IqConnectionState = 'unknown' | 'testing' | 'connected' | 'disconnected' | 'not-configured';

export interface IqConnectionStatus {
  /** Current connection state */
  state: IqConnectionState;
  /** Whether IQ Server URL is configured */
  isConfigured: boolean;
  /** Whether currently connected */
  isConnected: boolean;
  /** Whether connection test is in progress */
  isTesting: boolean;
  /** Error message if disconnected */
  errorMessage?: string;
  /** IQ Server URL (if configured) */
  url?: string;
  /** Manually trigger connection test */
  testConnection: () => Promise<void>;
  /** Refresh configuration and test */
  refresh: () => Promise<void>;
}

interface IqConfig {
  enabled?: boolean;
  url?: string;
  username?: string;
}

/**
 * Hook for IQ Server connection status awareness.
 *
 * @param autoTest - Whether to auto-test on mount (default: true)
 * @returns IqConnectionStatus object
 */
export function useIqConnectionStatus(autoTest = true): IqConnectionStatus {
  const [state, setState] = useState<IqConnectionState>('unknown');
  const [config, setConfig] = useState<IqConfig | null>(null);
  const [errorMessage, setErrorMessage] = useState<string | undefined>(undefined);

  const fetchConfig = useCallback(async (): Promise<IqConfig | null> => {
    try {
      const response = await Axios.get<IqConfig>(IQ_API);
      return response.data;
    } catch {
      return null;
    }
  }, []);

  const testConnection = useCallback(async () => {
    if (!(config?.url && config?.enabled)) {
      setState('not-configured');
      return;
    }

    setState('testing');
    setErrorMessage(undefined);

    try {
      const response = await Axios.post(IQ_VERIFY_API, config);
      if (response.data?.success !== false) {
        setState('connected');
      } else {
        setState('disconnected');
        setErrorMessage(response.data?.reason || 'Connection failed');
      }
    } catch (err: any) {
      setState('disconnected');
      const reason = err?.response?.data || err?.message || 'Connection verification failed';
      setErrorMessage(typeof reason === 'string' ? reason : JSON.stringify(reason));
    }
  }, [config]);

  const refresh = useCallback(async () => {
    const newConfig = await fetchConfig();
    setConfig(newConfig);

    if (!(newConfig?.url && newConfig?.enabled)) {
      setState('not-configured');
      setErrorMessage(undefined);
      return;
    }

    // Auto-test after fetching config
    setState('testing');
    setErrorMessage(undefined);

    try {
      const response = await Axios.post(IQ_VERIFY_API, newConfig);
      if (response.data?.success !== false) {
        setState('connected');
      } else {
        setState('disconnected');
        setErrorMessage(response.data?.reason || 'Connection failed');
      }
    } catch (err: any) {
      setState('disconnected');
      const reason = err?.response?.data || err?.message || 'Connection verification failed';
      setErrorMessage(typeof reason === 'string' ? reason : JSON.stringify(reason));
    }
  }, [fetchConfig]);

  // Initial load
  useEffect(() => {
    let mounted = true;

    const init = async () => {
      const cfg = await fetchConfig();
      if (!mounted) return;

      setConfig(cfg);

      if (!(cfg?.url && cfg?.enabled)) {
        setState('not-configured');
        return;
      }

      if (autoTest) {
        setState('testing');
        try {
          const response = await Axios.post(IQ_VERIFY_API, cfg);
          if (!mounted) return;
          if (response.data?.success !== false) {
            setState('connected');
          } else {
            setState('disconnected');
            setErrorMessage(response.data?.reason || 'Connection failed');
          }
        } catch (err: any) {
          if (!mounted) return;
          setState('disconnected');
          const reason = err?.response?.data || err?.message || 'Connection verification failed';
          setErrorMessage(typeof reason === 'string' ? reason : JSON.stringify(reason));
        }
      }
    };

    init();

    return () => {
      mounted = false;
    };
  }, [autoTest, fetchConfig]);

  const isConfigured = Boolean(config?.url && config?.enabled);
  const isConnected = state === 'connected';
  const isTesting = state === 'testing';

  return {
    state,
    isConfigured,
    isConnected,
    isTesting,
    errorMessage,
    url: config?.url,
    testConnection,
    refresh,
  };
}

export default useIqConnectionStatus;
