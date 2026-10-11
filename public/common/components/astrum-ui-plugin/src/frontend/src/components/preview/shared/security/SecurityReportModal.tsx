

import React from 'react';
import { Dialog, Flex, Text, Box, Button, Grid, Separator, Heading } from '@radix-ui/themes';
import { Shield, ExternalLink, ShieldCheck, X } from 'lucide-react';
import { SecurityStatusData } from './security.types';

import './SecurityReportModal.scss';
import SecurityReportStrings from '../../constants/SecurityReportStrings';
import CommonStrings from '../../constants/CommonStrings';
import plural from '../../../../interface/plural';

export interface SecurityReportModalProps {
  /** Repository name */
  repositoryName: string;
  /** Firewall/Security data */
  data?: SecurityStatusData;
  /** Whether the modal is open */
  isOpen: boolean;
  /** Callback when modal is closed */
  onClose: () => void;
  /** Optional title override */
  title?: string;
  /** Report type for visual differentiation and policy summary (firewall vs health-check) */
  type?: 'firewall' | 'health-check';
}

/**
 * SecurityReportModal - Displays detailed security analysis for a repository.
 * Aligned with AgentUX's design for the Protected Repo Upsell initiative.
 *
 * @deprecated Use full-page Health Report and Firewall Report routes instead.
 * HealthCheckCell and FirewallCell now navigate to preview.browse.health-report
 * and preview.browse.firewall-report rather than opening this modal.
 */
