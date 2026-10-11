

/**
 * Capabilities API Hook
 *
 * Migration Status (AgentDev3):
 * - fetchCapabilities: ✅ REST (GET /v1/capabilities)
 * - fetchCapabilityTypes: ✅ REST (GET /v1/capabilities/types)
 * - createCapability: ✅ REST (POST /v1/capabilities)
 * - updateCapability: ✅ REST (PUT /v1/capabilities/{id})
 * - deleteCapability: ✅ REST (DELETE /v1/capabilities/{id})
 * - enableCapability: ✅ REST (PUT /v1/capabilities/{id} with enabled: true)
 * - disableCapability: ✅ REST (PUT /v1/capabilities/{id} with enabled: false)
 */

import { useState, useCallback } from 'react';
import { restClient, parseApiError, urlBuilder } from '../../../../../../interface/api';
import { Capability, CapabilityType, CapabilityFormData } from './types';
import CapabilitiesStrings from '../../../../constants/pages/settings/system/CapabilitiesStrings';

/**
 * Capability type IDs that are deliberately hidden from the new UI even though the REST API
 * still exposes them.
 *
 * `firewall.audit` was migrated to per-repository configuration; the legacy capability surface
 * is preserved server-side (via FirewallCapabilityShimService) only for backward compatibility
 * with hardcoded REST API clients (Terraform, Ansible, scripts). The new UI must never display
 * it: firewall configuration belongs on the repository config page, not the capabilities page.
 *
 * If you remove an entry here, also re-evaluate whether the matching server-side shim should
 * be retired entirely.
 */
const UI_HIDDEN_CAPABILITY_TYPE_IDS: ReadonlySet<string> = new Set(['firewall.audit']);

// =============================================================================
// REST API RESPONSE TYPES
// =============================================================================

interface RestCapability {
  id: string;
  type: string;
  typeName?: string;
  notes?: string;
  enabled: boolean;
  active?: boolean;
  error?: boolean;
  state?: string;
  stateDescription?: string;
  description?: string;
  status?: string;
  properties: Record<string, string>;
  tags?: Record<string, string>;
  isSystem?: boolean;
  disableWarningMessage?: string;
  deleteWarningMessage?: string;
}

interface RestCapabilityType {
  id: string;
  name: string;
  about?: string;
  formFields?: Array<{
    id: string;
    type: string;
    label: string;
    helpText?: string;
    required: boolean;
    disabled?: boolean;
    readOnly?: boolean;
    initialValue?: string | boolean | number | null;
    minValue?: number | null;
    maxValue?: number | null;
    storeApi?: string | null;
    storeFilters?: Record<string, string> | null;
    idMapping?: string | null;
    nameMapping?: string | null;
    allowAutocomplete?: boolean | null;
    attributes?: Record<string, unknown>;
  }>;
}

// =============================================================================
// TRANSFORMERS
// =============================================================================

/**
 * Transform REST capability to internal Capability type.
 * REST API now includes typeName, state, description, and tags.
 */
function restToCapability(rest: RestCapability): Capability {
  return {
    id: rest.id,
    typeId: rest.type,
    typeName: rest.typeName || rest.type,
    enabled: rest.enabled,
    active: rest.active ?? rest.enabled,
    error: rest.error ?? false,
    state: (rest.state as Capability['state']) || (rest.enabled ? 'active' : 'disabled'),
    stateDescription: rest.stateDescription,
    description: rest.description,
    status: rest.status,
    notes: rest.notes,
    properties: rest.properties || {},
    tags: rest.tags,
    isSystem: rest.isSystem ?? false,
    disableWarningMessage: rest.disableWarningMessage ?? undefined,
    deleteWarningMessage: rest.deleteWarningMessage ?? undefined,
  };
}

/**
 * Transform REST capability type to internal CapabilityType.
 */
