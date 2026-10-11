

import { useState, useCallback } from 'react';
import { APIConstants } from '../../../../../../constants/APIConstants';
import { restClient, parseApiError } from '../../../../../../interface/api';
import { SamlConfiguration } from './types';
import SamlStrings from '../../../../constants/pages/settings/security/SamlStrings';

const SAML_URL = APIConstants.REST.PUBLIC.SAML;

export function useSamlApi() {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchConfiguration = useCallback(async (): Promise<SamlConfiguration | null> => {
    setLoading(true);
    setError(null);

    try {
      const data = await restClient.get<SamlConfiguration>(SAML_URL);
      return data;
    } catch (err: any) {
      // 404 means no configuration exists yet - not an error
      if (err.response?.status === 404) {
        return null;
      }
      const apiError = parseApiError(err);
      const message = apiError.message || SamlStrings.API.FAILED_LOAD_SAML_CONFIGURATION;
      setError(message);
      throw err;
    } finally {
      setLoading(false);
    }
  }, []);

  const saveConfiguration = useCallback(async (config: SamlConfiguration): Promise<void> => {
    setLoading(true);
    setError(null);

    try {
      await restClient.put(SAML_URL, config);
    } catch (err: any) {
      const apiError = parseApiError(err);
      const message = apiError.message || SamlStrings.API.FAILED_SAVE_SAML_CONFIGURATION;
      setError(message);
      throw err;
    } finally {
      setLoading(false);
    }
  }, []);

  const deleteConfiguration = useCallback(async (): Promise<void> => {
    setLoading(true);
    setError(null);

    try {
      await restClient.delete(SAML_URL);
    } catch (err: any) {
      const apiError = parseApiError(err);
      const message = apiError.message || SamlStrings.API.FAILED_DELETE_SAML_CONFIGURATION;
      setError(message);
      throw err;
    } finally {
      setLoading(false);
    }
  }, []);

  const getMetadataUrl = useCallback((): string => {
    return `${SAML_URL}/metadata`;
  }, []);

  return {
    loading,
    error,
    setError,
    fetchConfiguration,
    saveConfiguration,
    deleteConfiguration,
    getMetadataUrl,
  };
}


