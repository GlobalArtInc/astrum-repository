

import { restClient, parseApiError } from '../../../../../../interface/api';
import { CrowdConfig, DEFAULT_CROWD_CONFIG } from './types';

const CROWD_API_URL = '/service/rest/v1/security/atlassian-crowd';

/**
 * Fetch the Crowd configuration, merged over defaults.
 *
 * Pure (no React) so it can be invoked from the Crowd settings machine.
 */
export async function fetchCrowdConfig(): Promise<CrowdConfig> {
  try {
    const data = await restClient.get<CrowdConfig>(CROWD_API_URL);
    return {
      ...DEFAULT_CROWD_CONFIG,
      ...data,
    };
  } catch (err: unknown) {
    const apiError = parseApiError(err);
    throw new Error(apiError.message || 'Failed to load Crowd configuration');
  }
}

/** Save the Crowd configuration. */
export async function saveCrowdConfig(config: CrowdConfig): Promise<void> {
  try {
    await restClient.put(CROWD_API_URL, config);
  } catch (err: unknown) {
    const apiError = parseApiError(err);
    throw new Error(apiError.message || 'Failed to save Crowd configuration');
  }
}

/** Verify connectivity to the configured Crowd server. */
export async function verifyCrowdConnection(config: CrowdConfig): Promise<void> {
  try {
    await restClient.post(`${CROWD_API_URL}/verify-connection`, config);
  } catch (err: unknown) {
    const apiError = parseApiError(err);
    throw new Error(apiError.message || 'Failed to verify Crowd connection');
  }
}

/** Clear the Crowd authentication cache. */
export async function clearCrowdCache(): Promise<void> {
  try {
    await restClient.post(`${CROWD_API_URL}/clear-cache`);
  } catch (err: unknown) {
    const apiError = parseApiError(err);
    throw new Error(apiError.message || 'Failed to clear Crowd cache');
  }
}
