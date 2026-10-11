

import React, { useEffect, useState, useMemo } from 'react';
import { Text, Heading, Box, Card, Flex, Button, Progress } from '@radix-ui/themes';
import { AlertCircle, AlertTriangle, RefreshCw } from 'lucide-react';
import { ExtJS } from '../../../../../interface/ExtJS';
import { Permissions } from '../../../../../constants/Permissions';
import { useMetricHealthApi } from '../../settings/support/metric-health/useMetricHealthApi';
import { getHealthStatus } from '../../settings/support/metric-health/types';
import { useCleanupPoliciesApi } from '../../settings/repository/cleanup/useCleanupPoliciesApi';
import { useRepositoriesByFormat } from './useRepositoriesByFormat';
import { useInstanceTotals } from './useInstanceTotals';
import { CE_WARN_THRESHOLD } from './ceThresholds';
import UIStrings from '../../../../../constants/UIStrings';

import './QuickActionStatsPanel.scss';

const {
  WELCOME: {
    ACTIONS: { SYSTEM_HEALTH, CLEANUP_POLICIES, BROWSE, SEARCH, CONNECT },
  },
} = UIStrings;

const SYSTEM_HEALTH_HREF = '#admin/support/metrichealth';
const BROWSE_HREF = '#browse';
const SEARCH_HREF = '#browse/search';
const CLEANUP_POLICIES_HREF = '#admin/repository/cleanup-policies';

function TileLoadingSkeleton() {
  return (
    <Flex align="center" gap="2" role="status" aria-live="polite" aria-busy="true">
      <RefreshCw size={16} aria-hidden="true" className="as-quick-action-stat-card__spinner" />
      <Text size="2" color="gray">Loading…</Text>
    </Flex>
  );
}

interface StatCardProps {
  title: string;
  href?: string;
  onClick?: () => void;
  children: React.ReactNode;
  className?: string;
}

function StatCard({ title, href, onClick, children, className }: StatCardProps) {
  const content = (
    <Box p="3" className="as-quick-action-stat-card__content">
      <Text size="2" color="gray" mb="1" as="span" className="as-quick-action-stat-card__title">
        {title}
      </Text>
      <Box className="as-quick-action-stat-card__value">{children}</Box>
    </Box>
  );

  const cardProps = {
    className: ['as-quick-action-stat-card', className].filter(Boolean).join(' '),
    size: '1' as const,
    variant: 'surface' as const,
  };

  if (href) {
    return (
      <Card {...cardProps} asChild>
        <a
          href={href}
          className={['as-quick-action-stat-card__link', className].filter(Boolean).join(' ')}
        >
          {content}
        </a>
      </Card>
    );
  }

  if (onClick) {
    return (
      <Card
        {...cardProps}
        tabIndex={0}
        role="button"
        onClick={onClick}
        onKeyDown={(e) => (e.key === 'Enter' || e.key === ' ') && (e.preventDefault(), onClick())}
      >
        {content}
      </Card>
    );
  }

  return <Card {...cardProps}>{content}</Card>;
}

export interface QuickActionStatsPanelProps {
  onConnectClick?: () => void;
}

