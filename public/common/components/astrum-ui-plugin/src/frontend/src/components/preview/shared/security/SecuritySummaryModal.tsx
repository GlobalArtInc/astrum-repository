

import React from 'react';
import { Dialog, Flex, Text, Box, Button, Separator, Badge, Spinner } from '@radix-ui/themes';
import { ShieldCheck, Shield, ExternalLink, X, AlertCircle } from 'lucide-react';
import { SecurityStatusData } from './security.types';
import { restClient } from '../../../../interface/api';

import './SecuritySummaryModal.scss';
import SecurityReportStrings from '../../constants/SecurityReportStrings';
import CommonStrings from '../../constants/CommonStrings';

interface MalwareCleanupSummary {
  repositoryName: string;
  scrubbedCount: number;
  pendingCount: number;
  lastRun?: string;
  taskStatus: string;
  taskEnabled: boolean;
}

export interface SecuritySummaryModalProps {
  repositoryName: string;
  data: SecurityStatusData;
  isOpen: boolean;
  onClose: () => void;
  type: 'health-check' | 'firewall';
  onViewFullReport?: () => void;
  reportUrl?: string;
  firewallStatus?: 'protected' | 'audit' | 'unprotected' | 'unavailable';
  hasFirewallLicense?: boolean;
  /** @deprecated Enable buttons removed from modal -- prop accepted for backwards compat but ignored */
  onEnableSuccess?: () => void;
  onBrowseRepo?: () => void;
  onConfigureFirewall?: () => void;
  unsupportedMessage?: string;
  unsupportedBadgeColor?: 'green' | 'amber' | 'red';
  unsupportedBadgeText?: string;
  unsupportedExtraMessage?: string;
  unsupportedExtraButtons?: 'malware-defense' | 'learn-more' | 'none';
  onMalwareDefense?: () => void;
  /** When true (firewall only), per-repository report detail is still loading — hide empty state and show loading UI */
  isFirewallReportDetailLoading?: boolean;
}

