

import React, { useState } from 'react';
import { Badge, Dialog, Flex, Text, Box, Button, Spinner, Tooltip } from '@radix-ui/themes';
import { HelpCircle, ShieldCheck, Skull, X } from 'lucide-react';
import { useRouter } from '@uirouter/react';

import { SecurityRepositoryInfo } from './security.types';
import { SecuritySummaryModal } from './SecuritySummaryModal';

import './HealthCheckCell.scss';
import { isHealthCheckSupportedFormat } from '../../../../utils/healthCheckFormats';
import SecurityReportStrings from '../../constants/SecurityReportStrings';

// Health check status from API - matching the shape returned by the backend
export interface HealthCheckStatus {
  enabled?: boolean;
  analyzing?: boolean;
  detailedReport?: string | null;
  summaryUrl?: string;
  securityIssueCount?: number;
  licenseIssueCount?: number;
  /** Malicious component count from HDS when health check data is available; omit when unknown */
  malwareCount?: number | null;
  licenseCopyleftCount?: number;
  licenseNonStandardCount?: number;
  licenseNotProvidedCount?: number;
  licenseWeakCopyleftCount?: number;
  licenseLiberalCount?: number;
  iframeHeight?: number;
  iframeWidth?: number;
  results?: { criticalCount?: number; severeCount?: number; moderateCount?: number; totalCount?: number };
  totalCounts?: number[];
  vulnerableCounts?: number[];
  reportDate?: string;
  reportAge?: string;
}

export interface HealthCheckCellProps {
  /** Repository data */
  repository: SecurityRepositoryInfo;
  /** Health check status from API */
  healthStatus?: HealthCheckStatus;
  /** Callback when Analyze button is clicked */
  onAnalyze?: (repositoryName: string) => void;
  /** Whether the analyze action is loading */
  analyzeLoading?: boolean;
  /** Whether this repo is known to be supported by the health check summary API */
  rhcSupportedByBackend?: boolean;
}

/**
 * HealthCheckCell displays the health check status for a repository.
 *
 * Shows one of:
 * - Vulnerability counts (shield for security, scale for license)
 * - "Analyzing..." when analysis is in progress
 * - "Analyze" button to start analysis
 * - Empty/disabled for unsupported formats
 *
 * Click cell → center modal. "View full report" navigates to repo profile Health Check tab.
 */
