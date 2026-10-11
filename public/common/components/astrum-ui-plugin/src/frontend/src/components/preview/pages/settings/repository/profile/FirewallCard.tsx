

import React, { useState, useEffect, useRef } from 'react';
import { Box, Card, Flex, Text, Badge, Button, Tooltip, Grid, Separator } from '@radix-ui/themes';
import { Shield, ShieldCheck, Info, ExternalLink, ShieldOff } from 'lucide-react';
import type { FirewallData, IqCapabilities, MalwareCleanupSummary } from './types';
import { useFirewallEnable, } from '../../../../shared/security/useFirewallEnable';
import {
  setMalwareRemediatorEnabledForRepository,
  type MalwareRemediatorMode,
} from '../../../../shared/security/malwareRemediatorTask';
import { useToast } from '../../../../shared/Toast';
import { ExtJS } from '../../../../../../interface/ExtJS';
import Permissions from '../../../../../../constants/Permissions';
import RepositoryProfileStrings from '../../../../constants/pages/settings/repository/RepositoryProfileStrings';
import CommonStrings from '../../../../constants/CommonStrings';

export interface FirewallCardProps {
  repositoryName: string;
  firewall: FirewallData | null;
  malwareCleanupSummary: MalwareCleanupSummary | null;
  iqCapabilities: IqCapabilities | null;
  isSupported: boolean;
  refresh: () => void;
  onSelectTab?: (tab: string) => void;
}

type ProtectionLevel = 'unprotected' | 'audit' | 'quarantine';

const RADIX_COLOR: Record<ProtectionLevel, 'red' | 'amber' | 'green'> = {
  unprotected: 'red',
  audit: 'amber',
  quarantine: 'green',
};

const CSS_COLOR: Record<ProtectionLevel, string> = {
  unprotected: 'var(--red-9)',
  audit: 'var(--amber-9)',
  quarantine: 'var(--green-9)',
};

const LABELS: Record<ProtectionLevel, string> = {
  unprotected: RepositoryProfileStrings.FIREWALL_CARD.UNPROTECTED,
  audit: RepositoryProfileStrings.FIREWALL_CARD.AUDIT,
  quarantine: RepositoryProfileStrings.FIREWALL_CARD.QUARANTINE,
};

const DESCRIPTIONS: Record<ProtectionLevel, string> = {
  unprotected: RepositoryProfileStrings.FIREWALL_CARD.NO_PROTECTION_COMPONENTS_NOT_EVALUATED,
  audit: RepositoryProfileStrings.FIREWALL_CARD.EVALUATE_COMPONENTS_LOG_VIOLATIONS,
  quarantine: RepositoryProfileStrings.FIREWALL_CARD.EVALUATE_BLOCK_RISKY_COMPONENTS,
};

const MALWARE_TASK_LABELS: Record<MalwareRemediatorMode, string> = {
  disabled: RepositoryProfileStrings.FIREWALL_CARD.DISABLED,
  audit: RepositoryProfileStrings.FIREWALL_CARD.AUDIT,
  delete: CommonStrings.ACTIONS.DELETE,
};

const MALWARE_TASK_DESCRIPTIONS: Record<MalwareRemediatorMode, string> = {
  disabled: RepositoryProfileStrings.FIREWALL_CARD.NO_SCHEDULED_SCANNING_MALICIOUS_PACKAGES,
  audit: RepositoryProfileStrings.FIREWALL_CARD.SCAN_RECORD_MALICIOUS_PACKAGES_BUT,
  delete: RepositoryProfileStrings.FIREWALL_CARD.SCAN_RECORD_AUTOMATICALLY_REMOVE_MALICIOUS,
};

const MALWARE_TASK_COLORS: Record<MalwareRemediatorMode, string> = {
  disabled: 'var(--gray-9)',
  audit: 'var(--amber-9)',
  delete: 'var(--green-9)',
};

function deriveProtectionLevel(firewall: FirewallData | null): ProtectionLevel {
  if (!firewall) return 'unprotected';
  if (firewall.quarantineEnabled) return 'quarantine';
  const msg = firewall.message?.toLowerCase() ?? '';
  if (msg.includes('quarantine')) return 'quarantine';
  if (msg.includes('audit')) return 'audit';
  if (msg.includes('enabled')) return 'audit';
  if (firewall.enabled) return 'audit';
  return 'unprotected';
}