export function SecuritySummaryModal({
  repositoryName,
  data,
  isOpen,
  onClose,
  type,
  onViewFullReport,
  reportUrl,
  firewallStatus = 'unavailable',
  hasFirewallLicense = false,
  onBrowseRepo,
  onConfigureFirewall,
  unsupportedMessage,
  unsupportedBadgeColor,
  unsupportedBadgeText,
  unsupportedExtraMessage,
  unsupportedExtraButtons = 'none',
  onMalwareDefense,
  isFirewallReportDetailLoading = false,
}: SecuritySummaryModalProps): JSX.Element {
  const [malwareSummary, setMalwareSummary] = React.useState<MalwareCleanupSummary | null>(null);
  const isHealthCheck = type === 'health-check';
  const Icon = isHealthCheck ? ShieldCheck : Shield;
  const title = isHealthCheck ? SecurityReportStrings.SUMMARY_MODAL.HEALTH_CHECK(repositoryName) : SecurityReportStrings.SUMMARY_MODAL.FIREWALL_REPORT(repositoryName);

  React.useEffect(() => {
    if (!isHealthCheck && isOpen) {
      const fetchMalware = async () => {
        try {
          const summary = await restClient.get<MalwareCleanupSummary>(
            `/service/rest/internal/ui/iq/malware-cleanup/summary/${encodeURIComponent(repositoryName)}`
          );
          setMalwareSummary(summary);
        } catch {
          // endpoint may not be available
        }
      };
      fetchMalware();
    }
  }, [isHealthCheck, repositoryName, isOpen]);

  const {
    criticalComponentCount,
    severeComponentCount,
    moderateComponentCount,
    quarantinedComponentCount,
    affectedComponentCount,
    reportDate,
    reportAge,
    componentsIdentified,
    componentsTotal,
    securityCriticalCount,
    securitySevereCount,
    securityModerateCount,
    licenseIssueCount,
    licenseCopyleftCount,
    licenseNonStandardCount,
    licenseNotProvidedCount,
    licenseWeakCopyleftCount,
    licenseLiberalCount,
    threatLevelCounts,
  } = data;

  const isFirewall = type === 'firewall';

  const critical = securityCriticalCount ?? criticalComponentCount;
  const severe = securitySevereCount ?? severeComponentCount;
  const moderate = securityModerateCount ?? moderateComponentCount;
  const hasSecurityIssues = critical > 0 || severe > 0 || moderate > 0;

  const componentsId = componentsIdentified ?? affectedComponentCount ?? 0;
  const componentsTot = componentsTotal ?? componentsId;
  const pct = componentsTot > 0 ? Math.round((componentsId / componentsTot) * 100) : 100;

  const hasLicenseBreakdown =
    (licenseCopyleftCount ?? 0) > 0 ||
    (licenseNonStandardCount ?? 0) > 0 ||
    (licenseNotProvidedCount ?? 0) > 0 ||
    (licenseWeakCopyleftCount ?? 0) > 0 ||
    (licenseLiberalCount ?? 0) > 0;
  const hasLicenseData = hasLicenseBreakdown || (licenseIssueCount ?? 0) > 0;

  // Only show per-threat-level bars when real data exists.
  // The Health Check API only returns aggregate Critical/Severe/Moderate counts,
  // not per-level (1-10) breakdown. Fabricating per-level data is misleading.
  const threatBars = React.useMemo(() => {
    if (!isHealthCheck) return [];
    if (threatLevelCounts && threatLevelCounts.length >= 10) {
      return threatLevelCounts.slice(0, 10).map((c) => c ?? 0);
    }
    return [];
  }, [isHealthCheck, threatLevelCounts]);

  const maxThreat = threatBars.length > 0 ? Math.max(1, ...threatBars) : 1;

  React.useEffect(() => {
    if (isOpen) {
      document.body.classList.add('as-security-summary-modal-open');
      return () => document.body.classList.remove('as-security-summary-modal-open');
    }
  }, [isOpen]);

  const isFirewallNoLicense = type === 'firewall' && !hasFirewallLicense;
  const isFirewallUnavailable = type === 'firewall' && (firewallStatus === 'unavailable' || isFirewallNoLicense);
  const isFirewallUnprotected = type === 'firewall' && firewallStatus === 'unprotected' && hasFirewallLicense;

  const isFirewallReportBodyLoading =
    isFirewall &&
    isFirewallReportDetailLoading &&
    !isFirewallUnavailable &&
    !isFirewallUnprotected &&
    !unsupportedMessage;

  return (
    <Dialog.Root open={isOpen} onOpenChange={(open) => { if (!open) { onClose(); } }}>
      <Dialog.Content className="security-summary-modal" aria-describedby={undefined} onClick={(e) => e.stopPropagation()} onPointerDown={(e) => e.stopPropagation()}>
        <Flex justify="between" align="center" gap="3" className="security-summary-modal__header">
          <Flex align="center" gap="3">
            <Icon size={24} color="var(--accent-9)" aria-hidden />
            <Dialog.Title asChild>
              <Text as="h2" size="5" weight="bold">{title}</Text>
            </Dialog.Title>
          </Flex>
          <Dialog.Close asChild>
            <Button variant="ghost" color="gray" size="1" aria-label={CommonStrings.ACTIONS.CLOSE}>
              <X size={20} />
            </Button>
          </Dialog.Close>
        </Flex>

        <Box className="security-summary-modal__body">
          {/* No Firewall License -- marketing CTA */}
          {isFirewallNoLicense && !unsupportedMessage && (
            <Box p="4">
              <Badge size="2" color="blue" variant="soft" style={{ marginBottom: 'var(--space-3)', display: 'inline-flex' }}>
                {SecurityReportStrings.SUMMARY_MODAL.REPOSITORY_FIREWALL}
              </Badge>
              <Text size="3" weight="medium" as="p" mb="2">
                {SecurityReportStrings.SUMMARY_MODAL.KEEP_BAD_CODE_OUT_REPOSITORY}
              </Text>
              <Text size="2" color="gray" as="p">
                {SecurityReportStrings.SUMMARY_MODAL.REPOSITORY_FIREWALL_BLOCKS_MALICIOUS_VULNERABLE}
              </Text>
              <Flex gap="2" mt="4">
                <Button
                  variant="solid"
                  size="2"
                  onClick={() => window.open('https://links.globalart.dev/astrum-repository-firewall', '_blank', 'noopener,noreferrer')}
                >
                  {CommonStrings.ACTIONS.LEARN_MORE_TITLE} <ExternalLink size={12} />
                </Button>
              </Flex>
            </Box>
          )}

          {/* Has Firewall License but not enabled on this repo */}
          {isFirewallUnprotected && !unsupportedMessage && (
            <Box p="4">
              <Badge size="2" color="red" variant="soft" style={{ marginBottom: 'var(--space-3)', display: 'inline-flex' }}>
                {SecurityReportStrings.SUMMARY_MODAL.UNPROTECTED}
              </Badge>
              <Text size="3" weight="medium" as="p" mb="2">
                {SecurityReportStrings.SUMMARY_MODAL.REPOSITORY_DOES_NOT_HAVE_FIREWALL}
              </Text>
              <Text size="2" color="gray" as="p">
                {SecurityReportStrings.SUMMARY_MODAL.ENABLE_FIREWALL_AUTOMATICALLY_EVALUATE_COMPONENTS}
              </Text>
              <Flex gap="2" mt="4">
                {onConfigureFirewall && (
                  <Button variant="solid" size="2" onClick={onConfigureFirewall}>
                    {SecurityReportStrings.SUMMARY_MODAL.ENABLE_FIREWALL_PROTECTION}
                  </Button>
                )}
                <Button
                  variant="solid"
                  size="2"
                  onClick={() => window.open('https://links.globalart.dev/astrum-repository-firewall', '_blank', 'noopener,noreferrer')}
                >
                  {CommonStrings.ACTIONS.LEARN_MORE_TITLE} <ExternalLink size={12} />
                </Button>
              </Flex>
            </Box>
          )}

          {/* Firewall unavailable (IQ server error) */}
          {isFirewallUnavailable && !isFirewallNoLicense && !unsupportedMessage && (
            <Box p="4">
              <Badge size="2" color="amber" variant="soft" style={{ marginBottom: 'var(--space-3)', display: 'inline-flex' }}>
                {SecurityReportStrings.SUMMARY_MODAL.UNAVAILABLE}
              </Badge>
              <Text size="2" color="gray" as="p">
                {SecurityReportStrings.SUMMARY_MODAL.FIREWALL_STATUS_TEMPORARILY_UNAVAILABLE_IQ}
              </Text>
            </Box>
          )}

          {/* Unsupported format */}
          {unsupportedMessage && (
            <Box p="4" className="security-summary-modal__unsupported">
              {unsupportedBadgeText ? (
                <Badge size="2" color={unsupportedBadgeColor || 'gray'} variant="soft" style={{ marginBottom: 'var(--space-3)', display: 'inline-flex' }}>
                  {unsupportedBadgeText}
                </Badge>
              ) : (
                <Flex align="center" gap="2" mb="3">
                  <AlertCircle size={20} color="var(--gray-8)" />
                  <Text size="3" weight="bold" color="gray">{SecurityReportStrings.SUMMARY_MODAL.NOT_SUPPORTED}</Text>
                </Flex>
              )}
              <Text size="2" color="gray" as="p">{unsupportedMessage}</Text>
              {unsupportedExtraMessage && (
                <Text size="2" color="gray" as="p" mt="2">{unsupportedExtraMessage}</Text>
              )}
              {unsupportedExtraButtons === 'malware-defense' && onMalwareDefense && (
                <Flex gap="2" mt="3">
                  <Button variant="soft" size="2" onClick={onMalwareDefense}>{SecurityReportStrings.SUMMARY_MODAL.MALWARE_DEFENSE}</Button>
                </Flex>
              )}
              {unsupportedExtraButtons === 'learn-more' && (
                <Flex gap="2" mt="3">
                  <Button
                    variant="solid"
                    size="2"
                    onClick={() => window.open('https://links.globalart.dev/astrum-repository-firewall', '_blank', 'noopener,noreferrer')}
                  >
                    {CommonStrings.ACTIONS.LEARN_MORE_TITLE} <ExternalLink size={12} />
                  </Button>
                </Flex>
              )}
            </Box>
          )}

          {/* Report metadata + violations (or firewall per-repo loading) */}
          {!((isFirewallUnavailable || isFirewallUnprotected ) || unsupportedMessage ) && (
            isFirewallReportBodyLoading ? (
              <Flex align="center" justify="center" gap="3" py="6" px="4" className="security-summary-modal__report-loading">
                <Spinner size="3" />
                <Text size="2" color="gray">{SecurityReportStrings.SUMMARY_MODAL.LOADING_FIREWALL_REPORT}</Text>
              </Flex>
            ) : (
              <>
                <Flex justify="between" align="start" gap="4" wrap="wrap" className="security-summary-modal__meta">
                  {!isFirewall && (reportDate || reportAge || componentsId > 0) && (
                    <Box>
                      <Text size="2" color="gray" as="div">
                        <Text weight="medium" size="2">{SecurityReportStrings.SUMMARY_MODAL.FOR}</Text> {repositoryName}
                      </Text>
                      {reportDate && (
                        <Text size="2" color="gray" as="div">
                          <Text weight="medium" size="2">{SecurityReportStrings.SUMMARY_MODAL.ON}</Text> {reportDate}
                        </Text>
                      )}
                      {reportAge && (
                        <Text size="2" color="gray" as="div">
                          <Text weight="medium" size="2">{SecurityReportStrings.SUMMARY_MODAL.AGE}</Text> {reportAge}
                        </Text>
                      )}
                    </Box>
                  )}
                  {isFirewall && (
                    <Box>
                      <Text size="2" color="gray" as="div">
                        <Text weight="medium" size="2">{SecurityReportStrings.SUMMARY_MODAL.FOR}</Text> {repositoryName}
                      </Text>
                    </Box>
                  )}
                  {isFirewall ? (
                    <Flex gap="5" align="start">
                      <Box style={{ textAlign: 'center' }}>
                        <Text size="4" weight="bold" as="div">{componentsId.toLocaleString()}</Text>
                        <Text size="1" color="gray" as="div">{SecurityReportStrings.SUMMARY_MODAL.COMPONENTS}</Text>
                        <Text size="1" color="gray" as="div">{SecurityReportStrings.SUMMARY_MODAL.AFFECTED_POLICY_VIOLATIONS}</Text>
                      </Box>
                      <Box style={{ textAlign: 'center' }}>
                        <Text size="4" weight="bold" as="div">{(quarantinedComponentCount ?? 0).toLocaleString()}</Text>
                        <Text size="1" color="gray" as="div">{SecurityReportStrings.SUMMARY_MODAL.QUARANTINED}</Text>
                        <Text size="1" color="gray" as="div">{SecurityReportStrings.SUMMARY_MODAL.COMPONENTS_LOWER}</Text>
                      </Box>
                    </Flex>
                  ) : (componentsId > 0 || componentsTot > 0) ? (
                    <Box className="security-summary-modal__components-block">
                      <Text size="4" weight="bold" as="div">{componentsId.toLocaleString()}</Text>
                      <Text size="1" color="gray" as="div">{SecurityReportStrings.SUMMARY_MODAL.COMPONENTS_IDENTIFIED}</Text>
                      <Text size="1" color="gray" as="div">{SecurityReportStrings.SUMMARY_MODAL.OF_TOTAL(pct, componentsTot.toLocaleString())}</Text>
                    </Box>
                  ) : null}
                </Flex>

                {/* Violation / Vulnerability counts */}
                {hasSecurityIssues && (
                  <Box className="security-summary-modal__section" mt="3">
                    <Text size="1" weight="bold" color="gray" className="security-summary-modal__section-title">
                      {isHealthCheck ? SecurityReportStrings.SUMMARY_MODAL.SECURITY_VULNERABILITIES : SecurityReportStrings.SUMMARY_MODAL.POLICY_VIOLATIONS}
                    </Text>
                    {isFirewall && (
                      <Text size="1" color="gray" as="div" mb="2">
                        {SecurityReportStrings.SUMMARY_MODAL.COMPONENTS_GROUPED_HIGHEST_VIOLATION_SEVERITY}
                      </Text>
                    )}
                    <Flex direction="column" gap="2" mt="2">
                      <SeverityRow label={SecurityReportStrings.SUMMARY_MODAL.CRITICAL_7_10} value={critical} color="red" />
                      <SeverityRow label={SecurityReportStrings.SUMMARY_MODAL.SEVERE_4_6} value={severe} color="orange" />
                      <SeverityRow label={SecurityReportStrings.SUMMARY_MODAL.MODERATE_1_3} value={moderate} color="yellow" />
                    </Flex>

                    {/* Threat bars -- Health Check only (real per-level data) */}
                    {isHealthCheck && threatBars.length > 0 && (
                      <Box className="security-summary-modal__threat-bars" mt="3">
                        {threatBars.map((count, i) => (
                          <Flex key={i} align="center" gap="2" mb="1">
                            <Text size="1" color="gray" style={{ width: 14 }}>{i + 1}</Text>
                            <Box
                              className="security-summary-modal__threat-bar"
                              style={{
                                width: `${maxThreat > 0 ? (count / maxThreat) * 100 : 0}%`,
                                backgroundColor: i >= 6 ? 'var(--red-9)' : i >= 3 ? 'var(--orange-9)' : 'var(--yellow-9)',
                              }}
                            />
                            {count > 0 && <Text size="1" color="gray">{count}</Text>}
                          </Flex>
                        ))}
                      </Box>
                    )}
                  </Box>
                )}
              </>
            )
          )}

          {/* Firewall: malicious packages summary */}
          {type === 'firewall' && !isFirewallUnavailable && !isFirewallUnprotected && !isFirewallReportBodyLoading && malwareSummary && (malwareSummary.scrubbedCount > 0 || malwareSummary.pendingCount > 0) && (
            <>
              <Separator size="4" my="3" />
              <Box>
                <Text size="1" weight="bold" color="gray">{SecurityReportStrings.SUMMARY_MODAL.MALICIOUS_PACKAGES}</Text>
                <Flex gap="4" mt="1">
                  <Text size="2" color="gray">{SecurityReportStrings.SUMMARY_MODAL.SCRUBBED(malwareSummary.scrubbedCount)}</Text>
                  {malwareSummary.pendingCount > 0 && (
                    <Text size="2" color="red">{SecurityReportStrings.SUMMARY_MODAL.PENDING_REMOVAL(malwareSummary.pendingCount)}</Text>
                  )}
                </Flex>
              </Box>
            </>
          )}

          {/* License Warnings -- Health Check only */}
          {isHealthCheck && hasLicenseData && (
            <Box className="security-summary-modal__section" mt="3">
              <Text size="1" weight="bold" color="gray" className="security-summary-modal__section-title">
                {SecurityReportStrings.SUMMARY_MODAL.LICENSE_WARNINGS}
              </Text>
              <Flex direction="column" gap="2" mt="2">
                {hasLicenseBreakdown ? (
                  <>
                    {(licenseCopyleftCount ?? 0) > 0 && <LicenseBadge label={SecurityReportStrings.SUMMARY_MODAL.COPYLEFT} value={licenseCopyleftCount} color="red" />}
                    {(licenseNonStandardCount ?? 0) > 0 && <LicenseBadge label={SecurityReportStrings.SUMMARY_MODAL.NON_STANDARD} value={licenseNonStandardCount} color="orange" />}
                    {(licenseNotProvidedCount ?? 0) > 0 && <LicenseBadge label={SecurityReportStrings.SUMMARY_MODAL.NOT_PROVIDED} value={licenseNotProvidedCount} color="yellow" />}
                    {(licenseWeakCopyleftCount ?? 0) > 0 && <LicenseBadge label={SecurityReportStrings.SUMMARY_MODAL.WEAK_COPYLEFT} value={licenseWeakCopyleftCount} color="amber" />}
                    {(licenseLiberalCount ?? 0) > 0 && <LicenseBadge label={SecurityReportStrings.SUMMARY_MODAL.LIBERAL} value={licenseLiberalCount} color="green" />}
                  </>
                ) : (licenseIssueCount ?? 0) > 0 ? (
                  <SeverityRow label={SecurityReportStrings.SUMMARY_MODAL.LICENSE_ISSUES} value={licenseIssueCount!} color="orange" />
                ) : null}
              </Flex>
            </Box>
          )}

          {/* Empty state (not while firewall per-repo report is still loading) */}
          {!(((((isFirewallUnavailable || isFirewallUnprotected ) || unsupportedMessage ) || isFirewallReportBodyLoading ) || hasSecurityIssues ) || hasLicenseData ) && (
            <Flex align="center" gap="3" p="4" className="security-summary-modal__empty">
              <ShieldCheck size={32} color="var(--green-9)" />
              <Box>
                <Text size="3" weight="bold" color="green">{SecurityReportStrings.SUMMARY_MODAL.NO_ISSUES_FOUND}</Text>
              </Box>
            </Flex>
          )}

          <Separator size="4" my="4" />

          {/* CTAs -- read-only: View Full Report and Browse Repo only */}
          <Flex gap="3" wrap="wrap" className="security-summary-modal__ctas">
            {reportUrl ? (
              <Button variant="soft" size="3" asChild>
                <a href={reportUrl} target="_blank" rel="noopener noreferrer">
                  {SecurityReportStrings.SUMMARY_MODAL.VIEW_FULL_REPORT} <ExternalLink size={14} />
                </a>
              </Button>
            ) : onViewFullReport ? (
              <Button variant="soft" size="3" onClick={onViewFullReport}>
                {SecurityReportStrings.SUMMARY_MODAL.VIEW_FULL_REPORT} <ExternalLink size={14} />
              </Button>
            ) : null}
            {onBrowseRepo && (
              <Button variant="ghost" size="2" onClick={onBrowseRepo}>
                {SecurityReportStrings.SUMMARY_MODAL.BROWSE_REPO}
              </Button>
            )}
          </Flex>
        </Box>
      </Dialog.Content>
    </Dialog.Root>
  );
}

function SeverityRow({
  label,
  value,
  color,
}: {
  label: string;
  value: number;
  color: 'red' | 'orange' | 'yellow';
}): JSX.Element {
  return (
    <Flex justify="between" align="center" gap="2">
      <Text size="2">{label}</Text>
      {value > 0 && (
        <span className={`security-summary-modal__badge security-summary-modal__badge--${color}`} data-value={value}>
          {value.toLocaleString()}
        </span>
      )}
    </Flex>
  );
}

function LicenseBadge({
  label,
  value,
  color,
}: {
  label: string;
  value?: number;
  color: 'red' | 'orange' | 'yellow' | 'amber' | 'green';
}): JSX.Element {
  const count = value ?? 0;
  if (count === 0) return <></>;
  return (
    <Flex justify="between" align="center" gap="2">
      <Text size="2">{label}</Text>
      <span className={`security-summary-modal__badge security-summary-modal__badge--${color}`} data-value={count}>
        {count.toLocaleString()}
      </span>
    </Flex>
  );
}

export default SecuritySummaryModal;
