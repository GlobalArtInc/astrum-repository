import React, { useState } from 'react';
import { Box, Button, Callout, Card, Flex, Heading, SegmentedControl, Select, Skeleton, Spinner, Text } from '@radix-ui/themes';
import { AlertTriangle, RefreshCw } from 'lucide-react';
import UIStrings from '../../../../../constants/UIStrings';
import { DownloadsChart, DownloadsChartLegend } from './DownloadsChart';
import { RecentComponentsCard } from './RecentComponentsCard';
import { RepositoriesCard } from './RepositoriesCard';
import { TopComponentsCard } from './TopComponentsCard';
import { UnusedCard } from './UnusedCard';
import { UsageKpiSkeletons, UsageKpiTiles } from './UsageKpiTiles';
import { formatDay, formatDays, formatNumber } from './usageFormat';
import { useUsageDashboard } from './useUsageDashboard';
import { USAGE_PERIODS, type UsageDashboardData } from './usageDashboard.types';

import './UsageDashboard.scss';

const ALL = '*';

const SKELETON_ROWS = ['first', 'second', 'third', 'fourth', 'fifth'];

export interface UsageDashboardProps {
  /** Whether to link to the search. */
  canSearch?: boolean;
  /** Whether to offer configuring cleanup policies. */
  canConfigureCleanup?: boolean;
  /** The period shown first, in days. */
  initialDays?: number;
}

function ChartCard({ data }: { data: UsageDashboardData }): JSX.Element {
  const strings = UIStrings.USAGE_DASHBOARD.CHART;
  const period = formatDays(data.days);
  const total = data.totals.downloadsInPeriod;
  const peak = data.downloadsPerDay.reduce<UsageDashboardData['downloadsPerDay'][number] | null>(
    (best, day) => (best === null || day.downloads > best.downloads ? day : best), null);
  return (
    <Card size="3" className="as-usage-dashboard__panel" aria-label={strings.TITLE}>
      <Flex justify="between" align="baseline" gap="3" wrap="wrap" mb="4">
        <div>
          <Heading as="h2" size="4">{strings.TITLE}</Heading>
          <Text as="p" size="2" color="gray" mt="1">
            {total > 0 && peak
              ? strings.SUBTITLE(period, formatNumber(total), formatNumber(peak.downloads), formatDay(peak.day))
              : strings.SUBTITLE_EMPTY(period)}
          </Text>
        </div>
        {total > 0 && <DownloadsChartLegend />}
      </Flex>
      <DownloadsChart days={data.downloadsPerDay} />
    </Card>
  );
}

function DashboardSkeleton(): JSX.Element {
  return (
    <>
      <UsageKpiSkeletons />
      <Card size="3" className="as-usage-dashboard__panel">
        <Skeleton width="30%" height="20px" mb="4" />
        <Skeleton width="100%" height="240px" />
      </Card>
      <div className="as-usage-dashboard__row">
        {['wide', 'narrow'].map((width) => (
          <Card key={width} size="3" className={`as-usage-dashboard__panel as-usage-dashboard__${width}`}>
            <Flex direction="column" gap="3">
              <Skeleton width="40%" height="20px" />
              {SKELETON_ROWS.map((row) => <Skeleton key={row} width="100%" height="18px" />)}
            </Flex>
          </Card>
        ))}
      </div>
    </>
  );
}

/**
 * How much of what is used in the repositories the user may browse: headline figures, downloads per day, the most
 * downloaded components, the largest and busiest repositories, what was published lately and what nobody downloads.
 */
