

import React from 'react';
import { Box, Flex, Text, Card, Grid, Separator, Badge, Tooltip } from '@radix-ui/themes';
import {
  HardDrive,
  FileBox,
  Database,
  ShieldCheck,
  Activity,
  Copy,
  Check,
  Info,
} from 'lucide-react';
import { ensureTrailingSlash } from '../../../../../../../utils/url';
import type {
  RepositoryProfileData,
  RepositoryMetrics,
  HealthCheckData,
  FirewallData,
  MalwareCleanupSummary,
  BlobStoreInfo,
} from '../types';
import RepositoryProfileStrings from '../../../../../constants/pages/settings/repository/RepositoryProfileStrings';
import CommonStrings from '../../../../../constants/CommonStrings';
import { pageLanguage } from '../../../../../../../interface/plural';

export interface UsageTabProps {
  repository: RepositoryProfileData;
  metrics: RepositoryMetrics | null;
  healthCheck: HealthCheckData | null;
  firewall: FirewallData | null;
  malwareCleanupSummary: MalwareCleanupSummary | null;
  blobStore: BlobStoreInfo | null;
}

function formatBytes(bytes: number | undefined | null): string {
  if (bytes === undefined || bytes === null || bytes < 0) return '\u2014';
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB', 'PB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(2))} ${sizes[i]}`;
}

function formatTimestamp(epoch: number | undefined): string | null {
  if (!epoch) return null;
  try {
    const ms = epoch < 1e12 ? epoch * 1000 : epoch;
    return new Date(ms).toLocaleString(pageLanguage(), {
      month: 'short', day: 'numeric', year: 'numeric',
      hour: 'numeric', minute: '2-digit',
    });
  } catch {
    return null;
  }
}

function deriveProtectionLabel(fw: FirewallData): { label: string; color: 'green' | 'amber' | 'red' } {
  if (fw.quarantineEnabled) return { label: RepositoryProfileStrings.USAGE_TAB.QUARANTINE, color: 'green' };
  const msg = fw.message?.toLowerCase() ?? '';
  if (msg.includes('quarantine')) return { label: RepositoryProfileStrings.USAGE_TAB.QUARANTINE, color: 'green' };
  if (msg.includes('audit') || msg.includes('enabled') || fw.enabled) return { label: RepositoryProfileStrings.USAGE_TAB.AUDIT, color: 'amber' };
  return { label: RepositoryProfileStrings.USAGE_TAB.UNPROTECTED, color: 'red' };
}

function StatValue({ value, fallback = '\u2014' }: { value: number | undefined | null; fallback?: string }) {
  const display = value != null && value >= 0 ? value.toLocaleString(pageLanguage()) : fallback;
  return <Text size="5" weight="bold">{display}</Text>;
}

function StatLabel({ children }: { children: React.ReactNode }) {
  return <Text size="1" color="gray">{children}</Text>;
}

function InfoTip({ content }: { content: string }) {
  return (
    <Tooltip content={content}>
      <Box style={{ cursor: 'help', display: 'inline-flex', alignItems: 'center' }}>
        <Info size={12} color="var(--gray-8)" />
      </Box>
    </Tooltip>
  );
}

function CopyableUrl({ url }: { url: string }) {
  const [copied, setCopied] = React.useState(false);
  const urlWithSlash = ensureTrailingSlash(url);
  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(urlWithSlash);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch { /* clipboard not available */ }
  };
  return (
    <Flex align="center" gap="2" style={{ minWidth: 0 }}>
      <Text size="2" style={{ wordBreak: 'break-all', fontFamily: 'var(--code-font-family)', flex: 1 }}>{urlWithSlash}</Text>
      <Tooltip content={copied ? RepositoryProfileStrings.USAGE_TAB.COPIED : RepositoryProfileStrings.USAGE_TAB.COPY_URL}>
        <Box onClick={handleCopy} style={{ cursor: 'pointer', flexShrink: 0, display: 'inline-flex' }}>
          {copied ? <Check size={14} color="var(--green-9)" /> : <Copy size={14} color="var(--gray-8)" />}
        </Box>
      </Tooltip>
    </Flex>
  );
}

export function UsageTab({
  repository,
  metrics,
  healthCheck,
  firewall,
  malwareCleanupSummary,
  blobStore,
}: UsageTabProps): JSX.Element {
  if (!metrics) {
    return (
      <Flex direction="column" align="center" justify="center" gap="3" py="9">
        <Activity size={48} color="var(--gray-7)" />
        <Text size="3" weight="medium" color="gray">{RepositoryProfileStrings.USAGE_TAB.USAGE_METRICS_UNAVAILABLE}</Text>
        <Text size="2" color="gray">{RepositoryProfileStrings.USAGE_TAB.METRICS_NOT_YET_AVAILABLE_REPOSITORY}</Text>
      </Flex>
    );
  }

  const hasHealthCheck = healthCheck != null;
  const hasFirewall = firewall?.enabled;
  const hasSecurityData = hasHealthCheck || hasFirewall;
  const remoteUrl = repository.attributes?.proxy?.remoteUrl;
  const blobStoreName = repository.attributes?.storage?.blobStoreName;

  const repoSize = metrics.totalSize || 0;
  const assetCount = metrics.assetCount || blobStore?.blobCount || 0;
  const metricsUnavailable = repoSize === 0 && assetCount === 0;

  return (
    <Flex direction="column" gap="5">
      {/* ================================================================
          Section 1: Storage Metrics
          ================================================================ */}
      <Grid columns="3" gap="4">
        <Card size="2">
          <Flex direction="column" gap="2" align="center" py="2">
            <HardDrive size={22} color="var(--accent-9)" />
            {repoSize > 0 ? (
              <StatValue value={undefined} fallback={formatBytes(repoSize)} />
            ) : (
              <Flex align="center" gap="1">
                <Text size="5" weight="bold" color="gray">{'\u2014'}</Text>
                <InfoTip content={RepositoryProfileStrings.USAGE_TAB.SIZE_NOT_YET_CALCULATED_REPOSITORY} />
              </Flex>
            )}
            <StatLabel>{RepositoryProfileStrings.USAGE_TAB.REPOSITORY_SIZE}</StatLabel>
          </Flex>
        </Card>

        <Card size="2">
          <Flex direction="column" gap="2" align="center" py="2">
            <FileBox size={22} color="var(--accent-9)" />
            {assetCount > 0 ? (
              <StatValue value={assetCount} />
            ) : metricsUnavailable ? (
              <Flex align="center" gap="1">
                <Text size="5" weight="bold" color="gray">{'\u2014'}</Text>
                <InfoTip content={RepositoryProfileStrings.USAGE_TAB.ASSET_COUNT_NOT_YET_CALCULATED} />
              </Flex>
            ) : (
              <StatValue value={0} />
            )}
            <StatLabel>{RepositoryProfileStrings.USAGE_TAB.TOTAL_ASSETS}</StatLabel>
          </Flex>
        </Card>

        <Card size="2">
          <Flex direction="column" gap="2" align="center" py="2">
            <Database size={22} color="var(--accent-9)" />
            <Text size="5" weight="bold">{blobStoreName ?? '\u2014'}</Text>
            <StatLabel>
              {blobStore?.availableSpaceInBytes != null
                ? RepositoryProfileStrings.USAGE_TAB.AVAILABLE(formatBytes(blobStore.availableSpaceInBytes))
                : RepositoryProfileStrings.USAGE_TAB.BLOB_STORE}
            </StatLabel>
          </Flex>
        </Card>
      </Grid>

      {/* ================================================================
          Section 2: Security Posture (only when data exists)
          ================================================================ */}
      {hasSecurityData && (
        <Card size="2">
          <Flex direction="column" gap="3">
            <Flex align="center" gap="2">
              <ShieldCheck size={18} color="var(--accent-9)" />
              <Text size="3" weight="bold">{RepositoryProfileStrings.USAGE_TAB.SECURITY_POSTURE}</Text>
            </Flex>
            <Separator size="4" />

            <Grid columns={{ initial: '1', sm: hasHealthCheck && hasFirewall ? '2' : '1' }} gap="5">
              {/* Health Check side */}
              {hasHealthCheck && (
                <Flex direction="column" gap="3">
                  <Flex align="center" gap="2">
                    <Text size="2" weight="medium">{RepositoryProfileStrings.USAGE_TAB.HEALTH_CHECK}</Text>
                    <Badge
                      size="1"
                      variant="soft"
                      color={healthCheck.enabled ? 'green' : 'gray'}
                    >
                      {healthCheck.enabled ? RepositoryProfileStrings.USAGE_TAB.ENABLED : RepositoryProfileStrings.USAGE_TAB.DISABLED}
                    </Badge>
                  </Flex>
                  <Grid columns="2" gap="3">
                    <Box>
                      <StatValue value={healthCheck.securityIssueCount} />
                      <StatLabel>{RepositoryProfileStrings.USAGE_TAB.SECURITY_ISSUES}</StatLabel>
                    </Box>
                    <Box>
                      <StatValue value={healthCheck.licenseIssueCount} />
                      <StatLabel>{RepositoryProfileStrings.USAGE_TAB.LICENSE_ISSUES}</StatLabel>
                    </Box>
                  </Grid>
                  {(() => {
                    const ts = formatTimestamp(healthCheck.lastAnalyzedDate);
                    return ts ? (
                      <Text size="1" color="gray">{RepositoryProfileStrings.USAGE_TAB.LAST_ANALYZED(ts)}</Text>
                    ) : (
                      <Text size="1" color="gray">{RepositoryProfileStrings.USAGE_TAB.LAST_ANALYZED_NOT_AVAILABLE}</Text>
                    );
                  })()}
                </Flex>
              )}

              {/* Firewall side */}
              {hasFirewall && (
                <Flex direction="column" gap="3"
                  style={hasHealthCheck ? { borderLeft: '1px solid var(--gray-4)', paddingLeft: 'var(--space-5)' } : undefined}
                >
                  <Flex align="center" gap="2">
                    <Text size="2" weight="medium">{RepositoryProfileStrings.USAGE_TAB.FIREWALL}</Text>
                    {(() => {
                      const { label, color } = deriveProtectionLabel(firewall);
                      return <Badge size="1" variant="soft" color={color}>{label}</Badge>;
                    })()}
                  </Flex>
                  <Grid columns="2" gap="3">
                    <Box>
                      <StatValue value={firewall.affectedComponentCount} />
                      <StatLabel>{RepositoryProfileStrings.USAGE_TAB.AFFECTED_COMPONENTS}</StatLabel>
                    </Box>
                    <Box>
                      <StatValue value={firewall.quarantinedComponentCount} />
                      <StatLabel>{RepositoryProfileStrings.USAGE_TAB.QUARANTINED}</StatLabel>
                    </Box>
                    <Box>
                      <Flex gap="1" align="baseline">
                        <StatValue value={firewall.criticalComponentCount} />
                        <Text size="1" color="red">{RepositoryProfileStrings.USAGE_TAB.CRITICAL_LABEL}</Text>
                      </Flex>
                    </Box>
                    <Box>
                      <Flex gap="1" align="baseline">
                        <StatValue value={firewall.severeComponentCount} />
                        <Text size="1" color="orange">{RepositoryProfileStrings.USAGE_TAB.SEVERE_LABEL}</Text>
                      </Flex>
                    </Box>
                  </Grid>
                  {malwareCleanupSummary && (
                    <Flex align="center" gap="3">
                      <Text size="1" color="gray">
                        {RepositoryProfileStrings.USAGE_TAB.MALICIOUS_PACKAGES_MODE(malwareCleanupSummary.taskEnabled ? (malwareCleanupSummary.taskCleanupEnabled ? CommonStrings.ACTIONS.DELETE : RepositoryProfileStrings.USAGE_TAB.AUDIT) : RepositoryProfileStrings.USAGE_TAB.OFF)}
                      </Text>
                      {malwareCleanupSummary.scrubbedCount > 0 && (
                        <Text size="1" color="gray">{RepositoryProfileStrings.USAGE_TAB.SCRUBBED(malwareCleanupSummary.scrubbedCount)}</Text>
                      )}
                      {malwareCleanupSummary.pendingCount > 0 && (
                        <Text size="1" color="red">{RepositoryProfileStrings.USAGE_TAB.PENDING(malwareCleanupSummary.pendingCount)}</Text>
                      )}
                    </Flex>
                  )}
                </Flex>
              )}
            </Grid>
          </Flex>
        </Card>
      )}

      {/* ================================================================
          Section 3: Repository Configuration
          ================================================================ */}
      <Card size="2">
        <Flex direction="column" gap="3">
          <Text size="3" weight="bold">{RepositoryProfileStrings.USAGE_TAB.REPOSITORY_CONFIGURATION}</Text>
          <Separator size="4" />
          <Grid columns="2" gap="4" rows="auto">
            <Box>
              <Text size="1" color="gray" as="div" mb="1">{CommonStrings.LABELS.FORMAT}</Text>
              <Text size="2" weight="medium">{repository.format}</Text>
            </Box>
            <Box>
              <Text size="1" color="gray" as="div" mb="1">{CommonStrings.LABELS.TYPE}</Text>
              <Text size="2" weight="medium" style={{ textTransform: 'capitalize' }}>{repository.type}</Text>
            </Box>
            <Box style={{ gridColumn: 'span 2' }}>
              <Text size="1" color="gray" as="div" mb="1">{RepositoryProfileStrings.USAGE_TAB.REPOSITORY_URL}</Text>
              <CopyableUrl url={repository.url} />
            </Box>
            {remoteUrl && (
              <Box style={{ gridColumn: 'span 2' }}>
                <Text size="1" color="gray" as="div" mb="1">{RepositoryProfileStrings.USAGE_TAB.REMOTE_URL_PROXY}</Text>
                <CopyableUrl url={remoteUrl} />
              </Box>
            )}
            <Box>
              <Text size="1" color="gray" as="div" mb="1">{CommonStrings.LABELS.STATUS}</Text>
              <Badge color={repository.online ? 'green' : 'red'} variant="soft">
                {repository.online ? RepositoryProfileStrings.USAGE_TAB.ONLINE : RepositoryProfileStrings.USAGE_TAB.OFFLINE}
              </Badge>
            </Box>
            {blobStoreName && (
              <Box>
                <Text size="1" color="gray" as="div" mb="1">{RepositoryProfileStrings.USAGE_TAB.BLOB_STORE}</Text>
                <Text size="2" weight="medium">{blobStoreName}</Text>
              </Box>
            )}
          </Grid>
        </Flex>
      </Card>
    </Flex>
  );
}

export default UsageTab;