export function HealthCheckCell({
  repository,
  healthStatus,
  onAnalyze,
  analyzeLoading = false,
  rhcSupportedByBackend,
}: HealthCheckCellProps): JSX.Element | null {
  let router: ReturnType<typeof useRouter> | null = null;
  try { router = useRouter(); } catch { /* no UIRouter context */ }
  const [showModal, setShowModal] = useState(false);
  const [showUnsupportedModal, setShowUnsupportedModal] = useState(false);

  if (repository.type !== 'proxy') {
    const reason = repository.type === 'group'
      ? SecurityReportStrings.HEALTH_CHECK_CELL.HEALTH_CHECK_RUNS_INDIVIDUAL_PROXY
      : SecurityReportStrings.HEALTH_CHECK_CELL.HEALTH_CHECK_ANALYZES_COMPONENTS_FETCHED;
    return (
      <Tooltip content={reason}>
        <span className="health-check-cell health-check-cell--na">
          <Text size="1" color="gray">{SecurityReportStrings.HEALTH_CHECK_CELL.NOT_SUPPORTED}</Text>
        </span>
      </Tooltip>
    );
  }

  const formatSupported = isHealthCheckSupportedFormat(repository.format);
  const supportsHealthCheck = formatSupported && (rhcSupportedByBackend !== false);

  const unsupportedTooltip = formatSupported
    ? SecurityReportStrings.HEALTH_CHECK_CELL.HEALTH_CHECK_ONLY_SUPPORTS_MAVEN
    : SecurityReportStrings.HEALTH_CHECK_CELL.HEALTH_CHECK_NOT_AVAILABLE_REPOSITORIES(repository.format);

  if (!supportsHealthCheck) {
    return (
      <>
        <Tooltip content={unsupportedTooltip}>
          <Flex
            align="center"
            gap="1"
            className="health-check-cell health-check-cell--unsupported health-check-cell--clickable"
            onClick={(e) => { e.stopPropagation(); setShowUnsupportedModal(true); }}
            role="button"
            tabIndex={0}
            onKeyDown={(e) => (e.key === 'Enter' || e.key === ' ') && setShowUnsupportedModal(true)}
            aria-label={unsupportedTooltip}
            style={{ cursor: 'help' }}
          >
            <Text size="1" color="gray">{SecurityReportStrings.HEALTH_CHECK_CELL.NOT_SUPPORTED}</Text>
            <HelpCircle size={12} color="var(--gray-8)" />
          </Flex>
        </Tooltip>
        {showUnsupportedModal && (
          <SecuritySummaryModal
            repositoryName={repository.name}
            data={{
              repositoryName: repository.name,
              affectedComponentCount: 0,
              criticalComponentCount: 0,
              severeComponentCount: 0,
              moderateComponentCount: 0,
              quarantinedComponentCount: 0,
            }}
            isOpen={showUnsupportedModal}
            onClose={() => setShowUnsupportedModal(false)}
            type="health-check"
            unsupportedMessage={unsupportedTooltip}
            onBrowseRepo={() => {
              setShowUnsupportedModal(false);
              router?.stateService.go('preview.browse.browse.repo', { repoName: repository.name });
            }}
          />
        )}
      </>
    );
  }

  // Show analyzing state
  if (healthStatus?.analyzing || analyzeLoading) {
    return (
      <Flex align="center" gap="2" className="health-check-cell health-check-cell--analyzing">
        <Spinner size="1" />
        <Text size="1" color="gray">{SecurityReportStrings.HEALTH_CHECK_CELL.ANALYZING}</Text>
      </Flex>
    );
  }

  // Show vulnerability counts if enabled and has data
  if (healthStatus?.enabled) {
    const securityIssues = healthStatus.securityIssueCount ?? 0;
    const licenseIssues = healthStatus.licenseIssueCount ?? 0;
    const malwareCount = healthStatus.malwareCount;
    const showMalwareBadge = malwareCount != null;
    const iframeH = healthStatus.iframeHeight ?? 480;
    const iframeW = healthStatus.iframeWidth ?? 640;

    const openModal = (e: React.MouseEvent) => {
      e.stopPropagation();
      setShowModal(true);
    };

    const malwareAria = showMalwareBadge ? SecurityReportStrings.HEALTH_CHECK_CELL.MALWARE(malwareCount) : '';

    return (
      <>
        <Flex
          align="center"
          gap="3"
          className="health-check-cell health-check-cell--enabled health-check-cell--clickable"
          onClick={openModal}
          role="button"
          tabIndex={0}
          onKeyDown={(e) => (e.key === 'Enter' || e.key === ' ') && openModal(e as unknown as React.MouseEvent)}
          aria-label={SecurityReportStrings.HEALTH_CHECK_CELL.HEALTH_CHECK_SECURITY_LICENSE_ISSUES(securityIssues, licenseIssues, malwareAria)}
        >
          <Flex align="center" gap="1" className="health-check-cell__metric">
            <Tooltip content={SecurityReportStrings.HEALTH_CHECK_CELL.SECURITY_ISSUES}>
              <Badge
                color={securityIssues === 0 ? 'green' : 'red'}
                variant="solid"
                radius="full"
                size="1"
              >
                {securityIssues}
              </Badge>
            </Tooltip>
          </Flex>
          <Flex align="center" gap="1" className="health-check-cell__metric">
            <Tooltip content={SecurityReportStrings.HEALTH_CHECK_CELL.LICENSE_ISSUES}>
              <Badge
                color={licenseIssues === 0 ? 'green' : 'orange'}
                variant="solid"
                radius="full"
                size="1"
              >
                {licenseIssues}
              </Badge>
            </Tooltip>
          </Flex>
          {showMalwareBadge && (
            <Flex align="center" gap="1" className="health-check-cell__metric">
              <Tooltip
                content={
                  malwareCount === 0
                    ? SecurityReportStrings.HEALTH_CHECK_CELL.NO_MALICIOUS_COMPONENTS_DETECTED
                    : SecurityReportStrings.HEALTH_CHECK_CELL.MALICIOUS_COMPONENTS_DETECTED
                }
              >
                <Badge
                  color={malwareCount === 0 ? 'green' : 'red'}
                  variant="solid"
                  radius="full"
                  size="1"
                >
                  <span className="health-check-cell__malware-badge-inner">
                    <Skull size={10} strokeWidth={2.25} aria-hidden />
                    {malwareCount}
                  </span>
                </Badge>
              </Tooltip>
            </Flex>
          )}
        </Flex>
        <Dialog.Root open={showModal} onOpenChange={setShowModal}>
          <Dialog.Content
            style={{ maxWidth: iframeW + 48, padding: 0, overflow: 'hidden' }}
            onClick={(e) => e.stopPropagation()}
          >
            <Flex align="center" justify="between" px="4" py="3">
              <Flex align="center" gap="2">
                <ShieldCheck size={18} color="var(--green-9)" />
                <Dialog.Title size="4" weight="bold" style={{ margin: 0 }}>
                  {SecurityReportStrings.HEALTH_CHECK_CELL.HEALTH_CHECK(repository.name)}
                </Dialog.Title>
              </Flex>
              <Dialog.Close>
                <Button variant="ghost" size="1" color="gray" style={{ cursor: 'pointer' }}>
                  <X size={16} />
                </Button>
              </Dialog.Close>
            </Flex>
            {healthStatus.summaryUrl ? (
              <Box px="4" pb="4">
                <iframe
                  src={healthStatus.summaryUrl}
                  width={iframeW}
                  height={iframeH}
                  style={{ border: 'none', display: 'block' }}
                  title={SecurityReportStrings.HEALTH_CHECK_CELL.HEALTH_CHECK_SUMMARY(repository.name)}
                />
              </Box>
            ) : (
              <Box px="4" pb="4">
                <Text size="2" color="gray">{SecurityReportStrings.HEALTH_CHECK_CELL.SUMMARY_REPORT_NOT_AVAILABLE}</Text>
              </Box>
            )}
            <Flex px="4" pb="3" gap="3" align="center">
              {healthStatus.detailedReport && (
                <Button
                  variant="soft"
                  size="1"
                  onClick={() => window.open(healthStatus.detailedReport!, '_blank', 'noopener,noreferrer')}
                >
                  {SecurityReportStrings.HEALTH_CHECK_CELL.VIEW_FULL_REPORT}
                </Button>
              )}
              <Button
                variant="ghost"
                size="1"
                color="gray"
                onClick={() => {
                  setShowModal(false);
                  router?.stateService.go('preview.browse.browse.repo', { repoName: repository.name });
                }}
              >
                {SecurityReportStrings.HEALTH_CHECK_CELL.BROWSE_REPO}
              </Button>
            </Flex>
          </Dialog.Content>
        </Dialog.Root>
      </>
    );
  }

  // Show Analyze button if not enabled or no status yet
  if (onAnalyze) {
    return (
      <Button
        size="1"
        variant="soft"
        color="blue"
        onClick={(e) => {
          e.stopPropagation();
          onAnalyze(repository.name);
        }}
        className="health-check-cell__analyze-btn"
      >
        {SecurityReportStrings.HEALTH_CHECK_CELL.ANALYZE}
      </Button>
    );
  }

  // Default: not enabled, no action
  return (
    <Tooltip content={SecurityReportStrings.HEALTH_CHECK_CELL.HEALTH_CHECK_NOT_YET_ENABLED}>
      <span className="health-check-cell health-check-cell--disabled">
        <Text size="2" color="gray">{SecurityReportStrings.HEALTH_CHECK_CELL.N_A}</Text>
      </span>
    </Tooltip>
  );
}

export default HealthCheckCell;
