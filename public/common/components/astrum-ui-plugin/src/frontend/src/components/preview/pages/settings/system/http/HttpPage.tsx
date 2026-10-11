
import React, { useState, useCallback } from 'react';
import { Box, Flex, Text } from '@radix-ui/themes';
import { Loader2, Info, ExternalLink, Plus, Trash2 } from 'lucide-react';
import { ExtJS } from '../../../../../../interface/ExtJS';

import { PageHeader } from '../../../../shared';
import {
  SettingsForm,
  SettingsFormSection,
  SettingsCheckbox,
  SettingsTextInput,
  SettingsPasswordInput,
  SettingsSelect,
  SettingsButton,
  SettingsAlert,
} from '../../../../shared/form';
import { useHttpForm } from './useHttpForm';
import {
  HttpConfiguration,
  HttpPageProps,
} from './types';

import './HttpPage.scss';
import HttpStrings from '../../../../constants/pages/settings/system/HttpStrings';
import CommonStrings from '../../../../constants/CommonStrings';
import plural from '../../../../../../interface/plural';

const navigateTo = (path: string) => {
  window.location.hash = path;
};

const ALLOWED_NUMBER_KEYS = new Set([
  'Backspace', 'Delete', 'Tab', 'Escape', 'Enter',
  'ArrowLeft', 'ArrowRight', 'ArrowUp', 'ArrowDown',
  'Home', 'End',
]);

function handleNumberKeyDown(e: React.KeyboardEvent) {
  if (ALLOWED_NUMBER_KEYS.has(e.key)) return;
  if (e.ctrlKey || e.metaKey) return;
  if (!/^[0-9]$/.test(e.key)) {
    e.preventDefault();
  }
}

function Row({ label, value }: { label: string; value: React.ReactNode }) {
  return (
    <Flex className="http-page__row" gap="3">
      <Text size="2" weight="medium" className="http-page__label">{label}</Text>
      <Text size="2">{value}</Text>
    </Flex>
  );
}

/**
 * HttpPage - HTTP Settings configuration page for Preview UI
 *
 * Configures HTTP proxy settings, timeouts, and retry behavior.
 * Uses XState form machine for state management.
 */