export function QuickActionStatsPanel({ onConnectClick }: QuickActionStatsPanelProps) {
  const { fetchMetricHealth } = useMetricHealthApi();
  const { fetchCleanupPolicies } = useCleanupPoliciesApi();
  const reposByFormat = useRepositoriesByFormat();
  const instanceTotals = useInstanceTotals();

  const [healthChecks, setHealthChecks] = useState<{ healthy: number; unhealthy: number } | null>(null);
  const [healthChecksError, setHealthChecksError] = useState<boolean>(false);
  const [cleanupCount, setCleanupCount] = useState<number | null>(null);
  const [cleanupError, setCleanupError] = useState<boolean>(false);

  // Check if user has permission to view cleanup policies (requires admin permission)
  const canViewCleanupPolicies = ExtJS.checkPermission(Permissions.ADMIN) && ExtJS.state().getUser();
  const isCloud: boolean = ExtJS.state?.()?.getValue?.('isCloud', false) ?? false;

  useEffect(() => {
    let cancelled = false;
    setHealthChecksError(false);
    fetchMetricHealth()
      .then((checks) => {
        if (cancelled) return;
        const counts = checks.reduce(
          (acc, check) => {
            const status = getHealthStatus(check.result);
            if (status === 'healthy') acc.healthy++;
            else if (status === 'unhealthy') acc.unhealthy++;
            return acc;
          },
          { healthy: 0, unhealthy: 0 }
        );
        setHealthChecks(counts);
      })
      .catch(() => {
        // Hide the tile when the data source is unavailable rather than
        // rendering a misleading 0 unhealthy / 0 healthy (AC2, ASTRUM-54012).
        if (!cancelled) setHealthChecksError(true);
      });
    return () => {
      cancelled = true;
    };
  }, [fetchMetricHealth]);

  useEffect(() => {
    // Only fetch cleanup policies if user has permission
    if (!canViewCleanupPolicies) {
      return;
    }

    let cancelled = false;
    setCleanupError(false);
    fetchCleanupPolicies()
      .then((policies) => {
        if (!cancelled) setCleanupCount(Array.isArray(policies) ? policies.length : 0);
      })
      .catch(() => {
        // Hide the tile when the data source is unavailable rather than
        // rendering a misleading zero-state (AC2, ASTRUM-54012).
        if (!cancelled) setCleanupError(true);
      });
    return () => {
      cancelled = true;
    };
  }, [fetchCleanupPolicies, canViewCleanupPolicies]);

  const repoCount = useMemo(() => {
    if (reposByFormat.loading || reposByFormat.error) return null;
    return reposByFormat.data?.reduce((sum, f) => sum + f.totalCount, 0) ?? 0;
  }, [reposByFormat.loading, reposByFormat.error, reposByFormat.data]);

  // Use the decoupled Components view: it resolves as soon as
  // component_total_count arrives and does not wait on the peak-requests
  // metric (which this compact card never displays).
  const componentCount = instanceTotals.componentCount;
  const componentLimit = instanceTotals.componentLimit;
  const hasComponentLimit = componentLimit > 0;
  const componentRatio = hasComponentLimit && componentCount != null ? componentCount / componentLimit : 0;
  const componentExceeding = hasComponentLimit && componentRatio >= 1;
  const componentApproaching = hasComponentLimit && !componentExceeding && componentRatio >= CE_WARN_THRESHOLD;
  const componentBarColor = componentExceeding ? 'red' : componentApproaching ? 'orange' : 'blue';
  // peakRequestsPerDayLimit is intentionally not shown here — space constraints in the compact stat card.
  // InstanceTotalsPanel shows the full progress bar for both limits.

  const systemHealthClassName =
    healthChecks != null
      ? healthChecks.unhealthy > 0
        ? 'as-quick-action-stat-card--unhealthy'
        : 'as-quick-action-stat-card--healthy'
      : undefined;

  return (
    <div className="as-quick-action-stats">
      <div className="as-quick-action-stats__grid">
        {isCloud || healthChecksError ? null : (
          <StatCard
            title={SYSTEM_HEALTH.title}
            href={healthChecks != null ? SYSTEM_HEALTH_HREF : undefined}
            className={systemHealthClassName}
          >
            {healthChecks != null ? (
              <Flex direction="column" gap="2" className="as-system-health__progress">
                <Flex justify="between" align="baseline">
                  <Heading
                    as="h1"
                    size="6"
                    className={`as-quick-action-stat-card__number ${healthChecks.unhealthy > 0 ? 'as-system-health__number--unhealthy' : ''}`}
                  >
                    {healthChecks.unhealthy.toLocaleString()}
                  </Heading>
                  <Heading as="h1" size="6" className="as-quick-action-stat-card__number">
                    {healthChecks.healthy.toLocaleString()}
                  </Heading>
                </Flex>
                <Flex
                  className={`as-system-health__progress-bar ${
                    healthChecks.unhealthy + healthChecks.healthy === 0
                      ? 'as-system-health__progress-bar--empty'
                      : ''
                  }`}
                >
                  <Box
                    className="as-system-health__progress-segment as-system-health__progress-segment--unhealthy"
                    style={{
                      flex: healthChecks.unhealthy + healthChecks.healthy === 0 ? 1 : healthChecks.unhealthy,
                    }}
                  />
                  <Box
                    className="as-system-health__progress-segment as-system-health__progress-segment--healthy"
                    style={{
                      flex: healthChecks.unhealthy + healthChecks.healthy === 0 ? 1 : healthChecks.healthy,
                    }}
                  />
                </Flex>
                <Flex justify="between">
                  <Text size="1" color="gray">
                    Unhealthy
                  </Text>
                  <Text size="1" color="gray">
                    Healthy
                  </Text>
                </Flex>
              </Flex>
            ) : (
              <TileLoadingSkeleton />
            )}
          </StatCard>
        )}

        {reposByFormat.error ? null : (
          <StatCard title="Repositories">
            {reposByFormat.loading ? (
              <TileLoadingSkeleton />
            ) : repoCount != null ? (
              <Flex direction="column" gap="3">
                <Heading as="h1" size="6" className="as-quick-action-stat-card__number">
                  {repoCount.toLocaleString()}
                </Heading>
                <Flex gap="2" wrap="wrap">
                  <Button
                    size="2"
                    variant="surface"
                    color="blue"
                    onClick={() => {
                      window.location.hash = BROWSE_HREF;
                    }}
                  >
                    {BROWSE.title}
                  </Button>
                  <Button size="2" variant="surface" color="blue" onClick={onConnectClick}>
                    {CONNECT.title}
                  </Button>
                </Flex>
              </Flex>
            ) : null}
          </StatCard>
        )}

        <StatCard title="Components">
          {instanceTotals.componentsLoading ? (
            <Flex align="center" gap="2">
              <RefreshCw size={16} aria-hidden="true" className="as-quick-action-stat-card__spinner" />
              <Text size="2" color="gray">
                Loading…
              </Text>
            </Flex>
          ) : instanceTotals.componentsError ? (
            <Flex direction="column" gap="2" className="as-quick-action-stat-card__error">
              <Flex align="center" gap="2">
                <AlertCircle size={16} aria-hidden="true" className="as-quick-action-stat-card__icon--unhealthy" />
                <Text size="2" color="gray">
                  Couldn't load metrics
                </Text>
              </Flex>
              <Button size="2" variant="surface" color="blue" onClick={instanceTotals.retry}>
                Retry
              </Button>
            </Flex>
          ) : (
            // useInstanceTotals guarantees componentCount is non-null when
            // both componentsLoading and componentsError are false.
            <Flex direction="column" gap="3">
              <Flex align="baseline" gap="2">
                {componentExceeding && (
                  <AlertCircle size={16} aria-hidden="true" className="as-quick-action-stat-card__icon--unhealthy" />
                )}
                {componentApproaching && (
                  <AlertTriangle size={16} aria-hidden="true" className="as-quick-action-stat-card__icon--warning" />
                )}
                <Heading as="h1" size="6" className="as-quick-action-stat-card__number">
                  {componentCount?.toLocaleString()}
                </Heading>
              </Flex>
              {hasComponentLimit && (
                <Box>
                  <Progress
                    value={Math.min(componentRatio * 100, 100)}
                    max={100}
                    color={componentBarColor}
                    size="1"
                  />
                  <Flex justify="between" mt="1">
                    <Text size="1" color="gray">
                      {componentCount?.toLocaleString()} of {componentLimit.toLocaleString()}
                    </Text>
                    <Text size="1" color="gray">
                      limit
                    </Text>
                  </Flex>
                </Box>
              )}
              <Button
                size="2"
                variant="surface"
                color="blue"
                onClick={() => {
                  window.location.hash = SEARCH_HREF;
                }}
              >
                {SEARCH.title}
              </Button>
            </Flex>
          )}
        </StatCard>

        {canViewCleanupPolicies && !cleanupError && (cleanupCount === null ? (
          <StatCard title={CLEANUP_POLICIES.title}>
            <TileLoadingSkeleton />
          </StatCard>
        ) : cleanupCount === 0 ? (
          <StatCard
            title={CLEANUP_POLICIES.title}
            className="as-quick-action-stat-card--zero-cleanup"
          >
            <Flex direction="column" gap="3">
              <Heading as="h1" size="6" className="as-quick-action-stat-card__number">
                0
              </Heading>
              <Button
                size="2"
                variant="surface"
                color="blue"
                onClick={() => {
                  window.location.hash = CLEANUP_POLICIES_HREF;
                }}
              >
                Add Cleanup Policy
              </Button>
            </Flex>
          </StatCard>
        ) : (
          <StatCard title={CLEANUP_POLICIES.title} href={CLEANUP_POLICIES_HREF}>
            <Heading as="h1" size="6" className="as-quick-action-stat-card__number">
              {cleanupCount.toLocaleString()}
            </Heading>
          </StatCard>
        ))}
      </div>
    </div>
  );
}

export default QuickActionStatsPanel;