function restToCapabilityType(rest: RestCapabilityType): CapabilityType {
  return {
    id: rest.id,
    name: rest.name,
    about: rest.about,
    formFields: rest.formFields?.map((field) => ({
      id: field.id,
      type: field.type as any,
      label: field.label,
      helpText: field.helpText,
      required: field.required,
      disabled: field.disabled ?? false,
      readOnly: field.readOnly ?? false,
      initialValue: field.initialValue ?? undefined,
      minValue: field.minValue ?? undefined,
      maxValue: field.maxValue ?? undefined,
      storeApi: field.storeApi ?? undefined,
      storeFilters: field.storeFilters ?? undefined,
      idMapping: field.idMapping ?? undefined,
      nameMapping: field.nameMapping ?? undefined,
      allowAutocomplete: field.allowAutocomplete ?? undefined,
      attributes: field.attributes,
    })),
  };
}

/**
 * Transform form data to REST create/update payload.
 * Property values are coerced to strings — the Java backend
 * throws NullPointerException on null/undefined values (bug jqxh).
 */
function formDataToRestPayload(data: CapabilityFormData) {
  const sanitizedProperties: Record<string, string> = {};
  if (data.properties) {
    for (const [key, value] of Object.entries(data.properties)) {
      sanitizedProperties[key] = value ?? '';
    }
  }
  return {
    id: data.id,
    type: data.typeId,
    enabled: data.enabled,
    notes: data.notes || '',
    properties: sanitizedProperties,
  };
}

// =============================================================================
// HOOK
// =============================================================================

/**
 * Custom hook for Capabilities API operations
 * Uses REST API (v1/capabilities)
 */