export function SecurityReportModal({
  repositoryName,
  data,
  isOpen,
  onClose,
  title = SecurityReportStrings.REPORT_MODAL.SECURITY_REPORT,
  type = 'health-check',
}: SecurityReportModalProps) {
  if (!data) return null;

  const hasIssues = data.criticalComponentCount > 0 || data.severeComponentCount > 0 || data.moderateComponentCount > 0;
  const isFirewall = type === 'firewall';
  const hasQuarantine = (data.quarantinedComponentCount ?? 0) > 0;

  return (
    <Dialog.Root open={isOpen} onOpenChange={(open) => !open && onClose()}>
      <Dialog.Content
        maxWidth="800px"
        className={`security-report-modal ${isFirewall ? 'security-report-modal--firewall' : ''}`}
      >
        <Flex justify="between" align="center" mb="4">
          <Dialog.Title m="0">
            <Flex align="center" gap="3">
              <Shield size={24} color="var(--accent-9)" />
              <Box>
                <Heading size="5">{title}</Heading>
                <Text size="2" color="gray">{repositoryName}</Text>
              </Box>
            </Flex>
          </Dialog.Title>
          <Dialog.Close>
            <Button variant="ghost" color="gray" onClick={onClose}>
              <X size={20} />
            </Button>
          </Dialog.Close>
        </Flex>

        <Box className="security-report-modal__body">
          {/* Policy Summary - Firewall only */}
          {isFirewall && (
            <Box mb="6" p="4" className="security-report-modal__policy-summary">
              <Flex direction="column" gap="2">
                <Text size="2" weight="bold">{SecurityReportStrings.REPORT_MODAL.POLICY_SUMMARY}</Text>
                <Text size="2" color="gray">
                  {SecurityReportStrings.REPORT_MODAL.QUARANTINED_COMPONENTS_BLOCKED_REPOSITORY_FIREWALL}
                </Text>
              </Flex>
            </Box>
          )}

          {/* Quarantine Badge - prominent when components in quarantine */}
          {hasQuarantine && (
            <Box mb="6">
              <Flex
                align="center"
                gap="3"
                p="4"
                className={`security-report-modal__quarantine-badge ${isFirewall ? 'security-report-modal__quarantine-badge--firewall' : ''}`}
              >
                <Shield size={24} color="var(--orange-9)" />
                <Box>
                  <Text size="3" weight="bold">
                    {plural(SecurityReportStrings.QUARANTINED_COMPONENTS, data.quarantinedComponentCount ?? 0)}
                  </Text>
                  <Text size="2" color="gray">
                    {isFirewall
                      ? SecurityReportStrings.REPORT_MODAL.BLOCKED_FIREWALL_POLICY_CONFIGURE_AUTO
                      : SecurityReportStrings.REPORT_MODAL.THESE_COMPONENTS_REQUIRE_REVIEW}
                  </Text>
                </Box>
              </Flex>
            </Box>
          )}

          {/* Summary Stats */}
          <Grid columns="3" gap="4" mb="6">
            <SummaryCard
              label={SecurityReportStrings.REPORT_MODAL.CRITICAL}
              value={data.criticalComponentCount}
              color="red"
              description={isFirewall ? SecurityReportStrings.REPORT_MODAL.HIGHEST_RISK_VIOLATIONS : SecurityReportStrings.REPORT_MODAL.HIGHEST_RISK_VULNERABILITIES}
            />
            <SummaryCard
              label={SecurityReportStrings.REPORT_MODAL.SEVERE}
              value={data.severeComponentCount}
              color="orange"
              description={isFirewall ? SecurityReportStrings.REPORT_MODAL.HIGH_RISK_VIOLATIONS : SecurityReportStrings.REPORT_MODAL.HIGH_RISK_VULNERABILITIES}
            />
            <SummaryCard
              label={SecurityReportStrings.REPORT_MODAL.MODERATE}
              value={data.moderateComponentCount}
              color="yellow"
              description={isFirewall ? SecurityReportStrings.REPORT_MODAL.MEDIUM_RISK_VIOLATIONS : SecurityReportStrings.REPORT_MODAL.MEDIUM_RISK_VULNERABILITIES}
            />
          </Grid>

          <Separator size="4" mb="6" />

          {/* Detailed Report Section */}
          <Box mb="6">
            <Heading size="4" mb="3">{SecurityReportStrings.REPORT_MODAL.ANALYSIS_DETAILS}</Heading>
            {data.reportUrl ? (
              <Box className="security-report-modal__iframe-wrapper">
                <iframe
                  src={data.reportUrl}
                  title={SecurityReportStrings.REPORT_MODAL.SECURITY_REPORT_FOR(repositoryName)}
                  width="100%"
                  height="450px"
                  style={{ border: 'none', borderRadius: 'var(--radius-3)' }}
                  sandbox="allow-same-origin allow-scripts"
                />
              </Box>
            ) : (
              <Flex direction="column" align="center" justify="center" p="8" className="security-report-modal__empty">
                <ShieldCheck size={48} color="var(--gray-7)" mb="3" />
                <Text size="3" weight="medium" color="gray">
                  {hasIssues ? SecurityReportStrings.REPORT_MODAL.FULL_REPORT_DATA_BEING_PROCESSED : SecurityReportStrings.REPORT_MODAL.NO_SECURITY_ISSUES_DETECTED}
                </Text>
                <Text size="2" color="gray">
                  {SecurityReportStrings.REPORT_MODAL.RUN_NEW_ANALYSIS_UPDATE_THESE}
                </Text>
              </Flex>
            )}
          </Box>
        </Box>

        <Flex gap="3" justify="end" mt="4">
          {data.reportUrl && (
            <Button variant="soft" asChild>
              <a href={data.reportUrl} target="_blank" rel="noopener noreferrer">
                <ExternalLink size={16} />
                {SecurityReportStrings.REPORT_MODAL.VIEW_FULL_REPORT}
              </a>
            </Button>
          )}
          <Dialog.Close>
            <Button variant="solid" onClick={onClose}>{CommonStrings.ACTIONS.CLOSE}</Button>
          </Dialog.Close>
        </Flex>
      </Dialog.Content>
    </Dialog.Root>
  );
}

function SummaryCard({ label, value, color, description }: { label: string; value: number; color: any; description: string }) {
  return (
    <Card p="4" className={`security-report-modal__card security-report-modal__card--${color}`}>
      <Flex direction="column" align="center" gap="1">
        <Text size="2" weight="bold" color={color} style={{ textTransform: 'uppercase', letterSpacing: '0.05em' }}>
          {label}
        </Text>
        <Heading size="8" style={{ color: `var(--${color}-9)` }}>
          {value.toLocaleString()}
        </Heading>
        <Text size="1" color="gray" align="center">
          {description}
        </Text>
      </Flex>
    </Card>
  );
}

// Internal Card helper since @radix-ui/themes Card might not be enough
function Card({ children, p, className, style }: any) {
  return (
    <Box 
      p={p} 
      className={className} 
      style={{ 
        background: 'var(--color-surface)', 
        border: '1px solid var(--gray-6)', 
        borderRadius: 'var(--radius-3)',
        ...style 
      }}
    >
      {children}
    </Box>
  );
}
