

/**
 * Firewall config tab for Edit repository form – proxy repos only.
 * Same 3-button treatment as Malware Defense: None | Audit | Quarantine with outline on selected.
 * Includes Repository Health Check section (enable/disable).
 */
import React, { useState, useEffect, useCallback } from 'react';
import { Box, Flex, Text, Card, Button, Separator } from '@radix-ui/themes';
import { Shield, ShieldCheck, ExternalLink } from 'lucide-react';
import {
  useFirewallEnable,
  fetchIqAuditStatus,
  fetchPccsSupportedFormats,
  type IqAuditStatus,
} from '../../../../shared/security/useFirewallEnable';
import { ProtectionLevelSelector, type ProtectionLevel } from './ProtectionLevelSelector';
import { useRepositoriesApi } from './useRepositoriesApi';
import { restClient, ENDPOINTS } from '../../../../../../interface/api';
import { useToast } from '../../../../shared';
import { ExtJS } from '../../../../../../interface/ExtJS';
import Permissions from '../../../../../../constants/Permissions';
import RepositoriesPageStrings from '../../../../constants/pages/settings/repository/RepositoriesPageStrings';
import CommonStrings from '../../../../constants/CommonStrings';

export interface RepositoryFirewallConfigTabProps {
  repositoryName: string;
  /** When false, show cross-sell instead of Enable buttons */
  hasFirewallLicense?: boolean;
  /** Called after successful enable (parent can refresh) */
  onEnableSuccess?: () => void;
  /** Show the Firewall section (default true) */
  showFirewall?: boolean;
  /** Show the Health Check section (default true) */
  showHealthCheck?: boolean;
  /**
   * Repository format (e.g. 'maven2', 'npm', 'pypi'). Used to decide whether to offer the
   * PCCS protection level. Optional for backward compatibility — when omitted, PCCS is hidden
   * (matching the pre-PCCS behaviour). Future callers should always pass this through.
   */
  format?: string;
}

interface HealthCheckStatus {
  enabled?: boolean;
  analyzing?: boolean;
}

interface HealthCheckListEntry extends HealthCheckStatus {
  repositoryName: string;
}

