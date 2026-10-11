

import React from 'react';
import { Box, Card, Flex, Text } from '@radix-ui/themes';
import { ShieldCheck, Shield } from 'lucide-react';
import type { HealthCheckData, FirewallData } from './types';
import RepositoryProfileStrings from '../../../../constants/pages/settings/repository/RepositoryProfileStrings';
import plural from '../../../../../../interface/plural';

export interface HealthCheckSummaryWidgetProps {
  repositoryName: string;
  healthCheck: HealthCheckData | null;
  firewall: FirewallData | null;
  onSelectTab: (tab: 'health-check' | 'firewall-report') => void;
  /** Hide for group repositories */
  isGroup?: boolean;
}

/**
 * Summary widget for Health Check + Firewall metrics in repo profile header.
 * Compact, scannable; click to switch to that tab.
 */
export function HealthCheckSummaryWidget({
  repositoryName,
  healthCheck,
  firewall,
  onSelectTab,
  isGroup = false,
}: HealthCheckSummaryWidgetProps): JSX.Element | null {
  if (isGroup) return null;

  const hcSecurity = healthCheck?.securityIssueCount ?? 0;
  const hcLicense = healthCheck?.licenseIssueCount ?? 0;

  const fwQuarantine = firewall?.quarantinedComponentCount ?? 0;
  const fwEnabled = firewall?.enabled ?? false;

  return (
    <Flex gap="3" wrap="wrap">
      <Card
        onClick={() => onSelectTab('health-check')}
        style={{
          cursor: 'pointer',
          flex: 1,
          minWidth: '180px',
          padding: 'var(--space-3)',
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
          transition: 'background-color 0.15s'
        }}
      >
        <Flex align="center" gap="2">
          <ShieldCheck size={18} color="var(--accent-9)" />
          <Box>
            <Text size="1" color="gray">{RepositoryProfileStrings.HEALTH_SUMMARY.HEALTH_CHECK}</Text>
            <Flex gap="2" align="center">
              {hcSecurity > 0 && <Text size="2" weight="bold" color="red">{plural(RepositoryProfileStrings.HEALTH_SUMMARY.VULNERABILITIES, hcSecurity)}</Text>}
              {hcLicense > 0 && <Text size="2" weight="bold" color="amber">{RepositoryProfileStrings.HEALTH_SUMMARY.LICENSE(hcLicense)}</Text>}
              {hcSecurity === 0 && hcLicense === 0 && (
                <Text size="2" color="green">{RepositoryProfileStrings.HEALTH_SUMMARY.NO_ISSUES}</Text>
              )}
            </Flex>
          </Box>
        </Flex>
        <Text size="1" color="blue" style={{ cursor: 'pointer' }}>
          {RepositoryProfileStrings.HEALTH_SUMMARY.VIEW_REPORT}
        </Text>
      </Card>
      <Card
        onClick={() => onSelectTab('firewall-report')}
        style={{
          cursor: 'pointer',
          flex: 1,
          minWidth: '180px',
          padding: 'var(--space-3)',
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
          transition: 'background-color 0.15s'
        }}
      >
        <Flex align="center" gap="2">
          <Shield size={18} color="var(--orange-9)" />
          <Box>
            <Text size="1" color="gray">{RepositoryProfileStrings.HEALTH_SUMMARY.FIREWALL}</Text>
            <Flex gap="2" align="center">
              {fwEnabled ? (
                <>
                  <Text size="2" color="green">{RepositoryProfileStrings.HEALTH_SUMMARY.PROTECTED}</Text>
                  {fwQuarantine > 0 && <Text size="2" color="orange">{RepositoryProfileStrings.HEALTH_SUMMARY.QUARANTINE(fwQuarantine)}</Text>}
                </>
              ) : (
                <Text size="2" color="gray">{RepositoryProfileStrings.HEALTH_SUMMARY.UNPROTECTED}</Text>
              )}
            </Flex>
          </Box>
        </Flex>
        <Text size="1" color="blue" style={{ cursor: 'pointer' }}>
          {fwEnabled ? RepositoryProfileStrings.HEALTH_SUMMARY.VIEW_REPORT : RepositoryProfileStrings.HEALTH_SUMMARY.ENABLE}
        </Text>
      </Card>
    </Flex>
  );
}

