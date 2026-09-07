

import { APIConstants } from '../../../../../../constants/APIConstants';
import { restClient, parseApiError } from '../../../../../../interface/api';
import { SamlConfiguration } from './types';

const SAML_URL = APIConstants.REST.PUBLIC.SAML;

/**
 * Fetch the SAML configuration.
 *
 * Returns `null` when the server responds 404 — meaning no configuration
 * exists yet (not an error). Pure (no React) so it can back the form machine's
 * load service.
 */
export async function fetchSamlConfiguration(): Promise<SamlConfiguration | null> {
  try {
    return await restClient.get<SamlConfiguration>(SAML_URL);
  } catch (err: unknown) {
    if ((err as { response?: { status?: number } })?.response?.status === 404) {
      return null;
    }
    const apiError = parseApiError(err);
    throw new Error(apiError.message || 'Failed to load SAML configuration');
  }
}

/** Save the SAML configuration. */
export async function saveSamlConfiguration(config: SamlConfiguration): Promise<void> {
  try {
    await restClient.put(SAML_URL, config);
  } catch (err: unknown) {
    const apiError = parseApiError(err);
    throw new Error(apiError.message || 'Failed to save SAML configuration');
  }
}

/** Delete the SAML configuration. */
export async function deleteSamlConfiguration(): Promise<void> {
  try {
    await restClient.delete(SAML_URL);
  } catch (err: unknown) {
    const apiError = parseApiError(err);
    throw new Error(apiError.message || 'Failed to delete SAML configuration');
  }
}

/** Relative path to the SAML Service Provider metadata endpoint. */
export function getSamlMetadataUrl(): string {
  return `${SAML_URL}/metadata`;
}