export function HttpPage({ className }: HttpPageProps) {
  // XState form hook handles load, save, dirty tracking, toast, validation
  const form = useHttpForm();

  const canUpdate = ExtJS.checkPermission('astrum:settings:update');
  const formData = form.data as HttpConfiguration;
  const isProxyEnabled = formData.httpEnabled || formData.httpsEnabled;

  // Local UI state for the "add non-proxy host" input
  const [newNonProxyHost, setNewNonProxyHost] = useState('');
  const [nonProxyHostError, setNonProxyHostError] = useState<string | null>(null);


  // Add non-proxy host(s) — supports comma-separated paste
  const handleAddNonProxyHost = useCallback(() => {
    const raw = newNonProxyHost;
    if (!raw.trim()) return;

    const allTokens = raw.split(',').map((t) => t.trim()).filter((t) => t.length > 0);
    const validTokens = allTokens.filter((t) => !/\s/.test(t));
    const invalidTokens = allTokens.filter((t) => /\s/.test(t));

    if (invalidTokens.length > 0) {
      setNonProxyHostError(
        `${plural(HttpStrings.PAGE.SKIPPED_INVALID_ENTRIES, invalidTokens.length)} ${invalidTokens.map((t) => `"${t}"`).join(', ')}`
      );
    } else {
      setNonProxyHostError(null);
    }

    if (validTokens.length === 0) return;

    const existing = formData.nonProxyHosts;
    const additions = validTokens.filter((t: string) => !existing.includes(t));
    if (additions.length === 0) {
      setNewNonProxyHost('');
      return;
    }

    const next = [...existing, ...additions];
    form.send({ type: 'UPDATE', name: 'nonProxyHosts', value: next } as any);
    setNewNonProxyHost('');
  }, [newNonProxyHost, formData.nonProxyHosts, form]);

  // Remove non-proxy host by value (display order is sorted, so index would be wrong)
  const handleRemoveNonProxyHost = useCallback((host: string) => {
    form.send({
      type: 'UPDATE',
      name: 'nonProxyHosts',
      value: formData.nonProxyHosts.filter((h: string) => h !== host),
    } as any);
  }, [formData.nonProxyHosts, form]);

  // Handle Enter key for adding non-proxy host
  const handleNonProxyHostKeyDown = useCallback((e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      e.preventDefault();
      handleAddNonProxyHost();
    }
  }, [handleAddNonProxyHost]);

  // Toggle HTTP proxy
  const handleToggleHttpProxy = useCallback(() => {
    const enabling = !formData.httpEnabled;
    form.send({ type: 'UPDATE', name: 'httpEnabled', value: enabling } as any);
    if (!enabling) {
      form.send({ type: 'UPDATE', name: 'httpAuthType', value: '' } as any);
    }
  }, [form, formData.httpEnabled]);

  // Toggle HTTPS proxy
  const handleToggleHttpsProxy = useCallback(() => {
    const enabling = !formData.httpsEnabled;
    form.send({ type: 'UPDATE', name: 'httpsEnabled', value: enabling } as any);
    if (!enabling) {
      form.send({ type: 'UPDATE', name: 'httpsAuthType', value: '' } as any);
    }
  }, [form, formData.httpsEnabled]);

  // Discard changes and clear local UI state
  const handleDiscard = useCallback(() => {
    form.reset();
    setNewNonProxyHost('');
    setNonProxyHostError(null);
  }, [form]);

  // Loading state
  if (form.isLoading) {
    return (
      <Box
        className={`http-page ${className || ''}`.trim()}
        role="status"
        aria-busy="true"
        aria-live="polite"
        data-testid="http-page-loading"
      >
        <Flex align="center" justify="center" gap="3" className="http-page__loading">
          <Loader2 size={24} className="http-page__spinner" />
          <Text size="2">{HttpStrings.PAGE.LOADING_HTTP_SETTINGS}</Text>
        </Flex>
      </Box>
    );
  }

  // Read-only view for users without update permission — full-config parity with legacy HttpReadOnly
  if (!canUpdate) {
    const showHttpProxy = formData.httpEnabled;
    const showHttpsProxy = formData.httpsEnabled;
    const showNonProxyHosts = (showHttpProxy || showHttpsProxy) && formData.nonProxyHosts.length > 0;

    return (
      <Box className={`http-page ${className || ''}`.trim()}>
        <PageHeader
          title={HttpStrings.PAGE.HTTP_SETTINGS}
          description={HttpStrings.PAGE.CONFIGURE_HTTP_PROXY_CONNECTION_SETTINGS}
          breadcrumbs={[
            { label: CommonStrings.LABELS.SETTINGS, onClick: () => navigateTo('#preview/admin/settings') },
            { label: 'HTTP' }
          ]}
        />

        <Box className="http-page__readonly-banner" mb="3">
          <SettingsAlert type="info">
            {HttpStrings.PAGE.YOU_VIEWING_READ_ONLY_VERSION}
          </SettingsAlert>
        </Box>

        <SettingsFormSection title={HttpStrings.PAGE.CONNECTION_SETTINGS}>
          {formData.userAgentSuffix && <Row label={HttpStrings.PAGE.USER_AGENT_SUFFIX} value={formData.userAgentSuffix} />}
          {formData.timeout != null && <Row label={HttpStrings.PAGE.CONNECTION_TIMEOUT} value={formData.timeout} />}
          {formData.retries != null && <Row label={HttpStrings.PAGE.CONNECTION_RETRIES} value={formData.retries} />}
        </SettingsFormSection>

        {showHttpProxy && (
          <SettingsFormSection title={HttpStrings.PAGE.HTTP_PROXY}>
            <Row label={HttpStrings.PAGE.HOST} value={formData.httpHost || '—'} />
            <Row label={HttpStrings.PAGE.PORT} value={formData.httpPort ?? '—'} />
            {formData.httpAuthType === 'username' && (
              <>
                <Row label={HttpStrings.PAGE.AUTH_USERNAME} value={formData.httpAuthUsername || '—'} />
                {formData.httpAuthNtlmHost && <Row label={HttpStrings.PAGE.NTLM_HOST} value={formData.httpAuthNtlmHost} />}
                {formData.httpAuthNtlmDomain && <Row label={HttpStrings.PAGE.NTLM_DOMAIN} value={formData.httpAuthNtlmDomain} />}
              </>
            )}
          </SettingsFormSection>
        )}

        {showHttpsProxy && (
          <SettingsFormSection title={HttpStrings.PAGE.HTTPS_PROXY}>
            <Row label={HttpStrings.PAGE.HOST} value={formData.httpsHost || '—'} />
            <Row label={HttpStrings.PAGE.PORT} value={formData.httpsPort ?? '—'} />
            {formData.httpsAuthType === 'username' && (
              <>
                <Row label={HttpStrings.PAGE.AUTH_USERNAME} value={formData.httpsAuthUsername || '—'} />
                {formData.httpsAuthNtlmHost && <Row label={HttpStrings.PAGE.NTLM_HOST} value={formData.httpsAuthNtlmHost} />}
                {formData.httpsAuthNtlmDomain && <Row label={HttpStrings.PAGE.NTLM_DOMAIN} value={formData.httpsAuthNtlmDomain} />}
              </>
            )}
          </SettingsFormSection>
        )}

        {showNonProxyHosts && (
          <SettingsFormSection title={HttpStrings.PAGE.HOSTS_EXCLUDE_PROXY}>
            <Box className="http-page__host-list">
              {[...formData.nonProxyHosts]
                .sort((a: string, b: string) => a.localeCompare(b, undefined, { sensitivity: 'base' }))
                .map((host: string) => (
                  <Text key={host} as="div" size="2">{host}</Text>
              ))}
            </Box>
          </SettingsFormSection>
        )}
      </Box>
    );
  }

  return (
    <Box className={`http-page ${className || ''}`.trim()}>
      <PageHeader
        title={HttpStrings.PAGE.HTTP_SETTINGS}
        description={HttpStrings.PAGE.CONFIGURE_HTTP_PROXY_CONNECTION_SETTINGS}
        breadcrumbs={[
          { label: CommonStrings.LABELS.SETTINGS, onClick: () => navigateTo('#preview/admin/settings') },
          { label: 'HTTP' }
        ]}
      />

      {/* Content area */}
      <Box className="http-page__content">
        <SettingsForm
          testId="http-form"
          onSubmit={() => form.submit()}
          onCancel={handleDiscard}
          loading={form.isSaving}
          pristine={form.isPristine}
          error={form.saveError || undefined}
          submitAnalyticsId="as-http-save"
        >
        <SettingsFormSection title={HttpStrings.PAGE.CONNECTION_SETTINGS}>
          <SettingsTextInput
            {...form.field('userAgentSuffix')}
            label={HttpStrings.PAGE.USER_AGENT_SUFFIX}
            helpText={HttpStrings.PAGE.CUSTOM_SUFFIX_APPEND_HTTP_USER}
            placeholder={HttpStrings.PAGE.MYCOMPANY_1_0_PLACEHOLDER}
          />

          <SettingsTextInput
            name="timeout"
            label={HttpStrings.PAGE.CONNECTION_TIMEOUT}
            type="number"
            value={formData.timeout != null ? String(formData.timeout) : ''}
            onChange={(value: string) => form.send({ type: 'UPDATE', name: 'timeout', value: value ? parseInt(value, 10) : null } as any)}
            onBlur={() => form.send({ type: 'BLUR', name: 'timeout' } as any)}
            onKeyDown={handleNumberKeyDown}
            error={form.touched?.timeout ? form.validationErrors?.timeout : undefined}
            helpText={HttpStrings.PAGE.TIME_SECONDS_WAIT_CONNECTION_1}
            min={1}
            max={3600}
            placeholder={HttpStrings.PAGE.SYSTEM_DEFAULT_PLACEHOLDER}
          />

          <SettingsTextInput
            name="retries"
            label={HttpStrings.PAGE.CONNECTION_RETRIES}
            type="number"
            value={formData.retries != null ? String(formData.retries) : ''}
            onChange={(value: string) => form.send({ type: 'UPDATE', name: 'retries', value: value ? parseInt(value, 10) : null } as any)}
            onBlur={() => form.send({ type: 'BLUR', name: 'retries' } as any)}
            onKeyDown={handleNumberKeyDown}
            error={form.touched?.retries ? form.validationErrors?.retries : undefined}
            helpText={HttpStrings.PAGE.NUMBER_RETRY_ATTEMPTS_0_10}
            min={0}
            max={10}
            placeholder={HttpStrings.PAGE.SYSTEM_DEFAULT_PLACEHOLDER}
          />
        </SettingsFormSection>

        <SettingsFormSection title={HttpStrings.PAGE.HTTP_PROXY}>
          <SettingsCheckbox
            name="httpEnabled"
            label={HttpStrings.PAGE.ENABLE_HTTP_PROXY}
            checked={formData.httpEnabled}
            onChange={handleToggleHttpProxy}
            description={HttpStrings.PAGE.ROUTE_HTTP_REQUESTS_THROUGH_PROXY}
          />

          {formData.httpEnabled && (
            <>
              <SettingsTextInput
                {...form.field('httpHost')}
                label={HttpStrings.PAGE.HTTP_PROXY_HOST}
                helpText={HttpStrings.PAGE.HOSTNAME_IP_ADDRESS_HTTP_PROXY}
                placeholder="proxy.example.com"
                required
              />

              <SettingsTextInput
                name="httpPort"
                label={HttpStrings.PAGE.HTTP_PROXY_PORT}
                type="number"
                value={formData.httpPort != null ? String(formData.httpPort) : ''}
                onChange={(value: string) => form.send({ type: 'UPDATE', name: 'httpPort', value: value ? parseInt(value, 10) : null } as any)}
                onBlur={() => form.send({ type: 'BLUR', name: 'httpPort' } as any)}
                onKeyDown={handleNumberKeyDown}
                error={form.touched?.httpPort ? form.validationErrors?.httpPort : undefined}
                min={1}
                max={65535}
                required
                placeholder="8080"
              />

              <SettingsFormSection
                key={`http-auth-${formData.httpAuthType !== ''}`}
                title={HttpStrings.PAGE.HTTP_PROXY_AUTHENTICATION}
                collapsible
                defaultCollapsed={formData.httpAuthType === ''}
              >
                <SettingsSelect
                  name="httpAuthType"
                  label={HttpStrings.PAGE.AUTHENTICATION}
                  value={formData.httpAuthType}
                  onChange={(v: string) => form.send({ type: 'UPDATE', name: 'httpAuthType', value: v } as any)}
                  options={[
                    { value: '', label: HttpStrings.PAGE.NO_AUTHENTICATION },
                    { value: 'username', label: HttpStrings.PAGE.USERNAME },
                  ]}
                  helpText={HttpStrings.PAGE.TYPE_AUTHENTICATION_USED_CONNECT_PROXY}
                />

                {formData.httpAuthType === 'username' && (
                  <>
                    <SettingsTextInput
                      {...form.field('httpAuthUsername')}
                      label={HttpStrings.PAGE.USERNAME}
                      helpText={HttpStrings.PAGE.USERNAME_HTTP_PROXY_AUTHENTICATION}
                      required
                      placeholder="proxy-user"
                    />

                    <SettingsPasswordInput
                      {...form.field('httpAuthPassword')}
                      label={HttpStrings.PAGE.PASSWORD}
                      helpText={HttpStrings.PAGE.PASSWORD_HTTP_PROXY_AUTHENTICATION}
                      autoComplete="new-password"
                    />

                    <SettingsTextInput
                      {...form.field('httpAuthNtlmHost')}
                      label={HttpStrings.PAGE.NTLM_HOST}
                      helpText={HttpStrings.PAGE.WINDOWS_HOST_NTLM_AUTHENTICATION_OPTIONAL}
                      placeholder={HttpStrings.PAGE.WORKSTATION01_PLACEHOLDER}
                    />

                    <SettingsTextInput
                      {...form.field('httpAuthNtlmDomain')}
                      label={HttpStrings.PAGE.NTLM_DOMAIN}
                      helpText={HttpStrings.PAGE.WINDOWS_DOMAIN_NTLM_AUTHENTICATION_OPTIONAL}
                      placeholder={HttpStrings.PAGE.COMPANY_PLACEHOLDER}
                    />
                  </>
                )}
              </SettingsFormSection>
            </>
          )}
        </SettingsFormSection>

        <SettingsFormSection title={HttpStrings.PAGE.HTTPS_PROXY}>
          <SettingsCheckbox
            name="httpsEnabled"
            label={HttpStrings.PAGE.ENABLE_HTTPS_PROXY}
            checked={formData.httpsEnabled}
            onChange={handleToggleHttpsProxy}
            description={HttpStrings.PAGE.ROUTE_HTTPS_REQUESTS_THROUGH_PROXY}
          />

          {formData.httpsEnabled && (
            <>
              <SettingsTextInput
                {...form.field('httpsHost')}
                label={HttpStrings.PAGE.HTTPS_PROXY_HOST}
                helpText={HttpStrings.PAGE.HOSTNAME_IP_ADDRESS_HTTPS_PROXY}
                placeholder="proxy.example.com"
                required
              />

              <SettingsTextInput
                name="httpsPort"
                label={HttpStrings.PAGE.HTTPS_PROXY_PORT}
                type="number"
                value={formData.httpsPort != null ? String(formData.httpsPort) : ''}
                onChange={(value: string) => form.send({ type: 'UPDATE', name: 'httpsPort', value: value ? parseInt(value, 10) : null } as any)}
                onBlur={() => form.send({ type: 'BLUR', name: 'httpsPort' } as any)}
                onKeyDown={handleNumberKeyDown}
                error={form.touched?.httpsPort ? form.validationErrors?.httpsPort : undefined}
                min={1}
                max={65535}
                required
                placeholder="8443"
              />

              <SettingsFormSection
                key={`https-auth-${formData.httpsAuthType !== ''}`}
                title={HttpStrings.PAGE.HTTPS_PROXY_AUTHENTICATION}
                collapsible
                defaultCollapsed={formData.httpsAuthType === ''}
              >
                <SettingsSelect
                  name="httpsAuthType"
                  label={HttpStrings.PAGE.AUTHENTICATION}
                  value={formData.httpsAuthType}
                  onChange={(v: string) => form.send({ type: 'UPDATE', name: 'httpsAuthType', value: v } as any)}
                  options={[
                    { value: '', label: HttpStrings.PAGE.NO_AUTHENTICATION },
                    { value: 'username', label: HttpStrings.PAGE.USERNAME },
                  ]}
                  helpText={HttpStrings.PAGE.TYPE_AUTHENTICATION_USED_CONNECT_PROXY}
                />

                {formData.httpsAuthType === 'username' && (
                  <>
                    <SettingsTextInput
                      {...form.field('httpsAuthUsername')}
                      label={HttpStrings.PAGE.USERNAME}
                      required
                      placeholder="proxy-user"
                    />

                    <SettingsPasswordInput
                      {...form.field('httpsAuthPassword')}
                      label={HttpStrings.PAGE.PASSWORD}
                      autoComplete="new-password"
                    />

                    <SettingsTextInput
                      {...form.field('httpsAuthNtlmHost')}
                      label={HttpStrings.PAGE.NTLM_HOST}
                      helpText={HttpStrings.PAGE.WINDOWS_HOST_NTLM_AUTHENTICATION_OPTIONAL}
                      placeholder={HttpStrings.PAGE.WORKSTATION01_PLACEHOLDER}
                    />

                    <SettingsTextInput
                      {...form.field('httpsAuthNtlmDomain')}
                      label={HttpStrings.PAGE.NTLM_DOMAIN}
                      helpText={HttpStrings.PAGE.WINDOWS_DOMAIN_NTLM_AUTHENTICATION_OPTIONAL}
                      placeholder={HttpStrings.PAGE.COMPANY_PLACEHOLDER}
                    />
                  </>
                )}
              </SettingsFormSection>
            </>
          )}
        </SettingsFormSection>

        {/* Non-Proxy Hosts */}
        {isProxyEnabled && (
          <SettingsFormSection
            title={
              formData.nonProxyHosts.length > 0
                ? HttpStrings.PAGE.HOSTS_EXCLUDE_PROXY_2(formData.nonProxyHosts.length)
                : HttpStrings.PAGE.HOSTS_EXCLUDE_PROXY
            }
          >
            <Text size="2" className="http-page__section-description">
              {HttpStrings.PAGE.SPECIFY_HOSTS_SHOULD_BYPASS_PROXY}
            </Text>

            <Flex gap="2" align="end" className="http-page__add-host">
              <Box className="http-page__add-host-input">
                <SettingsTextInput
                  name="newNonProxyHost"
                  label={HttpStrings.PAGE.HOST_PATTERN}
                  value={newNonProxyHost}
                  onChange={(v: string) => { setNewNonProxyHost(v); setNonProxyHostError(null); }}
                  onKeyDown={handleNonProxyHostKeyDown}
                  placeholder="*.example.com"
                  helpText={HttpStrings.PAGE.ENTER_HOSTNAME_PATTERN_EXCLUDE}
                  error={nonProxyHostError || undefined}
                />
              </Box>
              <SettingsButton
                type="button"
                variant="secondary"
                onClick={handleAddNonProxyHost}
                disabled={!newNonProxyHost.trim()}
                className="http-page__add-button"
                icon={Plus}
              >
                {HttpStrings.PAGE.ADD}
              </SettingsButton>
            </Flex>

            {formData.nonProxyHosts.length > 0 && (
              <Box className="http-page__host-list">
                {[...formData.nonProxyHosts]
                  .sort((a: string, b: string) => a.localeCompare(b, undefined, { sensitivity: 'base' }))
                  .map((host: string) => (
                    <Flex key={host} align="center" justify="between" className="http-page__host-item">
                      <Text size="2">{host}</Text>
                      <button
                        type="button"
                        className="http-page__remove-button"
                        onClick={() => handleRemoveNonProxyHost(host)}
                        aria-label={HttpStrings.PAGE.REMOVE(host)}
                      >
                        <Trash2 size={16} />
                      </button>
                    </Flex>
                ))}
              </Box>
            )}
          </SettingsFormSection>
        )}

          {/* Help Section */}
          <Box className="http-page__help">
            <Flex align="center" gap="2" className="http-page__help-header">
              <Info size={16} />
              <Text size="2" weight="medium">{HttpStrings.PAGE.ABOUT_HTTP_SETTINGS}</Text>
            </Flex>
            <Text size="2" className="http-page__help-text">
              {HttpStrings.PAGE.HTTP_SETTINGS_CONTROL_HOW_ASTRUM}
            </Text>
            <Text size="2" className="http-page__help-text">
              {HttpStrings.PAGE.SEE_DOCUMENTATION(
                <a
                  href="http://links.globalart.dev/products/astrum/docs/http-request-and-proxy-settings"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="http-page__help-link"
                >
                  {HttpStrings.PAGE.DOCUMENTATION_LINK}
                  <ExternalLink size={12} />
                </a>
              )}
            </Text>
          </Box>
        </SettingsForm>
      </Box>
    </Box>
  );
}

export default HttpPage;
