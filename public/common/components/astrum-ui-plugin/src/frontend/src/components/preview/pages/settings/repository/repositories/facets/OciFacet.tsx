

import React from 'react';
import { Box, Button, Flex, Heading, RadioGroup, Text } from '@radix-ui/themes';
import { Plus, Trash2 } from 'lucide-react';

import {
  SettingsFormSection,
  SettingsTextInput,
  SettingsCheckbox,
  SettingsSelect,
} from '../../../../../shared/form';

import {
  RepositoryFormData,
  RepositoryFormErrors,
  OciCosignAttributes,
  OciProxyAttributes,
} from '../types';
import CommonStrings from '../../../../../constants/CommonStrings';
import UIStrings from '../../../../../../../constants/pages/admin/repository/RepositoriesStrings';

interface OciFacetProps {
  formData: RepositoryFormData;
  onNestedChange: <K extends keyof RepositoryFormData>(
    key: K,
    updates: Partial<RepositoryFormData[K]>
  ) => void;
  errors?: RepositoryFormErrors & Record<string, unknown>;
  repoType: 'hosted' | 'proxy' | 'group';
}

/** The form machine reports errors under dotted keys such as `oci.httpPort`. */
function flatError(errors: OciFacetProps['errors'], key: string): string | undefined {
  const value = errors?.[key];
  return typeof value === 'string' ? value : undefined;
}

const COSIGN_ENFORCEMENT_OPTIONS = [
  { value: 'NONE', label: UIStrings.OCI.OFF_NO_COSIGN_ENFORCEMENT },
  { value: 'KEYLESS', label: UIStrings.OCI.KEYLESS_REQUIRE_COSIGN_SIGNATURE },
];

/**
 * OciFacet — OCI-specific repository configuration for the Preview UI.
 *
 * Branches by `repoType`:
 *   - hosted/group: connector + cosign sections only
 *   - proxy: additionally renders the OCI Registry Index radio (REGISTRY vs
 *     CUSTOM), the indexUrl field with truststore toggle, and the foreign-layer
 *     caching toggle + URL allow-list rows. This mirrors OciProxyFacet.js so
 *     proxy admins get the same surface in Preview as in classic.
 *
 * Fields:
 * - HTTP / HTTPS Connector Port
 * - Subdomain (subdomain-based routing)
 * - Path-Based Routing toggle
 * - Force Basic Authentication
 * - Cosign Keyless Policy (enforcement mode + identity/issuer regexes)
 * - [proxy only] OCI Registry Index source / index URL / truststore
 * - [proxy only] Foreign Layer Caching + URL allow-list
 */
