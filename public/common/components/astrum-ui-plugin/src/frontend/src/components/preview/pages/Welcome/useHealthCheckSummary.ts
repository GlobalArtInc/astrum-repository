

/**
 * useHealthCheckSummary - Aggregates RHC data for the dashboard Health Check card.
 *
 * Uses GET /service/rest/internal/ui/healthcheck/summary plus repository details so coverage
 * matches Protect Quick Config: only proxy repos whose format supports Health Check count toward
 * enabled/total. Eligible repos missing from the summary response count as not enabled.
 *
 * The malware count (astrum.malware.count) comes from ExtJS state, not this API,
 * so it's read separately and merged in MalwareStatusCard / HealthCheckStatusCard.
 */

import { useState, useEffect, useCallback, useMemo } from 'react';
import Axios from 'axios';
import { restClient, ENDPOINTS } from '../../../../interface/api';
import { isHealthCheckSupportedFormat } from '../../../../utils/healthCheckFormats';

export interface HealthCheckRepoStatus {
  repositoryName: string;
  enabled: boolean;
  analyzing?: boolean;
  securityIssueCount?: number;
  licenseIssueCount?: number;
  /** Epoch millis from server when repo was last analyzed */
  lastAnalyzedDate?: number | null;
}

export interface HealthCheckSummary {
  loading: boolean;
  error: string | null;
  /** Eligible proxy repos with Health Check enabled */
  enabledCount: number;
  /** Eligible proxy repos (format supports RHC); denominator for coverage */
  totalProxyCount: number;
  /** Proxy repos whose format does not support Health Check (excluded from coverage) */
  unsupportedFormatProxyCount: number;
  totalSecurityIssues: number;
  totalLicenseIssues: number;
  repos: HealthCheckRepoStatus[];
  refetch: () => void;
}

function isRequestAborted(err: unknown): boolean {
  return Axios.isCancel(err) || (err as { code?: string })?.code === 'ERR_CANCELED';
}

export function useHealthCheckSummary(): HealthCheckSummary {
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [repos, setRepos] = useState<HealthCheckRepoStatus[]>([]);
  const [eligibleRepoNames, setEligibleRepoNames] = useState<string[]>([]);
  const [unsupportedFormatProxyCount, setUnsupportedFormatProxyCount] = useState(0);
  const [_refetchTrigger, setRefetchTrigger] = useState(0);

  useEffect(() => {
    const controller = new AbortController();
    const { signal } = controller;

    (async () => {
      setLoading(true);
      setError(null);
      try {
        const [data, detailsRaw] = await Promise.all([
          restClient.get<HealthCheckRepoStatus[]>(ENDPOINTS.HEALTH_CHECK_SUMMARY, { signal }),
          restClient.get<{ name: string; type: string; format: string }[]>(ENDPOINTS.REPOSITORIES_DETAILS, {
            signal,
          }),
        ]);
        if (signal.aborted) {
          return;
        }
        const hcList = Array.isArray(data) ? data : [];
        const details = Array.isArray(detailsRaw) ? detailsRaw : [];
        const proxyRepos = details.filter((r) => r.type === 'proxy');
        const eligible = proxyRepos.filter((r) => isHealthCheckSupportedFormat(r.format));
        setRepos(hcList);
        setEligibleRepoNames(eligible.map((r) => r.name));
        setUnsupportedFormatProxyCount(proxyRepos.length - eligible.length);
      } catch (err) {
        if (isRequestAborted(err) || signal.aborted) {
          return;
        }
        setRepos([]);
        setEligibleRepoNames([]);
        setUnsupportedFormatProxyCount(0);
        setError(err instanceof Error ? err.message : 'Failed to load Health Check data');
      } finally {
        if (!signal.aborted) {
          setLoading(false);
        }
      }
    })();

    return () => controller.abort();
  }, []);

  const refetch = useCallback(() => setRefetchTrigger((n) => n + 1), []);

  const hcByName = useMemo(() => new Map(repos.map((r) => [r.repositoryName, r])), [repos]);
  const eligibleSet = useMemo(() => new Set(eligibleRepoNames), [eligibleRepoNames]);

  const enabledCount = useMemo(
    () => eligibleRepoNames.filter((name) => hcByName.get(name)?.enabled).length,
    [eligibleRepoNames, hcByName],
  );
  const totalProxyCount = eligibleRepoNames.length;

  const enabledRepos = useMemo(
    () => repos.filter((r) => r.enabled && eligibleSet.has(r.repositoryName)),
    [repos, eligibleSet],
  );
  const totalSecurityIssues = useMemo(
    () => enabledRepos.reduce((s, r) => s + (r.securityIssueCount ?? 0), 0),
    [enabledRepos],
  );
  const totalLicenseIssues = useMemo(
    () => enabledRepos.reduce((s, r) => s + (r.licenseIssueCount ?? 0), 0),
    [enabledRepos],
  );

  return {
    loading,
    error,
    enabledCount,
    totalProxyCount,
    unsupportedFormatProxyCount,
    totalSecurityIssues,
    totalLicenseIssues,
    repos,
    refetch,
  };
}
