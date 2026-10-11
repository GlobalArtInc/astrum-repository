

import React, { useCallback, useState } from 'react';
import { Box, Flex, Text, Code, Card, Grid, Badge, Button, Separator, Tooltip, IconButton } from '@radix-ui/themes';
import {
  Shield,
  Info,
  ExternalLink,
  CheckCircle,
  XCircle,
  Zap,
  Copy,
  Check,
} from 'lucide-react';
import { SettingsLink } from './settingsLink';
import type {
  IqCapabilities,
  CapabilityInfo,
} from '../types';
import RepositoryProfileStrings from '../../../../../constants/pages/settings/repository/RepositoryProfileStrings';
import CommonStrings from '../../../../../constants/CommonStrings';

interface InstanceConfigTabProps {
  iqCapabilities: IqCapabilities | null;
  capabilities: CapabilityInfo[];
}

/**
 * InstanceConfigTab - Read-only view of instance-level configurations (IQ, Health Check, Firewall)
 */
export function InstanceConfigTab({
  iqCapabilities,
  capabilities,
}: InstanceConfigTabProps): JSX.Element {
  const [deploymentIdCopied, setDeploymentIdCopied] = useState(false);

  const healthCheckCapability = capabilities.find(c => c.type === 'healthcheck');
  const isHealthCheckConfigured = Boolean(healthCheckCapability);
  const isHealthCheckEnabled = healthCheckCapability?.enabled ?? false;

  const deploymentId = iqCapabilities?.deploymentId?.trim() ?? '';
  const showFirewallManagementLink =
    Boolean(
      iqCapabilities?.hasFirewall &&
        iqCapabilities?.connected &&
        iqCapabilities?.url &&
        deploymentId
    );

  const firewallManagementUrl =
    showFirewallManagementLink && iqCapabilities?.url
      ? `${iqCapabilities.url.replace(/\/+$/, '')}/assets/index.html#/firewall/management/view/repository_manager/${deploymentId}`
      : '';

  const copyDeploymentId = useCallback(async () => {
    if (!deploymentId) {
      return;
    }
    try {
      await navigator.clipboard.writeText(deploymentId);
      setDeploymentIdCopied(true);
      window.setTimeout(() => setDeploymentIdCopied(false), 2000);
    } catch {
      // Clipboard may be unavailable (e.g. non-secure context)
    }
  }, [deploymentId]);

  const openFirewallManagement = useCallback(() => {
    if (firewallManagementUrl) {
      window.open(firewallManagementUrl, '_blank');
    }
  }, [firewallManagementUrl]);

  return (
    <Box>
      <Grid columns="2" gap="6">
        {/* IQ Server Connection */}
        <Card size="2">
          <Flex direction="column" gap="3">
            <Flex align="center" justify="between">
              <Flex align="center" gap="2">
                <Shield size={18} color="var(--accent-9)" />
                <Text weight="bold">{RepositoryProfileStrings.INSTANCE_CONFIG.IQ_SERVER_CONNECTION}</Text>
              </Flex>
            </Flex>
            <Separator size="4" />
            
            <Flex direction="column" gap="2">
              <Flex align="center" justify="between">
                <Text size="2" color="gray">{CommonStrings.LABELS.STATUS}</Text>
                {iqCapabilities?.connected ? (
                  <Badge color="green" variant="soft">
                    <Flex align="center" gap="1">
                      <CheckCircle size={12} /> {RepositoryProfileStrings.INSTANCE_CONFIG.CONNECTED}
                    </Flex>
                  </Badge>
                ) : (
                  <Badge color="red" variant="soft">
                    <Flex align="center" gap="1">
                      <XCircle size={12} /> {RepositoryProfileStrings.INSTANCE_CONFIG.DISCONNECTED}
                    </Flex>
                  </Badge>
                )}
              </Flex>
              
              <Flex align="center" justify="between">
                <Text size="2" color="gray">{RepositoryProfileStrings.INSTANCE_CONFIG.URL}</Text>
                <Text size="2" style={{ fontFamily: 'var(--code-font-family)' }}>
                  {iqCapabilities?.url || RepositoryProfileStrings.INSTANCE_CONFIG.NOT_CONFIGURED}
                </Text>
              </Flex>

              {iqCapabilities?.connected && (
                <Flex align="center" justify="between" gap="2">
                  <Text size="2" color="gray">{RepositoryProfileStrings.INSTANCE_CONFIG.REPOSITORY_MANAGER_ID}</Text>
                  <Flex align="center" gap="2" style={{ minWidth: 0 }}>
                    <Code size="2" variant="ghost" style={{ overflow: 'hidden', textOverflow: 'ellipsis' }}>
                      {deploymentId || '—'}
                    </Code>
                    {deploymentId ? (
                      <Tooltip content={deploymentIdCopied ? RepositoryProfileStrings.INSTANCE_CONFIG.COPIED : RepositoryProfileStrings.INSTANCE_CONFIG.COPY}>
                        <IconButton
                          variant="ghost"
                          size="1"
                          onClick={copyDeploymentId}
                          aria-label={RepositoryProfileStrings.INSTANCE_CONFIG.COPY_REPOSITORY_MANAGER_ID_ARIA}
                        >
                          {deploymentIdCopied ? (
                            <Check size={14} color="var(--green-9)" />
                          ) : (
                            <Copy size={14} />
                          )}
                        </IconButton>
                      </Tooltip>
                    ) : null}
                  </Flex>
                </Flex>
              )}

              {showFirewallManagementLink && (
                <Flex justify="end" mt="1">
                  <Button variant="soft" size="2" onClick={openFirewallManagement}>
                    <ExternalLink size={14} />
                    {RepositoryProfileStrings.INSTANCE_CONFIG.OPEN_FIREWALL_MANAGEMENT}
                  </Button>
                </Flex>
              )}
            </Flex>

            <Box mt="2">
              <Text size="2" weight="medium" mb="2" display="block">{RepositoryProfileStrings.INSTANCE_CONFIG.LICENSE_FEATURES}</Text>
              <Grid columns="2" gap="2">
                <Card variant="surface" size="1">
                  <Flex direction="column" gap="1">
                    <Text size="1" color="gray">{RepositoryProfileStrings.INSTANCE_CONFIG.LIFECYCLE}</Text>
                    <Flex align="center" gap="2">
                      {iqCapabilities?.hasLifecycle ? (
                        <CheckCircle size={14} color="var(--green-9)" />
                      ) : (
                        <XCircle size={14} color="var(--gray-7)" />
                      )}
                      <Text size="2">{iqCapabilities?.hasLifecycle ? RepositoryProfileStrings.INSTANCE_CONFIG.ACTIVE : RepositoryProfileStrings.INSTANCE_CONFIG.NOT_AVAILABLE}</Text>
                    </Flex>
                    {!iqCapabilities?.hasLifecycle && (
                      <a href="https://links.globalart.dev/products/astrum/browse/lc-learn" target="_blank" rel="noopener" className="iq-server-page__link" style={{ fontSize: '10px' }}>
                        {CommonStrings.ACTIONS.LEARN_MORE_TITLE} <ExternalLink size={8} />
                      </a>
                    )}
                  </Flex>
                </Card>
                <Card variant="surface" size="1">
                  <Flex direction="column" gap="1">
                    <Text size="1" color="gray">{RepositoryProfileStrings.INSTANCE_CONFIG.FIREWALL}</Text>
                    <Flex align="center" gap="2">
                      {iqCapabilities?.hasFirewall ? (
                        <CheckCircle size={14} color="var(--green-9)" />
                      ) : (
                        <XCircle size={14} color="var(--gray-7)" />
                      )}
                      <Text size="2">{iqCapabilities?.hasFirewall ? RepositoryProfileStrings.INSTANCE_CONFIG.ACTIVE : RepositoryProfileStrings.INSTANCE_CONFIG.NOT_AVAILABLE}</Text>
                    </Flex>
                    {!iqCapabilities?.hasFirewall && (
                      <a href="https://links.globalart.dev/astrum-repository-firewall" target="_blank" rel="noopener" className="iq-server-page__link" style={{ fontSize: '10px' }}>
                        {CommonStrings.ACTIONS.LEARN_MORE_TITLE} <ExternalLink size={8} />
                      </a>
                    )}
                  </Flex>
                </Card>
              </Grid>
            </Box>
          </Flex>
        </Card>

        {/* Global Capabilities */}
        <Card size="2">
          <Flex direction="column" gap="3">
            <Flex align="center" justify="between">
              <Flex align="center" gap="2">
                <Zap size={18} color="var(--yellow-9)" />
                <Text weight="bold">{RepositoryProfileStrings.INSTANCE_CONFIG.GLOBAL_CAPABILITIES}</Text>
              </Flex>
              <SettingsLink previewPath="admin/system/capabilities" label={RepositoryProfileStrings.INSTANCE_CONFIG.MANAGE} />
            </Flex>
            <Separator size="4" />

            <Flex direction="column" gap="4">
              {/* Health Check Instance Config */}
              <Box>
                <Flex align="center" justify="between" mb="1">
                  <Text size="2" weight="medium">{RepositoryProfileStrings.INSTANCE_CONFIG.REPOSITORY_HEALTH_CHECK}</Text>
                  {isHealthCheckConfigured ? (
                    <Badge color={isHealthCheckEnabled ? 'green' : 'yellow'} variant="soft">
                      {isHealthCheckEnabled ? RepositoryProfileStrings.INSTANCE_CONFIG.ACTIVE : RepositoryProfileStrings.INSTANCE_CONFIG.DISABLED}
                    </Badge>
                  ) : (
                    <Badge color="gray" variant="soft">{RepositoryProfileStrings.INSTANCE_CONFIG.NOT_CONFIGURED_2}</Badge>
                  )}
                </Flex>
                <Text size="1" color="gray">
                  {RepositoryProfileStrings.INSTANCE_CONFIG.INSTANCE_LEVEL_CAPABILITY_ANALYZE_PROXY}
                </Text>
              </Box>

              {/* Firewall Instance Config */}
              <Box>
                <Flex align="center" justify="between" mb="1">
                  <Text size="2" weight="medium">{RepositoryProfileStrings.INSTANCE_CONFIG.REPOSITORY_FIREWALL}</Text>
                  {iqCapabilities?.hasFirewall ? (
                    <Badge color="green" variant="soft">{RepositoryProfileStrings.INSTANCE_CONFIG.AVAILABLE}</Badge>
                  ) : (
                    <Badge color="gray" variant="soft">{RepositoryProfileStrings.INSTANCE_CONFIG.REQUIRES_LICENSE}</Badge>
                  )}
                </Flex>
                <Text size="1" color="gray">
                  {RepositoryProfileStrings.INSTANCE_CONFIG.KEEP_BAD_CODE_OUT_REPOSITORY}
                </Text>
              </Box>
            </Flex>

            <Box mt="auto" pt="2">
              <Card variant="ghost" size="1" style={{ backgroundColor: 'var(--gray-2)' }}>
                <Flex gap="2">
                  <Info size={14} color="var(--gray-9)" />
                  <Text size="1" color="gray">
                    {RepositoryProfileStrings.INSTANCE_CONFIG.THESE_SETTINGS_AFFECT_ALL_SUPPORTED}
                  </Text>
                </Flex>
              </Card>
            </Box>
          </Flex>
        </Card>
      </Grid>
    </Box>
  );
}

export default InstanceConfigTab;