export function OciFacet({
  formData,
  onNestedChange,
  errors,
  repoType,
}: OciFacetProps) {
  const cosign: OciCosignAttributes = formData.oci?.cosign ?? { enforcement: 'NONE' };
  const enforcement = cosign.enforcement ?? 'NONE';
  const keylessActive = enforcement === 'KEYLESS';

  const updateCosign = (patch: Partial<OciCosignAttributes>) => {
    const next: OciCosignAttributes = {
      enforcement,
      identityRegex: cosign.identityRegex,
      issuerRegex: cosign.issuerRegex,
      ...patch,
    };
    onNestedChange('oci', { cosign: next });
  };

  return (
    <>
      <SettingsFormSection
        title={UIStrings.OCI.OCI_REGISTRY_CONNECTORS}
        description={UIStrings.OCI.CONFIGURE_V2_DISTRIBUTION_SPEC_CONNECTORS}
      >
        <SettingsCheckbox
          name="oci-pathEnabled"
          label={UIStrings.OCI.ENABLE_PATH_BASED_ROUTING}
          checked={formData.oci?.pathEnabled ?? true}
          onChange={(checked) => onNestedChange('oci', { pathEnabled: checked })}
          description={UIStrings.OCI.EXPOSE_OCI_REPOSITORY_UNDER_PATH}
        />

        <SettingsTextInput
          name="oci-httpPort"
          label={UIStrings.OCI.HTTP_CONNECTOR}
          value={formData.oci?.httpPort?.toString() || ''}
          onChange={(value) => onNestedChange('oci', {
            httpPort: value ? parseInt(value, 10) : null,
          })}
          helpText={UIStrings.OCI.CREATE_HTTP_CONNECTOR_SPECIFIED_PORT}
          placeholder={CommonStrings.MESSAGES.EXAMPLE('8082')}
          type="number"
          error={flatError(errors, 'oci.httpPort')}
        />

        <SettingsTextInput
          name="oci-httpsPort"
          label={UIStrings.OCI.HTTPS_CONNECTOR}
          value={formData.oci?.httpsPort?.toString() || ''}
          onChange={(value) => onNestedChange('oci', {
            httpsPort: value ? parseInt(value, 10) : null,
          })}
          helpText={UIStrings.OCI.CREATE_HTTPS_CONNECTOR_SPECIFIED_PORT}
          placeholder={CommonStrings.MESSAGES.EXAMPLE('8083')}
          type="number"
          error={flatError(errors, 'oci.httpsPort')}
        />

        <SettingsTextInput
          name="oci-subdomain"
          label={UIStrings.OCI.SUBDOMAIN}
          value={formData.oci?.subdomain || ''}
          onChange={(value) => onNestedChange('oci', { subdomain: value || null })}
          helpText={UIStrings.OCI.USE_SPECIFIED_SUBDOMAIN_ACCESS_OCI}
          placeholder={CommonStrings.MESSAGES.EXAMPLE('oci-hosted')}
        />

        <SettingsCheckbox
          name="oci-forceBasicAuth"
          label={UIStrings.OCI.FORCE_BASIC_AUTHENTICATION}
          checked={formData.oci?.forceBasicAuth ?? false}
          onChange={(checked) => onNestedChange('oci', { forceBasicAuth: checked })}
          description={UIStrings.OCI.REQUIRE_AUTHENTICATION_EVEN_ANONYMOUS_ACCESS}
        />
      </SettingsFormSection>

      {repoType === 'proxy' && (
        <OciProxySection
          formData={formData}
          onNestedChange={onNestedChange}
          errors={errors}
        />
      )}

      <SettingsFormSection
        title={UIStrings.OCI.COSIGN_KEYLESS_POLICY}
        description={UIStrings.OCI.REQUIRE_COSIGN_KEYLESS_SIGNATURES_FULCIO}
      >
        <SettingsSelect
          name="oci-cosign-enforcement"
          label={UIStrings.OCI.ENFORCEMENT_MODE}
          value={enforcement}
          onChange={(value) => updateCosign({ enforcement: (value || 'NONE') as OciCosignAttributes['enforcement'] })}
          options={COSIGN_ENFORCEMENT_OPTIONS}
          helpText={UIStrings.OCI.OFF_DISABLES_ENFORCEMENT_KEYLESS_REQUIRES}
        />

        {keylessActive && (
          <>
            <Text as="p" size="2" color="amber" className="as-oci-cosign-keyless-warning">
              {UIStrings.OCI.KEYLESS_ENFORCEMENT_CONFIGURED_BUT_SIGNATURE}
            </Text>

            <SettingsTextInput
              name="oci-cosign-identityRegex"
              label={UIStrings.OCI.IDENTITY_REGEX}
              value={cosign.identityRegex || ''}
              onChange={(value) => updateCosign({ identityRegex: value || undefined })}
              helpText={UIStrings.OCI.REGEX_MATCHED_AGAINST_FULCIO_CERTIFICATE}
              placeholder={CommonStrings.MESSAGES.EXAMPLE('^https://github\\.com/acme/.*$')}
              required={keylessActive}
              error={
                keylessActive && !(cosign.identityRegex?.trim())
                  ? UIStrings.OCI.IDENTITY_REGEX_REQUIRED_WHEN_KEYLESS
                  : errors?.oci?.cosign?.identityRegex
              }
            />

            <SettingsTextInput
              name="oci-cosign-issuerRegex"
              label={UIStrings.OCI.ISSUER_REGEX}
              value={cosign.issuerRegex || ''}
              onChange={(value) => updateCosign({ issuerRegex: value || undefined })}
              helpText={UIStrings.OCI.REGEX_MATCHED_AGAINST_OIDC_ISSUER}
              placeholder={CommonStrings.MESSAGES.EXAMPLE('^https://token\\.actions\\.githubusercontent\\.com$')}
              required={keylessActive}
              error={
                keylessActive && !(cosign.issuerRegex?.trim())
                  ? UIStrings.OCI.ISSUER_REGEX_REQUIRED_WHEN_KEYLESS
                  : errors?.oci?.cosign?.issuerRegex
              }
            />
          </>
        )}
      </SettingsFormSection>
    </>
  );
}

