

import { useState, useCallback } from 'react';
import { APIConstants } from '../../../../../../constants/APIConstants';
import { restClient, parseApiError } from '../../../../../../interface/api';

import { RecoveryModeData } from './types';

const RECOVERY_MODE_UI_URL = APIConstants.REST.INTERNAL.RECOVERY_MODE;
const RECOVERY_MODE_PUBLIC_URL = APIConstants.REST.PUBLIC.RECOVERY_MODE;

/**
 * Hook for Recovery Mode API operations.
 *
 * - fetchRecoveryMode: GET internal/ui/recovery-mode (state + tasks)
 * - enableRecoveryMode: POST v1/recovery-mode
 * - disableRecoveryMode: DELETE v1/recovery-mode
 */
export function useRecoveryModeApi() {
  const [error, setError] = useState<string | null>(null);

  const fetchRecoveryMode = useCallback(async (): Promise<RecoveryModeData> => {
    setError(null);
    try {
      return await restClient.get<RecoveryModeData>(RECOVERY_MODE_UI_URL);
    } catch (err: unknown) {
      const message = parseApiError(err).message || 'Failed to load recovery mode settings';
      setError(message);
      throw err;
    }
  }, []);

  const enableRecoveryMode = useCallback(async (): Promise<void> => {
    setError(null);
    try {
      await restClient.post(RECOVERY_MODE_PUBLIC_URL);
    } catch (err: unknown) {
      const message = parseApiError(err).message || 'Failed to enable recovery mode';
      setError(message);
      throw err;
    }
  }, []);

  const disableRecoveryMode = useCallback(async (): Promise<void> => {
    setError(null);
    try {
      await restClient.delete(RECOVERY_MODE_PUBLIC_URL);
    } catch (err: unknown) {
      const message = parseApiError(err).message || 'Failed to disable recovery mode';
      setError(message);
      throw err;
    }
  }, []);

  return {
    error,
    setError,
    fetchRecoveryMode,
    enableRecoveryMode,
    disableRecoveryMode,
  };
}

export default useRecoveryModeApi;
