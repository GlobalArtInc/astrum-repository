

import React, { useEffect, useState, useCallback } from 'react';
import { useRouter } from '@uirouter/react';
import { Flex, Text, Box, Button, Badge } from '@radix-ui/themes';
import { ArrowLeft, ExternalLink, ShieldCheck, Shield, AlertTriangle } from 'lucide-react';
import { restClient, ENDPOINTS } from '../../../../interface/api';
import { LoadingState, ErrorState } from '..';
import { ArtifactTable } from './ArtifactTable';
import { useArtifactList } from './useArtifactList';
import type { SecurityStatusData } from './security.types';

import './SecurityReportPage.scss';
import SecurityReportStrings from '../../constants/SecurityReportStrings';
import CommonStrings from '../../constants/CommonStrings';

export type SecurityReportType = 'health-check' | 'firewall';

export interface SecurityReportPageProps {
  repositoryName: string;
  reportType: SecurityReportType;
  /** When true, hide Back button and breadcrumb (embedded in tab) */
  embedded?: boolean;
}

interface HealthCheckApiItem {
  repositoryName: string;
  enabled?: boolean;
  detailUrl?: string;
  summaryUrl?: string;
  securityIssueCount?: number;
  licenseIssueCount?: number;
  results?: { criticalCount?: number; severeCount?: number; totalCount?: number };
}

interface FirewallApiItem {
  repositoryName: string;
  affectedComponentCount?: number;
  criticalComponentCount?: number;
  severeComponentCount?: number;
  moderateComponentCount?: number;
  quarantinedComponentCount?: number;
  reportUrl?: string;
}

/**
 * SecurityReportPage - Full-page security report (Health Check or Firewall).
 * Replaces the cramped modal with a dedicated page; iframe fills viewport.
 */