interface OciProxySectionProps {
  formData: RepositoryFormData;
  onNestedChange: OciFacetProps['onNestedChange'];
  errors?: RepositoryFormErrors;
}

/**
 * Proxy-only OCI subform. Mirrors OciProxyFacet.js: index source radio, custom
 * index URL with truststore toggle, foreign-layer caching toggle, and an editable
 * regex allow-list for foreign-layer URLs.
 *
 * Whitelist rows are stored as a flat string[] under
 * formData.oci.ociProxy.foreignLayerUrlWhitelist. Empty array (or absent) means
 * "deny all foreign-layer fetches" — when caching is first enabled we seed a
 * single ".*" row to match the classic ExtJS UX.
 */
function OciProxySection({ formData, onNestedChange, errors }: OciProxySectionProps) {
  const ociProxy: OciProxyAttributes = formData.oci?.ociProxy ?? {};
  const indexType = ociProxy.indexType ?? 'REGISTRY';
  const cacheForeignLayers = ociProxy.cacheForeignLayers ?? false;
  const whitelist = ociProxy.foreignLayerUrlWhitelist ?? [];

  const patchProxy = (patch: Partial<OciProxyAttributes>) => {
    onNestedChange('oci', {
      ociProxy: { ...ociProxy, ...patch },
    });
  };

  const updateRow = (index: number, value: string) => {
    const next = whitelist.slice();
    next[index] = value;
    patchProxy({ foreignLayerUrlWhitelist: next });
  };
  const removeRow = (index: number) => {
    const next = whitelist.slice();
    next.splice(index, 1);
    patchProxy({ foreignLayerUrlWhitelist: next });
  };
  const addRow = () => {
    const next = whitelist.concat([whitelist.length === 0 ? '.*' : '']);
    patchProxy({ foreignLayerUrlWhitelist: next });
  };

  const onIndexTypeChange = (next: 'REGISTRY' | 'CUSTOM') => {
    // Match the classic UI: switching back to REGISTRY clears the custom URL
    // so a stale value can't sneak through; switching to CUSTOM preserves
    // whatever's already stored so the field is editable.
    patchProxy(
      next === 'REGISTRY'
        ? { indexType: next, indexUrl: null, useTrustStoreForIndexAccess: false }
        : { indexType: next }
    );
  };

  // Auto-seed a starter ".*" row the first time caching is turned on, mirroring
  // the classic UI where toggling Foreign Layer Caching adds a default row.
  const onCacheToggle = (checked: boolean) => {
    if (checked && whitelist.length === 0) {
      patchProxy({ cacheForeignLayers: true, foreignLayerUrlWhitelist: ['.*'] });
    } else {
      patchProxy({ cacheForeignLayers: checked });
    }
  };

  return (
    <SettingsFormSection
      title={UIStrings.OCI.OCI_PROXY}
      description={UIStrings.OCI.INDEX_SOURCE_FOREIGN_LAYER_CACHING}
    >
      <Box mb="3">
        <Text as="div" weight="medium" size="2" mb="1">
          {UIStrings.OCI.OCI_REGISTRY_INDEX}
        </Text>
        <RadioGroup.Root
          value={indexType}
          onValueChange={(v) => onIndexTypeChange(v as 'REGISTRY' | 'CUSTOM')}
          name="oci-proxy-indexType"
        >
          <Flex gap="3" direction="column">
            <Text as="label" size="2">
              <Flex gap="2" align="center">
                <RadioGroup.Item value="REGISTRY" />
                {UIStrings.OCI.USE_PROXY_REGISTRY_SPECIFIED_ABOVE}
              </Flex>
            </Text>
            <Text as="label" size="2">
              <Flex gap="2" align="center">
                <RadioGroup.Item value="CUSTOM" />
                {UIStrings.OCI.CUSTOM_INDEX}
              </Flex>
            </Text>
          </Flex>
        </RadioGroup.Root>
      </Box>

      {indexType === 'CUSTOM' && (
        <>
          <SettingsTextInput
            name="oci-proxy-indexUrl"
            label={UIStrings.OCI.OCI_REGISTRY_INDEX_URL}
            value={ociProxy.indexUrl || ''}
            onChange={(value) => patchProxy({ indexUrl: value || null })}
            helpText={UIStrings.OCI.LOCATION_OCI_REGISTRY_INDEX}
            placeholder="https://index.example.com"
            required
            error={
              !(ociProxy.indexUrl?.trim())
                ? UIStrings.OCI.INDEX_URL_REQUIRED_WHEN_CUSTOM
                : errors?.oci?.ociProxy?.indexUrl
            }
          />
          <SettingsCheckbox
            name="oci-proxy-useTrustStoreForIndexAccess"
            label={UIStrings.OCI.USE_ASTRUM_TRUSTSTORE_HTTPS_INDEX}
            checked={ociProxy.useTrustStoreForIndexAccess ?? false}
            onChange={(checked) => patchProxy({ useTrustStoreForIndexAccess: checked })}
            description={UIStrings.OCI.VALIDATE_INDEX_SERVER_CERTIFICATE_AGAINST}
          />
        </>
      )}

      <SettingsCheckbox
        name="oci-proxy-cacheForeignLayers"
        label={UIStrings.OCI.FOREIGN_LAYER_CACHING}
        checked={cacheForeignLayers}
        onChange={onCacheToggle}
        description={UIStrings.OCI.ALLOW_ASTRUM_REPOSITORY_MANAGER_DOWNLOAD}
      />

      {cacheForeignLayers && (
        <Box mt="2">
          <Heading as="h4" size="2" mb="1">
            {UIStrings.OCI.FOREIGN_LAYER_ALLOWED_URLS}
          </Heading>
          <Text as="div" size="1" color="gray" mb="2">
            {UIStrings.OCI.REGULAR_EXPRESSIONS_USED_IDENTIFY_URLS}
          </Text>
          {whitelist.map((value, idx) => (
            <Flex key={`oci-proxy-whitelist-row-${idx}`} gap="2" align="center" mb="2">
              <Box flexGrow="1">
                <SettingsTextInput
                  name={`oci-proxy-whitelist-${idx}`}
                  label={idx === 0 ? UIStrings.OCI.ALLOWED_URL_PATTERN : ''}
                  value={value}
                  onChange={(v) => updateRow(idx, v)}
                  placeholder=".*"
                />
              </Box>
              <Button
                type="button"
                variant="soft"
                color="red"
                size="1"
                onClick={() => removeRow(idx)}
                aria-label={UIStrings.OCI.REMOVE_URL_PATTERN(idx + 1)}
              >
                <Trash2 size={14} />
              </Button>
            </Flex>
          ))}
          <Button
            type="button"
            variant="soft"
            size="2"
            onClick={addRow}
            aria-label={UIStrings.OCI.ADD_URL_PATTERN_ARIA}
          >
            <Plus size={14} />
            {UIStrings.OCI.ADD_URL_PATTERN}
          </Button>
        </Box>
      )}
    </SettingsFormSection>
  );
}

export default OciFacet;