export function RepositoryFirewallConfigTab({
  repositoryName,
  hasFirewallLicense = true,
  onEnableSuccess,
  showFirewall = true,
  showHealthCheck = true,
  format,
}: RepositoryFirewallConfigTabProps): JSX.Element {
  const [status, setStatus] = useState<IqAuditStatus | null>(null);
  const [loadingStatus, setLoadingStatus] = useState(true);
  const [healthCheck, setHealthCheck] = useState<HealthCheckStatus | null>(null);
  const [healthCheckLoading, setHealthCheckLoading] = useState(true);
  const [pccsSupported, setPccsSupported] = useState(false);
  const { enableAudit, enableQuarantine, enablePccs, disable, loading, error } =
    useFirewallEnable(repositoryName);
  const { enableHealthCheck, disableHealthCheck } = useRepositoriesApi();
  const toast = useToast();

  // Hide health-check enable/disable actions for users without healthcheck:update (ASTRUM-54212).
  // Use the provider-independent ExtJS.usePermission: the context-based usePermission returns
  // false without a <PermissionsProvider> ancestor, which coreui (self-hosted) never mounts, so
  // the actions would be hidden even from admins. Depend on hasUser so the check re-evaluates
  // once the user and their permissions load (they arrive asynchronously after mount).
  const hasUser = ExtJS.useUser() ?? false;
  const canUpdateHealthCheck = ExtJS.usePermission(
    () => ExtJS.checkPermission(Permissions.HEALTHCHECK.UPDATE),
    [hasUser],
  );
  // Changing the protection level calls disable()/enableAudit()/enableQuarantine()/enablePccs(),
  // all of which mutate firewall config, so gate the selector on repository-admin edit — mirroring
  // the sibling FirewallCard. Without it a repository-admin:read user could pick a mode from the
  // Preview UI and only hit a backend 403, the exact behaviour ASTRUM-54212 removes.
  const canEditFirewall = ExtJS.usePermission(
    () => ExtJS.checkPermission(Permissions.REPOSITORY_ADMIN.EDIT),
    [hasUser],
  );

  const fetchStatus = useCallback(async () => {
    if (!hasFirewallLicense) {
      setStatus(null);
      setLoadingStatus(false);
      return;
    }
    setLoadingStatus(true);
    try {
      const data = await fetchIqAuditStatus(repositoryName);
      setStatus(data);
    } catch {
      setStatus(null);
    } finally {
      setLoadingStatus(false);
    }
  }, [repositoryName, hasFirewallLicense]);

  useEffect(() => {
    fetchStatus();
  }, [fetchStatus]);

  // Resolve whether this repository's format supports PCCS. We do not gate on
  // `hasFirewallLicense` here — the format-capabilities query is a cheap server lookup that
  // is independent of license state, and gating PCCS visibility on format alone matches the
  // legacy ExtJS form behaviour.
  useEffect(() => {
    if (!format) {
      setPccsSupported(false);
      return;
    }
    let cancelled = false;
    fetchPccsSupportedFormats().then((formats) => {
      if (!cancelled) {
        setPccsSupported(formats.includes(format));
      }
    });
    return () => {
      cancelled = true;
    };
  }, [format]);

  useEffect(() => {
    if (!showHealthCheck) {
      setHealthCheck(null);
      setHealthCheckLoading(false);
      return;
    }
    let cancelled = false;
    setHealthCheckLoading(true);
    restClient
      .get<HealthCheckListEntry[]>(ENDPOINTS.HEALTH_CHECK)
      .then((data) => {
        if (cancelled) return;
        const entry = Array.isArray(data)
          ? data.find((r) => r.repositoryName === repositoryName)
          : undefined;
        setHealthCheck(entry ?? null);
      })
      .catch(() => {
        if (!cancelled) setHealthCheck(null);
      })
      .finally(() => {
        if (!cancelled) setHealthCheckLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [repositoryName, showHealthCheck]);

  // Map the 4-mode firewall state down to the protection level the selector understands.
  // We deliberately read `status.mode` (post-migration full state) when available and only
  // fall back to the boolean view for legacy callers — otherwise QUARANTINE and PCCS would
  // both render as 'quarantine'.
  const protectionLevel: ProtectionLevel =
    status?.mode === 'PCCS'
      ? 'pccs'
      : status?.mode === 'QUARANTINE'
        ? 'quarantine'
        : status?.mode === 'AUDIT'
          ? 'audit'
          : status?.enabled && status?.enabledQuarantine
            ? 'quarantine'
            : status?.enabled
              ? 'audit'
              : 'none';

  const handleProtectionChange = async (level: ProtectionLevel) => {
    // Optimistically update the local status so the selector reflects the chosen button
    // before the next fetch round-trips. The server-side mode is the source of truth on
    // subsequent reads.
    const applyOptimistic = (mode: 'DISABLED' | 'AUDIT' | 'QUARANTINE' | 'PCCS') => {
      setStatus((prev) => ({
        repositoryName: prev?.repositoryName ?? repositoryName,
        enabled: mode !== 'DISABLED',
        enabledQuarantine: mode === 'QUARANTINE' || mode === 'PCCS',
        mode,
      }));
      onEnableSuccess?.();
    };
    try {
      if (level === 'none') await disable(() => applyOptimistic('DISABLED'));
      else if (level === 'audit') await enableAudit(() => applyOptimistic('AUDIT'));
      else if (level === 'quarantine') await enableQuarantine(() => applyOptimistic('QUARANTINE'));
      else await enablePccs(() => applyOptimistic('PCCS'));
    } catch {
      // Error shown inline via useFirewallEnable error state
    }
  };

  const handleEnableHealthCheck = async () => {
    try {
      await enableHealthCheck(repositoryName);
      setHealthCheck((p) => (p ? { ...p, enabled: true, analyzing: false } : { enabled: true, analyzing: false }));
      onEnableSuccess?.();
    } catch (err) {
      const message = err instanceof Error ? err.message : RepositoriesPageStrings.FIREWALL_CONFIG.FAILED_ENABLE_HEALTH_CHECK;
      toast.warning(RepositoriesPageStrings.FIREWALL_CONFIG.HEALTH_CHECK_COULD_NOT_ENABLED(repositoryName, message));
    }
  };

  const handleDisableHealthCheck = async () => {
    try {
      await disableHealthCheck(repositoryName);
      setHealthCheck((p) => (p ? { ...p, enabled: false, analyzing: false } : { enabled: false }));
      onEnableSuccess?.();
    } catch (err) {
      const message = err instanceof Error ? err.message : RepositoriesPageStrings.FIREWALL_CONFIG.FAILED_DISABLE_HEALTH_CHECK;
      toast.error(RepositoriesPageStrings.FIREWALL_CONFIG.HEALTH_CHECK_COULD_NOT_DISABLED(repositoryName, message));
    }
  };

  if (loadingStatus) {
    return (
      <Box p="4">
        <Text size="2" color="gray">
          {RepositoriesPageStrings.FIREWALL_CONFIG.LOADING_FIREWALL_STATUS}
        </Text>
      </Box>
    );
  }

  return (
    <Box p="4" className="repository-firewall-config-tab">
      {showFirewall && (
        <Card size="2">
          <Flex direction="column" gap="4">
            <Flex align="center" gap="3">
              <Shield size={24} color="var(--orange-9)" aria-hidden />
              <Box>
                <Text size="4" weight="bold" as="div" mb="1">
                  {RepositoriesPageStrings.FIREWALL_CONFIG.REPOSITORY_FIREWALL}
                </Text>
                <Text size="2" color="gray" as="div">
                  {RepositoriesPageStrings.FIREWALL_CONFIG.PROTECT_PROXY_REPOSITORY_ENABLING_FIREWALL}
                </Text>
              </Box>
            </Flex>

            {error && (
              <Text size="2" color="red">
                {error}
              </Text>
            )}

            {hasFirewallLicense ? (
              canEditFirewall && (
                <Box>
                  <Text size="2" weight="medium" mb="2" as="p">
                    {RepositoriesPageStrings.FIREWALL_CONFIG.PROTECTION_LEVEL}
                  </Text>
                  <ProtectionLevelSelector
                    value={protectionLevel}
                    onChange={handleProtectionChange}
                    disabled={loading}
                    size="2"
                    pccsSupported={pccsSupported}
                  />
                  <Text size="1" color="gray" mt="2">
                    {RepositoriesPageStrings.FIREWALL_CONFIG.AUDIT_TRAIL_CONFIGURATION_CHANGE_HISTORY}
                  </Text>
                </Box>
              )
            ) : (
              <Flex direction="column" gap="3">
                <Text size="2" color="gray">
                  {RepositoriesPageStrings.FIREWALL_CONFIG.PROTECT_REPOSITORY_REPOSITORY_FIREWALL_REQUIRES}
                </Text>
                <Flex gap="2" wrap="wrap">
                  <Button variant="ghost" size="2" asChild>
                    <a
                      href="https://links.globalart.dev/astrum-repository-firewall"
                      target="_blank"
                      rel="noopener noreferrer"
                    >
                      <ExternalLink size={14} />
                      {CommonStrings.ACTIONS.LEARN_MORE}
                    </a>
                  </Button>
                  <Button variant="ghost" size="2" asChild>
                    <a href="https://links.globalart.dev/contact-sales" target="_blank" rel="noopener noreferrer">
                      {RepositoriesPageStrings.FIREWALL_CONFIG.CONTACT_SALES}
                    </a>
                  </Button>
                </Flex>
              </Flex>
            )}
          </Flex>
        </Card>
      )}

      {showFirewall && showHealthCheck && <Separator size="4" my="4" />}

      {showHealthCheck && (
        <Card size="2">
          <Flex direction="column" gap="4">
            <Flex align="center" gap="3">
              <ShieldCheck size={24} color="var(--blue-9)" aria-hidden />
              <Box>
                <Text size="4" weight="bold" as="div" mb="1">
                  {RepositoriesPageStrings.FIREWALL_CONFIG.REPOSITORY_HEALTH_CHECK}
                </Text>
                <Text size="2" color="gray" as="div">
                  {RepositoriesPageStrings.FIREWALL_CONFIG.REPOSITORY_HEALTH_CHECK_ANALYZES_COMPONENTS}
                </Text>
              </Box>
            </Flex>

            {healthCheckLoading ? (
              <Text size="2" color="gray">
                {RepositoriesPageStrings.FIREWALL_CONFIG.LOADING}
              </Text>
            ) : healthCheck?.enabled ? (
              <Flex direction="column" gap="3">
                <Flex align="center" gap="2">
                  <Box
                    style={{
                      width: 8, height: 8, borderRadius: '50%',
                      backgroundColor: healthCheck.analyzing ? 'var(--orange-9)' : 'var(--green-9)',
                    }}
                  />
                  <Text size="2" weight="medium" color={healthCheck.analyzing ? 'orange' : 'green'}>
                    {healthCheck.analyzing ? RepositoriesPageStrings.FIREWALL_CONFIG.ANALYZING : RepositoriesPageStrings.FIREWALL_CONFIG.ENABLED}
                  </Text>
                </Flex>
                {canUpdateHealthCheck && (
                  <Flex gap="2">
                    <Button
                      type="button"
                      variant="soft"
                      color="red"
                      size="2"
                      onClick={handleDisableHealthCheck}
                    >
                      {RepositoriesPageStrings.FIREWALL_CONFIG.DISABLE_HEALTH_CHECK}
                    </Button>
                  </Flex>
                )}
              </Flex>
            ) : (
              <Flex direction="column" gap="3">
                <Flex align="center" gap="2">
                  <Box
                    style={{
                      width: 8, height: 8, borderRadius: '50%',
                      backgroundColor: 'var(--gray-8)',
                    }}
                  />
                  <Text size="2" weight="medium" color="gray">
                    {RepositoriesPageStrings.FIREWALL_CONFIG.DISABLED}
                  </Text>
                </Flex>
                {canUpdateHealthCheck && (
                  <Flex gap="2">
                    <Button
                      type="button"
                      variant="solid"
                      color="blue"
                      size="2"
                      onClick={handleEnableHealthCheck}
                    >
                      {RepositoriesPageStrings.FIREWALL_CONFIG.ENABLE_HEALTH_CHECK}
                    </Button>
                  </Flex>
                )}
              </Flex>
            )}
          </Flex>
        </Card>
      )}
    </Box>
  );
}

export default RepositoryFirewallConfigTab;