export function UsageDashboard({
  canSearch = true,
  canConfigureCleanup = false,
  initialDays = 30,
}: UsageDashboardProps): JSX.Element {
  const strings = UIStrings.USAGE_DASHBOARD;
  const [days, setDays] = useState(initialDays);
  const [repository, setRepository] = useState<string | null>(null);
  const { data, loading, error, reload } = useUsageDashboard(days, repository);
  const repositories = data?.repositories ?? [];
  const period = formatDays(days);

  return (
    <Flex direction="column" gap="5" className="as-usage-dashboard" data-testid="usage-dashboard"
      aria-busy={loading}>
      <Flex justify="between" align="end" gap="4" wrap="wrap" className="as-usage-dashboard__header">
        <div>
          <Heading as="h1" size="7" className="as-usage-dashboard__title">{strings.TITLE}</Heading>
          <Text as="p" size="2" color="gray" mt="1">
            {repository ? strings.SUBTITLE_REPOSITORY(repository, period) : strings.SUBTITLE(period)}
          </Text>
        </div>
        <Flex gap="3" align="center" wrap="wrap" className="as-usage-dashboard__controls">
          {loading && data && <Spinner size="2" aria-label={strings.LOADING} />}
          <SegmentedControl.Root value={String(days)} aria-label={strings.PERIOD_LABEL}
            onValueChange={(value) => setDays(Number(value))}>
            {USAGE_PERIODS.map((each) => (
              <SegmentedControl.Item key={each} value={String(each)}>{formatDays(each)}</SegmentedControl.Item>
            ))}
          </SegmentedControl.Root>
          <Select.Root value={repository ?? ALL} onValueChange={(value) => setRepository(value === ALL ? null : value)}
            disabled={repositories.length === 0}>
            <Select.Trigger aria-label={strings.REPOSITORY_LABEL} className="as-usage-dashboard__repository-select" />
            <Select.Content position="popper">
              <Select.Item value={ALL}>{strings.ALL_REPOSITORIES}</Select.Item>
              {repositories.length > 0 && <Select.Separator />}
              {repositories.map((each) => (
                <Select.Item key={each.name} value={each.name}>{each.name}</Select.Item>
              ))}
            </Select.Content>
          </Select.Root>
        </Flex>
      </Flex>

      {error && (
        <Callout.Root color="red" role="alert">
          <Callout.Icon><AlertTriangle size={16} /></Callout.Icon>
          <Callout.Text>
            <Flex align="center" gap="3" wrap="wrap" asChild>
              <span>
                <Text weight="medium">{strings.ERROR_TITLE}</Text>
                <Text>{error}</Text>
                <Button size="1" variant="soft" color="red" onClick={reload}>
                  <RefreshCw size={12} aria-hidden="true" />
                  {strings.RETRY}
                </Button>
              </span>
            </Flex>
          </Callout.Text>
        </Callout.Root>
      )}

      {!data && loading && <Box aria-label={strings.LOADING} role="status"><DashboardSkeleton /></Box>}

      {data && data.repositories.length === 0 && (
        <Card size="3" className="as-usage-dashboard__panel">
          <Flex align="center" justify="center" className="as-usage-dashboard__empty">
            <Text size="2" color="gray">{strings.NO_REPOSITORIES}</Text>
          </Flex>
        </Card>
      )}

      {data && data.repositories.length > 0 && (
        <div className={loading ? 'as-usage-dashboard__content as-usage-dashboard__content--loading'
          : 'as-usage-dashboard__content'}>
          <UsageKpiTiles data={data} />
          <ChartCard data={data} />
          <div className="as-usage-dashboard__row">
            <TopComponentsCard days={data.days} inPeriod={data.topComponents} allTime={data.topComponentsAllTime}
              canSearch={canSearch} />
            <RepositoriesCard byDownloads={data.topRepositoriesByDownloads}
              byStorage={data.topRepositoriesByStorage} />
          </div>
          <div className="as-usage-dashboard__row">
            <RecentComponentsCard components={data.recentComponents} />
            <UnusedCard unused={data.unused} unusedDays={data.unusedDays} canConfigureCleanup={canConfigureCleanup} />
          </div>
        </div>
      )}
    </Flex>
  );
}

export default UsageDashboard;