export function FirewallCard({
  repositoryName,
  firewall,
  malwareCleanupSummary,
  iqCapabilities,
  isSupported,
  refresh,
  onSelectTab,
}: FirewallCardProps): JSX.Element {
  const toast = useToast();
  const { enableAudit, enableQuarantine, disable, loading: firewallLoading } = useFirewallEnable(repositoryName);
  const [taskLoading, setTaskLoading] = useState(false);

  // Hide firewall protection-level controls without repository-admin edit, and
  // the malware-remediator task controls without tasks:create (ASTRUM-54212). coreui never
  // mounts a <PermissionsProvider>, so context usePermission returns false for everyone; use
  // the provider-independent ExtJS.usePermission.
  const hasUser = ExtJS.useUser() ?? false;
  const canEditFirewall = ExtJS.usePermission(
    () => ExtJS.checkPermission(Permissions.REPOSITORY_ADMIN.EDIT),
    [hasUser],
  );
  const canManageMalwareTask = ExtJS.usePermission(
    () => ExtJS.checkPermission(Permissions.TASKS.CREATE),
    [hasUser],
  );

  const derivedLevel = deriveProtectionLevel(firewall);
  const [optimisticLevel, setOptimisticLevel] = useState<ProtectionLevel | null>(null);
  const protectionLevel = optimisticLevel ?? derivedLevel;
  const refreshTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  // Clear optimistic override once the server-derived level catches up
  useEffect(() => {
    if (optimisticLevel && derivedLevel === optimisticLevel) {
      setOptimisticLevel(null);
    }
  }, [derivedLevel, optimisticLevel]);

  // Cleanup timer on unmount
  useEffect(() => {
    return () => {
      if (refreshTimerRef.current) clearTimeout(refreshTimerRef.current);
    };
  }, []);

  const handleProtectionChange = async (level: ProtectionLevel) => {
    if (level === protectionLevel || firewallLoading) return;
    try {
      if (level === 'unprotected') await disable();
      else if (level === 'audit') await enableAudit();
      else if (level === 'quarantine') await enableQuarantine();

      setOptimisticLevel(level);
      toast.success(RepositoryProfileStrings.FIREWALL_CARD.FIREWALL_SET(LABELS[level], repositoryName));

      // Schedule periodic refreshes so the server-derived level eventually
      // catches up and the useEffect above clears the optimistic override.
      const scheduleRefresh = (delay: number, remaining: number) => {
        if (remaining <= 0) return;
        refreshTimerRef.current = setTimeout(async () => {
          refresh();
          scheduleRefresh(delay, remaining - 1);
        }, delay);
      };
      scheduleRefresh(5000, 4);
    } catch (err) {
      setOptimisticLevel(null);
      toast.error(RepositoryProfileStrings.FIREWALL_CARD.FAILED_UPDATE_FIREWALL(err instanceof Error ? err.message : CommonStrings.MESSAGES.UNKNOWN_ERROR));
    }
  };

  const currentTaskMode: MalwareRemediatorMode =
    !malwareCleanupSummary?.taskEnabled
      ? 'disabled'
      : malwareCleanupSummary?.taskCleanupEnabled
        ? 'delete'
        : 'audit';

  const handleMalwareTaskModeChange = async (mode: MalwareRemediatorMode) => {
    if (mode === currentTaskMode || taskLoading) return;
    setTaskLoading(true);
    try {
      await setMalwareRemediatorEnabledForRepository(repositoryName, mode);
      const label = MALWARE_TASK_LABELS[mode];
      toast.success(RepositoryProfileStrings.FIREWALL_CARD.MALICIOUS_PACKAGES_SET(label, repositoryName));
      refresh();
    } catch (err) {
      console.error('Malicious Packages mode change error:', err);
      toast.error(RepositoryProfileStrings.FIREWALL_CARD.FAILED_UPDATE_MALICIOUS_PACKAGES_TASK(err instanceof Error ? err.message : CommonStrings.MESSAGES.UNKNOWN_ERROR));
    } finally {
      setTaskLoading(false);
    }
  };

  const infoIcon = (content: string) => (
    <Tooltip content={content}>
      <Box style={{ cursor: 'help', display: 'inline-flex', alignItems: 'center' }}>
        <Info size={14} color="var(--gray-8)" />
      </Box>
    </Tooltip>
  );

  const isConnected = iqCapabilities?.connected;
  const hasFirewallLicense = iqCapabilities?.hasFirewall;
  const hasFirewallAccess = isConnected && hasFirewallLicense;

  if (!isSupported) {
    return (
      <Card size="2" style={{ flex: 1 }}>
        <Flex align="center" gap="3" height="100%">
          <ShieldOff size={24} color="var(--gray-7)" />
          <Box>
            <Flex align="center" gap="2">
              <Text size="2" weight="bold" color="gray">{RepositoryProfileStrings.FIREWALL_CARD.FIREWALL}</Text>
              {infoIcon(RepositoryProfileStrings.FIREWALL_CARD.REPOSITORY_FIREWALL_NOT_SUPPORTED_REPOSITORY)}
            </Flex>
            <Text size="2" color="gray">{RepositoryProfileStrings.FIREWALL_CARD.FORMAT_NOT_SUPPORTED}</Text>
          </Box>
        </Flex>
      </Card>
    );
  }

  if (!hasFirewallAccess) {
    return (
      <Card size="2" style={{ flex: 1 }}>
        <Flex direction="column" gap="3">
          <Flex align="center" justify="between">
            <Flex align="center" gap="2">
              <Shield size={20} color="var(--gray-9)" />
              <Text weight="bold">{RepositoryProfileStrings.FIREWALL_CARD.FIREWALL}</Text>
            </Flex>
            <Badge color="gray" variant="soft">{RepositoryProfileStrings.FIREWALL_CARD.NO_FIREWALL_ACCESS}</Badge>
          </Flex>
          <Text size="2" color="gray">{RepositoryProfileStrings.FIREWALL_CARD.REPOSITORY_FIREWALL_PROTECTION_NOT_AVAILABLE}</Text>
          <Button
            variant="soft"
            size="1"
            onClick={() => window.open('https://links.globalart.dev/astrum-repository-firewall', '_blank')}
            style={{ width: 'fit-content' }}
          >
            {CommonStrings.ACTIONS.LEARN_MORE_TITLE} <ExternalLink size={14} />
          </Button>
        </Flex>
      </Card>
    );
  }

  const critical = firewall?.criticalComponentCount ?? 0;
  const severe = firewall?.severeComponentCount ?? 0;
  const moderate = firewall?.moderateComponentCount ?? 0;
  const affected = firewall?.affectedComponentCount ?? 0;
  const quarantined = firewall?.quarantinedComponentCount ?? 0;
  const totalViolations = critical + severe + moderate;
  const scrubbed = malwareCleanupSummary?.scrubbedCount ?? 0;
  const pending = malwareCleanupSummary?.pendingCount ?? 0;

  return (
    <Card size="2" style={{ flex: 1 }}>
      <Flex direction="column" gap="3">
        {/* ---- Header ---- */}
        <Flex align="center" justify="between">
          <Flex align="center" gap="2">
            <ShieldCheck size={20} color={CSS_COLOR[protectionLevel]} />
            <Text weight="bold">{RepositoryProfileStrings.FIREWALL_CARD.FIREWALL}</Text>
            {infoIcon(RepositoryProfileStrings.FIREWALL_CARD.ASTRUM_REPOSITORY_FIREWALL_STOPS_MALICIOUS)}
          </Flex>
          <Badge color={RADIX_COLOR[protectionLevel]} variant="soft">
            {LABELS[protectionLevel]}
          </Badge>
        </Flex>

        {/* ---- Violation counts: Critical / Severe / Moderate ---- */}
        <Grid columns="3" gap="2">
          <Flex direction="column" align="center" p="2" style={{ backgroundColor: 'var(--red-2)', borderRadius: '4px' }}>
            <Text size="1" color="red">{RepositoryProfileStrings.FIREWALL_CARD.CRITICAL}</Text>
            <Text size="4" weight="bold" color="red">{critical.toLocaleString()}</Text>
          </Flex>
          <Flex direction="column" align="center" p="2" style={{ backgroundColor: 'var(--orange-2)', borderRadius: '4px' }}>
            <Text size="1" color="orange">{RepositoryProfileStrings.FIREWALL_CARD.SEVERE}</Text>
            <Text size="4" weight="bold" color="orange">{severe.toLocaleString()}</Text>
          </Flex>
          <Flex direction="column" align="center" p="2" style={{ backgroundColor: 'var(--yellow-2)', borderRadius: '4px' }}>
            <Text size="1" color="yellow">{RepositoryProfileStrings.FIREWALL_CARD.MODERATE}</Text>
            <Text size="4" weight="bold" color="yellow">{moderate.toLocaleString()}</Text>
          </Flex>
        </Grid>

        {/* ---- Components + quarantine summary ---- */}
        <Text size="1" color="gray">
          {RepositoryProfileStrings.FIREWALL_CARD.VIOLATIONS_SUMMARY(totalViolations.toLocaleString(), affected.toLocaleString())}
          {quarantined > 0 && ` · ${RepositoryProfileStrings.FIREWALL_CARD.QUARANTINED_COUNT(quarantined.toLocaleString())}`}
        </Text>

        {firewall?.errorMessage && (
          <Text size="2" color="red">{firewall.errorMessage}</Text>
        )}

        {/* ---- Firewall Dashboard link ---- */}
        {firewall?.reportUrl && (
          <Button
            variant="soft"
            size="1"
            onClick={() => window.open(firewall.reportUrl, '_blank')}
          >
            {RepositoryProfileStrings.FIREWALL_CARD.FIREWALL_DASHBOARD} <ExternalLink size={14} />
          </Button>
        )}

        <Separator size="4" />

        {/* ---- Protection Level (compact row) ---- */}
        {canEditFirewall && (
        <Box data-testid="firewall-protection-level">
          <Flex align="center" gap="1" mb="2">
            <Text size="1" color="gray">{RepositoryProfileStrings.FIREWALL_CARD.PROTECTION_LEVEL}</Text>
            {infoIcon(RepositoryProfileStrings.FIREWALL_CARD.CONTROLS_HOW_FIREWALL_HANDLES_POLICY)}
          </Flex>
          <Flex gap="2">
            {(['unprotected', 'audit', 'quarantine'] as const).map((level) => {
              const selected = protectionLevel === level;
              const color = CSS_COLOR[level];
              return (
                <Tooltip key={level} content={DESCRIPTIONS[level]}>
                  <Flex
                    align="center"
                    gap="1"
                    px="2"
                    py="1"
                    style={{
                      borderRadius: '6px',
                      border: `1.5px solid ${selected ? color : 'var(--gray-5)'}`,
                      backgroundColor: selected ? `${color}15` : 'transparent',
                      cursor: firewallLoading ? 'not-allowed' : 'pointer',
                      opacity: firewallLoading ? 0.6 : 1,
                      transition: 'all 0.15s ease',
                      flex: 1,
                      justifyContent: 'center',
                    }}
                    onClick={() => handleProtectionChange(level)}
                  >
                    <Box
                      style={{
                        width: 12,
                        height: 12,
                        borderRadius: '50%',
                        border: `2px solid ${selected ? color : 'var(--gray-7)'}`,
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'center',
                        flexShrink: 0,
                      }}
                    >
                      {selected && (
                        <Box style={{ width: 6, height: 6, borderRadius: '50%', backgroundColor: color }} />
                      )}
                    </Box>
                    <Text size="1" weight={selected ? 'bold' : 'regular'} style={{ color: selected ? color : undefined }}>
                      {LABELS[level]}
                    </Text>
                  </Flex>
                </Tooltip>
              );
            })}
          </Flex>
        </Box>
        )}

        {/* ---- Malicious Packages (3-option radio) ---- */}
        {canManageMalwareTask && (
        <Box data-testid="firewall-malware-task">
          <Flex align="center" gap="1" mb="2">
            <Text size="1" color="gray">{RepositoryProfileStrings.FIREWALL_CARD.MALICIOUS_PACKAGES}</Text>
            {infoIcon(RepositoryProfileStrings.FIREWALL_CARD.SCHEDULED_TASK_SCANS_MALICIOUS_PACKAGES)}
          </Flex>
          <Flex gap="2" mb="2">
            {(['disabled', 'audit', 'delete'] as const).map((mode) => {
              const selected = currentTaskMode === mode;
              const color = MALWARE_TASK_COLORS[mode];
              return (
                <Tooltip key={mode} content={MALWARE_TASK_DESCRIPTIONS[mode]}>
                  <Flex
                    align="center"
                    gap="1"
                    px="2"
                    py="1"
                    style={{
                      borderRadius: '6px',
                      border: `1.5px solid ${selected ? color : 'var(--gray-5)'}`,
                      backgroundColor: selected ? `${color}15` : 'transparent',
                      cursor: taskLoading ? 'not-allowed' : 'pointer',
                      opacity: taskLoading ? 0.6 : 1,
                      transition: 'all 0.15s ease',
                      flex: 1,
                      justifyContent: 'center',
                    }}
                    onClick={() => handleMalwareTaskModeChange(mode)}
                  >
                    <Box
                      style={{
                        width: 12,
                        height: 12,
                        borderRadius: '50%',
                        border: `2px solid ${selected ? color : 'var(--gray-7)'}`,
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'center',
                        flexShrink: 0,
                      }}
                    >
                      {selected && (
                        <Box style={{ width: 6, height: 6, borderRadius: '50%', backgroundColor: color }} />
                      )}
                    </Box>
                    <Text size="1" weight={selected ? 'bold' : 'regular'} style={{ color: selected ? color : undefined }}>
                      {MALWARE_TASK_LABELS[mode]}
                    </Text>
                  </Flex>
                </Tooltip>
              );
            })}
          </Flex>
          <Flex gap="3" justify="end">
            <Text size="1" color="gray">{RepositoryProfileStrings.FIREWALL_CARD.SCRUBBED(scrubbed)}</Text>
            {pending > 0 && <Text size="1" color="red">{RepositoryProfileStrings.FIREWALL_CARD.PENDING(pending)}</Text>}
          </Flex>
        </Box>
        )}
      </Flex>
    </Card>
  );
}