export function SecurityReportPage({ repositoryName, reportType, embedded = false }: SecurityReportPageProps): JSX.Element {
  const router = useRouter();
  const [data, setData] = useState<SecurityStatusData | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const title = reportType === 'health-check' ? SecurityReportStrings.REPORT_PAGE.HEALTH_CHECK_REPORT : SecurityReportStrings.REPORT_PAGE.FIREWALL_REPORT;
  const breadcrumbReportLabel = reportType === 'health-check' ? SecurityReportStrings.REPORT_PAGE.HEALTH_CHECK_REPORT : SecurityReportStrings.REPORT_PAGE.FIREWALL_REPORT;

  const fetchData = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      if (reportType === 'health-check') {
        const items = await restClient.get<HealthCheckApiItem[]>(ENDPOINTS.HEALTH_CHECK);
        const item = (items || []).find((i) => i.repositoryName === repositoryName);
        if (!item) {
          setError(SecurityReportStrings.REPORT_PAGE.NO_HEALTH_CHECK_DATA_FOUND(repositoryName));
          setData(null);
          return;
        }
        const reportUrl = item.detailUrl || item.summaryUrl;
        setData({
          repositoryName,
          affectedComponentCount: item.results?.totalCount ?? (item.securityIssueCount || 0) + (item.licenseIssueCount || 0),
          criticalComponentCount: item.securityIssueCount ?? item.results?.criticalCount ?? 0,
          severeComponentCount: item.results?.severeCount ?? 0,
          moderateComponentCount: item.licenseIssueCount ?? 0,
          quarantinedComponentCount: 0,
          reportUrl: reportUrl || undefined,
        });
      } else {
        const items = await restClient.get<FirewallApiItem[]>(ENDPOINTS.FIREWALL_STATUS);
        const item = (items || []).find((i) => i.repositoryName === repositoryName);
        if (!item) {
          setError(SecurityReportStrings.REPORT_PAGE.NO_FIREWALL_DATA_FOUND(repositoryName));
          setData(null);
          return;
        }
        setData({
          repositoryName,
          affectedComponentCount: item.affectedComponentCount ?? 0,
          criticalComponentCount: item.criticalComponentCount ?? 0,
          severeComponentCount: item.severeComponentCount ?? 0,
          moderateComponentCount: item.moderateComponentCount ?? 0,
          quarantinedComponentCount: item.quarantinedComponentCount ?? 0,
          reportUrl: item.reportUrl,
        });
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : SecurityReportStrings.REPORT_PAGE.FAILED_LOAD_REPORT_DATA);
      setData(null);
    } finally {
      setLoading(false);
    }
  }, [repositoryName, reportType]);

  useEffect(() => {
    fetchData();
  }, [fetchData]);

  const handleBack = useCallback(() => {
    router.stateService.go('preview.browse.browse');
  }, [router]);

  const hasIssues =
    data &&
    (data.criticalComponentCount > 0 || data.severeComponentCount > 0 || data.moderateComponentCount > 0);

  const artifactList = useArtifactList(repositoryName, reportType);
  const useArtifactTable = artifactList.endpointAvailable === true;

  if (loading) {
    return (
      <Box className="security-report-page">
        <LoadingState message={SecurityReportStrings.REPORT_PAGE.LOADING_REPORT} />
      </Box>
    );
  }

  if (error) {
    return (
      <Box className="security-report-page">
        {!embedded && (
          <Box className="security-report-page__header">
            <Button variant="ghost" onClick={handleBack}>
              <ArrowLeft size={16} />
              {CommonStrings.ACTIONS.BACK}
            </Button>
          </Box>
        )}
        <ErrorState title={SecurityReportStrings.REPORT_PAGE.REPORT_UNAVAILABLE} message={error} action={embedded ? undefined : { label: SecurityReportStrings.REPORT_PAGE.BACK_BROWSE, onClick: handleBack }} />
      </Box>
    );
  }

  if (!data) {
    return null;
  }

  return (
    <Box className="security-report-page">
      <Box className="security-report-page__header">
        <Flex align="center" gap="3" wrap="wrap">
          {!embedded && (
            <>
              <Button variant="ghost" color="gray" onClick={handleBack} className="security-report-page__back">
                <ArrowLeft size={16} />
                {CommonStrings.ACTIONS.BACK}
              </Button>
              <Box className="security-report-page__breadcrumb">
                <Text size="1" color="gray">
                  {SecurityReportStrings.REPORT_PAGE.BROWSE(repositoryName, breadcrumbReportLabel)}
                </Text>
              </Box>
            </>
          )}
          {data.reportUrl && (
            <Button variant="soft" asChild>
              <a href={data.reportUrl} target="_blank" rel="noopener noreferrer" className="security-report-page__open-iq">
                <ExternalLink size={16} />
                {SecurityReportStrings.REPORT_PAGE.OPEN_IQ_SERVER}
              </a>
            </Button>
          )}
        </Flex>
        <Flex align="center" gap="3" mt="2">
          {reportType === 'health-check' ? (
            <ShieldCheck size={24} color="var(--accent-9)" />
          ) : (
            <Shield size={24} color="var(--orange-9)" />
          )}
          <Box>
            <Text size="5" weight="bold">{title}</Text>
            <Text size="2" color="gray" as="p">{repositoryName}</Text>
          </Box>
        </Flex>
      </Box>

      <Box className="security-report-page__body">
        <Flex direction="column" gap="4" className="security-report-page__content">
            {/* Quarantine badge (Firewall only) */}
            {reportType === 'firewall' && data.quarantinedComponentCount > 0 && (
              <Badge color="orange" size="2" variant="soft" className="security-report-page__quarantine">
                <AlertTriangle size={16} />
                {SecurityReportStrings.REPORT_PAGE.COMPONENTS_CURRENTLY_QUARANTINE(data.quarantinedComponentCount)}
              </Badge>
            )}

            {/* Summary row – three severity cards + artifacts identified */}
            <Flex gap="4" wrap="wrap" className="security-report-page__summary">
              <SummaryCard
                label={SecurityReportStrings.REPORT_PAGE.CRITICAL}
                value={data.criticalComponentCount}
                color="red"
              />
              <SummaryCard
                label={SecurityReportStrings.REPORT_PAGE.SEVERE}
                value={data.severeComponentCount}
                color="orange"
              />
              <SummaryCard
                label={SecurityReportStrings.REPORT_PAGE.MODERATE}
                value={data.moderateComponentCount}
                color="yellow"
              />
              <SummaryCard
                label={SecurityReportStrings.REPORT_PAGE.ARTIFACTS_IDENTIFIED}
                value={data.affectedComponentCount}
                color="gray"
              />
            </Flex>

            {/* Report content: native ArtifactTable when backend supports it, else iframe */}
            <Box className="security-report-page__iframe-container">
              {useArtifactTable ? (
                <ArtifactTable
                  items={artifactList.items}
                  loading={artifactList.loading}
                  error={artifactList.error}
                  hasMore={artifactList.hasMore}
                  onLoadMore={artifactList.loadMore}
                />
              ) : data.reportUrl ? (
                <iframe
                  src={data.reportUrl}
                  title={SecurityReportStrings.REPORT_PAGE.FOR(title, repositoryName)}
                  className="security-report-page__iframe"
                  sandbox="allow-same-origin allow-scripts"
                />
              ) : (
                <Flex direction="column" align="center" justify="center" p="8" className="security-report-page__empty">
                  <ShieldCheck size={48} color="var(--gray-7)" />
                  <Text size="3" weight="medium" color="gray" mt="3">
                    {hasIssues ? SecurityReportStrings.REPORT_PAGE.FULL_REPORT_DATA_BEING_PROCESSED : SecurityReportStrings.REPORT_PAGE.NO_SECURITY_ISSUES_DETECTED}
                  </Text>
                  <Text size="2" color="gray" mt="1">
                    {SecurityReportStrings.REPORT_PAGE.RUN_NEW_ANALYSIS_UPDATE_THESE}
                  </Text>
                </Flex>
              )}
            </Box>
        </Flex>
      </Box>
    </Box>
  );
}

function SummaryCard({
  label,
  value,
  color,
}: {
  label: string;
  value: number;
  color: 'red' | 'orange' | 'yellow' | 'gray';
}): JSX.Element {
  const colorStyle = color === 'gray'
    ? { color: 'var(--gray-11)' }
    : { color: `var(--${color}-9)` };
  return (
    <Box className={`security-report-page__card security-report-page__card--${color}`}>
      <Flex direction="column" align="center" gap="1">
        <Text size="2" weight="bold" style={color === 'gray' ? { color: 'var(--gray-11)' } : { color: `var(--${color}-9)`, textTransform: 'uppercase', letterSpacing: '0.05em' }}>
          {label}
        </Text>
        <Text size="6" weight="bold" style={colorStyle}>
          {value.toLocaleString()}
        </Text>
      </Flex>
    </Box>
  );
}

export default SecurityReportPage;