export function useCapabilitiesApi() {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  /**
   * Fetch all capabilities.
   *
   * Filters out type IDs in {@link UI_HIDDEN_CAPABILITY_TYPE_IDS} (currently `firewall.audit`).
   * The server-side shim still returns those for REST API backward compatibility, but the new
   * UI must not surface them — see UI_HIDDEN_CAPABILITY_TYPE_IDS for the rationale. This is the
   * single chokepoint: list view, deep-link resolver, and "existing types" used by the type
   * picker all flow through here, so one filter covers every UI surface.
   */
  const fetchCapabilities = useCallback(async (): Promise<Capability[]> => {
    try {
      const url = urlBuilder.capabilities.list();
      const response = await restClient.get<RestCapability[]>(url);
      const all = Array.isArray(response) ? response.map(restToCapability) : [];
      return all.filter((c) => !UI_HIDDEN_CAPABILITY_TYPE_IDS.has(c.typeId));
    } catch (err: unknown) {
      const apiError = parseApiError(err);
      console.error('Failed to fetch capabilities:', err);
      throw new Error(apiError.message || CapabilitiesStrings.API.FAILED_LOAD_CAPABILITIES);
    }
  }, []);

  /**
   * Fetch all capability types
   */
  const fetchCapabilityTypes = useCallback(async (): Promise<CapabilityType[]> => {
    try {
      const url = urlBuilder.capabilities.types();
      const response = await restClient.get<RestCapabilityType[]>(url);
      return Array.isArray(response) ? response.map(restToCapabilityType) : [];
    } catch (err: unknown) {
      const apiError = parseApiError(err);
      console.error('Failed to fetch capability types:', err);
      throw new Error(apiError.message || CapabilitiesStrings.API.FAILED_LOAD_CAPABILITY_TYPES);
    }
  }, []);

  /**
   * Fetch a single capability by ID (includes status field)
   */
  const fetchCapability = useCallback(async (id: string): Promise<Capability> => {
    try {
      const url = urlBuilder.capabilities.get(id);
      const response = await restClient.get<RestCapability>(url);
      return restToCapability(response);
    } catch (err: unknown) {
      const apiError = parseApiError(err);
      console.error('Failed to fetch capability:', err);
      throw new Error(apiError.message || CapabilitiesStrings.API.FAILED_LOAD_CAPABILITY);
    }
  }, []);

  /**
   * Create a new capability
   */
  const createCapability = useCallback(async (data: CapabilityFormData): Promise<Capability> => {
    setLoading(true);
    setError(null);
    try {
      const url = urlBuilder.capabilities.create();
      const payload = formDataToRestPayload(data);
      const response = await restClient.post<RestCapability>(url, payload);
      return restToCapability(response);
    } catch (err: unknown) {
      const apiError = parseApiError(err);
      setError(apiError.message || CapabilitiesStrings.API.FAILED_CREATE_CAPABILITY);
      throw new Error(apiError.message || CapabilitiesStrings.API.FAILED_CREATE_CAPABILITY);
    } finally {
      setLoading(false);
    }
  }, []);

  /**
   * Update an existing capability
   */
  const updateCapability = useCallback(async (data: CapabilityFormData): Promise<Capability> => {
    setLoading(true);
    setError(null);
    try {
      if (!data.id) {
        throw new Error(CapabilitiesStrings.API.CAPABILITY_ID_REQUIRED_UPDATE);
      }
      const url = urlBuilder.capabilities.update(data.id);
      const payload = formDataToRestPayload(data);
      const response = await restClient.put<RestCapability>(url, payload);
      return restToCapability(response);
    } catch (err: unknown) {
      const apiError = parseApiError(err);
      setError(apiError.message || CapabilitiesStrings.API.FAILED_UPDATE_CAPABILITY);
      throw new Error(apiError.message || CapabilitiesStrings.API.FAILED_UPDATE_CAPABILITY);
    } finally {
      setLoading(false);
    }
  }, []);

  /**
   * Delete a capability
   */
  const deleteCapability = useCallback(async (id: string): Promise<void> => {
    setLoading(true);
    setError(null);
    try {
      const url = urlBuilder.capabilities.delete(id);
      await restClient.delete(url);
    } catch (err: unknown) {
      const apiError = parseApiError(err);
      setError(apiError.message);
      throw new Error(apiError.message || CapabilitiesStrings.API.FAILED_DELETE_CAPABILITY);
    } finally {
      setLoading(false);
    }
  }, []);

  /**
   * Enable a capability
   */
  const enableCapability = useCallback(async (capability: Capability): Promise<void> => {
    setLoading(true);
    setError(null);
    try {
      const updateUrl = urlBuilder.capabilities.update(capability.id);
      const payload = formDataToRestPayload({
        id: capability.id,
        typeId: capability.typeId,
        enabled: true,
        notes: capability.notes,
        properties: capability.properties,
      });
      await restClient.put(updateUrl, payload);
    } catch (err: unknown) {
      const apiError = parseApiError(err);
      setError(apiError.message);
      throw new Error(apiError.message || CapabilitiesStrings.API.FAILED_ENABLE_CAPABILITY);
    } finally {
      setLoading(false);
    }
  }, []);

  /**
   * Disable a capability
   */
  const disableCapability = useCallback(async (capability: Capability): Promise<void> => {
    setLoading(true);
    setError(null);
    try {
      const updateUrl = urlBuilder.capabilities.update(capability.id);
      const payload = formDataToRestPayload({
        id: capability.id,
        typeId: capability.typeId,
        enabled: false,
        notes: capability.notes,
        properties: capability.properties,
      });
      await restClient.put(updateUrl, payload);
    } catch (err: unknown) {
      const apiError = parseApiError(err);
      setError(apiError.message);
      throw new Error(apiError.message || CapabilitiesStrings.API.FAILED_DISABLE_CAPABILITY);
    } finally {
      setLoading(false);
    }
  }, []);

  return {
    loading,
    error,
    setError,
    fetchCapabilities,
    fetchCapability,
    fetchCapabilityTypes,
    createCapability,
    updateCapability,
    deleteCapability,
    enableCapability,
    disableCapability,
  };
}

export default useCapabilitiesApi;

