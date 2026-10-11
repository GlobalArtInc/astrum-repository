

import React from 'react';
import { Popover, Flex, Text, Box, Button, Separator, Heading } from '@radix-ui/themes';
import { Shield, ShieldCheck, ExternalLink, ChevronRight } from 'lucide-react';
import { SecurityStatusData } from './security.types';

import './SecurityPopover.scss';
import SecurityReportStrings from '../../constants/SecurityReportStrings';
import plural from '../../../../interface/plural';

export interface SecurityPopoverProps {
  /** The clickable element that triggers the popover */
  children: React.ReactNode;
  /** Repository name for context */
  repositoryName: string;
  /** Security data summary */
  data: SecurityStatusData;
  /** Callback to open the full report modal */
  onViewFullReport: () => void;
  /** Type of security data (Firewall or Health Check) */
  type: 'firewall' | 'health-check';
}

/**
 * SecurityPopover - Shared component for the "Popover + Modal" pattern.
 * Displays a quick summary of vulnerabilities and an action to see the full report.
 */
export function SecurityPopover({
  children,
  repositoryName,
  data,
  onViewFullReport,
  type
}: SecurityPopoverProps) {
  const { criticalComponentCount, severeComponentCount, moderateComponentCount } = data;
  const isHealthy = criticalComponentCount === 0 && severeComponentCount === 0 && moderateComponentCount === 0;

  const title = type === 'firewall' ? SecurityReportStrings.POPOVER.FIREWALL_PROTECTION : SecurityReportStrings.POPOVER.HEALTH_CHECK_SUMMARY;

  return (
    <Popover.Root>
      <Popover.Trigger>
        {children}
      </Popover.Trigger>
      <Popover.Content
        size="2"
        width="320px"
        className={`security-popover ${type === 'firewall' ? 'security-popover--firewall' : ''}`}
      >
        <Flex direction="column" gap="3">
          <Box>
            <Flex align="center" gap="2" mb="1">
              <Shield size={16} color="var(--accent-9)" />
              <Heading size="3">{title}</Heading>
            </Flex>
            <Text size="1" color="gray">{repositoryName}</Text>
          </Box>

          <Separator size="4" />

          {isHealthy ? (
            <Flex align="center" gap="2" p="2" className="security-popover__status--safe">
              <ShieldCheck size={20} color="var(--green-9)" />
              <Box>
                <Text size="2" weight="bold" color="green">{SecurityReportStrings.POPOVER.SECURE}</Text>
                <Text size="1" color="gray" as="p">{SecurityReportStrings.POPOVER.NO_MALICIOUS_COMPONENTS_DETECTED}</Text>
              </Box>
            </Flex>
          ) : (
            <Flex direction="column" gap="2">
              <MetricRow label={SecurityReportStrings.POPOVER.CRITICAL} count={criticalComponentCount} color="red" />
              <MetricRow label={SecurityReportStrings.POPOVER.SEVERE} count={severeComponentCount} color="orange" />
              <MetricRow label={SecurityReportStrings.POPOVER.MODERATE} count={moderateComponentCount} color="yellow" />
            </Flex>
          )}

          {data.quarantinedComponentCount > 0 && (
            <Flex
              align="center"
              gap="2"
              className={`security-popover__quarantine-alert ${type === 'firewall' ? 'security-popover__quarantine-alert--badge' : ''}`}
              p="2"
            >
              <Shield size={14} />
              <Text size="1" weight="medium">
                {plural(SecurityReportStrings.QUARANTINED_COMPONENTS, data.quarantinedComponentCount ?? 0)}
              </Text>
            </Flex>
          )}
          {type === 'firewall' && (
            <Box className="security-popover__policy-summary" p="2">
              <Text size="1" color="gray">
                {SecurityReportStrings.POPOVER.POLICY_DETAILS_REPOSITORY_PROFILE_SETTINGS}
              </Text>
            </Box>
          )}

          <Button 
            variant="soft" 
            size="2" 
            onClick={(e) => {
              e.stopPropagation();
              onViewFullReport();
            }}
            className="security-popover__action"
          >
            <ExternalLink size={14} />
            {SecurityReportStrings.POPOVER.VIEW_DETAILED_REPORT}
            <ChevronRight size={14} style={{ marginLeft: 'auto' }} />
          </Button>
        </Flex>
      </Popover.Content>
    </Popover.Root>
  );
}

function MetricRow({ label, count, color }: { label: string; count: number; color: any }) {
  if (count === 0) return null;
  return (
    <Flex justify="between" align="center" className={`security-popover__metric security-popover__metric--${color}`}>
      <Text size="2" weight="medium">{label}</Text>
      <Flex align="center" gap="2">
        <Text size="2" weight="bold" color={color}>{count}</Text>
        <Box className={`security-popover__dot security-popover__dot--${color}`} />
      </Flex>
    </Flex>
  );
}
