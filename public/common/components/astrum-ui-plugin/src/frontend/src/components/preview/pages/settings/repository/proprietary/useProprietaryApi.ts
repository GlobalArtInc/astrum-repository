

/**
 * Proprietary Repositories API Hook
 *
 * 100% REST - No ExtDirect.
 * Backend: RepositoryProprietaryContentResource.java
 * - GET  /internal/proprietary-content -> list of proprietary repo names
 * - POST /internal/proprietary-content -> update proprietary/nonProprietary lists
 */

import { useState, useCallback } from 'react';
import { restClient, parseApiError, ENDPOINTS } from '../../../../../../interface/api';
import { RepositoryReference, ProprietaryRepositoriesSettings } from './types';
import ProprietaryStrings from '../../../../constants/pages/settings/repository/ProprietaryStrings';

const PROPRIETARY_URL = '/service/rest/internal/proprietary-content';

interface RestRepository {
  name: string;
  format: string;
  type: string;
  url?: string;
  online?: boolean;
}

export function useProprietaryApi() {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchSettings = useCallback(async (): Promise<ProprietaryRepositoriesSettings> => {
    try {
      const data = await restClient.get<string[]>(PROPRIETARY_URL);
      return {
        enabledRepositories: Array.isArray(data) ? data : [],
      };
    } catch (err: unknown) {
      const apiError = parseApiError(err);
      throw new Error(apiError.message || ProprietaryStrings.API.FAILED_LOAD_PROPRIETARY_REPOSITORIES_SETTINGS);
    }
  }, []);

  const fetchPossibleRepositories = useCallback(async (): Promise<RepositoryReference[]> => {
    try {
      const repositories = await restClient.get<RestRepository[]>(ENDPOINTS.REPOSITORIES);
      if (!Array.isArray(repositories)) return [];
      return repositories
        .filter((repo) => repo.type === 'hosted')
        .map((repo) => ({ id: repo.name, name: repo.name }));
    } catch (err: unknown) {
      const apiError = parseApiError(err);
      throw new Error(apiError.message || ProprietaryStrings.API.FAILED_LOAD_REPOSITORIES);
    }
  }, []);

  const updateSettings = useCallback(async (
    enabledRepositories: string[],
    allPossibleRepositories?: RepositoryReference[]
  ): Promise<ProprietaryRepositoriesSettings> => {
    setLoading(true);
    setError(null);
    try {
      const allHostedNames = allPossibleRepositories?.map((r) => r.name) || [];
      const enabledSet = new Set(enabledRepositories);
      const nonProprietary = allHostedNames.filter((name) => !enabledSet.has(name));

      await restClient.post(PROPRIETARY_URL, {
        proprietary: enabledRepositories,
        nonProprietary,
      });

      return await fetchSettings();
    } catch (err: unknown) {
      const apiError = parseApiError(err);
      const message = apiError.message || ProprietaryStrings.API.FAILED_UPDATE_PROPRIETARY_REPOSITORIES_SETTINGS;
      setError(message);
      throw new Error(message);
    } finally {
      setLoading(false);
    }
  }, [fetchSettings]);

  return {
    loading,
    error,
    setError,
    fetchSettings,
    fetchPossibleRepositories,
    updateSettings,
  };
}

export default useProprietaryApi;
