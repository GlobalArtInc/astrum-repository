import React from 'react';
import { Card, Flex, Skeleton, Text } from '@radix-ui/themes';
import { Archive, Database, Download, File, Folder, Package } from 'lucide-react';
import UIStrings from '../../../../../constants/UIStrings';
import plural from '../../../../../interface/plural';
import { formatBytes, formatDays, formatNumber } from './usageFormat';
import type { UsageDashboardData } from './usageDashboard.types';

interface KpiTileProps {
  icon: React.ReactNode;
  label: string;
  value: React.ReactNode;
  caption?: React.ReactNode;
  tone?: 'orange';
}

function KpiTile({ icon, label, value, caption, tone }: KpiTileProps): JSX.Element {
  return (
    <Card size="2" className="as-usage-dashboard__kpi">
      <Flex direction="column" gap="1">
        <Text as="div" size="2" color={tone ?? 'gray'} className="as-usage-dashboard__kpi-label">
          <Flex align="center" gap="2" asChild>
            <span>{icon}{label}</span>
          </Flex>
        </Text>
        <div className="as-usage-dashboard__kpi-value">{value}</div>
        <Text as="div" size="1" color="gray" className="as-usage-dashboard__kpi-caption">{caption}</Text>
      </Flex>
    </Card>
  );
}

const ICON_SIZE = 15;

const KPI_NAMES = ['downloads', 'components', 'assets', 'storage', 'repositories', 'unused'];

/**
 * The headline figures of the period: downloads, components, files, storage, repositories and unused components.
 */
export function UsageKpiTiles({ data }: { data: UsageDashboardData }): JSX.Element {
  const strings = UIStrings.USAGE_DASHBOARD.KPI;
  const { totals, unused, days } = data;
  const average = totals.downloadsInPeriod / Math.max(1, days);
  return (
    <div className="as-usage-dashboard__kpis">
      <KpiTile
        icon={<Download size={ICON_SIZE} aria-hidden="true" />}
        label={strings.DOWNLOADS}
        value={formatNumber(totals.downloadsInPeriod)}
        caption={strings.DOWNLOADS_CAPTION(formatNumber(totals.downloadsToday),
          formatNumber(average, average < 10 ? 1 : 0))}
      />
      <KpiTile
        icon={<Package size={ICON_SIZE} aria-hidden="true" />}
        label={strings.COMPONENTS}
        value={formatNumber(totals.components)}
        caption={strings.COMPONENTS_CAPTION(formatNumber(totals.componentsDownloadedInPeriod))}
      />
      <KpiTile
        icon={<File size={ICON_SIZE} aria-hidden="true" />}
        label={strings.ASSETS}
        value={formatNumber(totals.assets)}
        caption={plural(strings.ASSETS_CAPTION, totals.repositories, formatNumber(totals.repositories))}
      />
      <KpiTile
        icon={<Database size={ICON_SIZE} aria-hidden="true" />}
        label={strings.STORAGE}
        value={formatBytes(totals.storageBytes)}
        caption={totals.proxy > 0 ? strings.STORAGE_CAPTION(formatBytes(totals.proxyStorageBytes)) : undefined}
      />
      <KpiTile
        icon={<Folder size={ICON_SIZE} aria-hidden="true" />}
        label={strings.REPOSITORIES}
        value={formatNumber(totals.repositories)}
        caption={strings.REPOSITORIES_CAPTION(formatNumber(totals.hosted), formatNumber(totals.proxy),
          formatNumber(totals.group))}
      />
      <KpiTile
        icon={<Archive size={ICON_SIZE} aria-hidden="true" />}
        label={strings.UNUSED(formatDays(data.unusedDays))}
        value={formatNumber(unused.components)}
        caption={strings.UNUSED_CAPTION(formatBytes(unused.bytes))}
        tone="orange"
      />
    </div>
  );
}

/** Placeholders of the tiles while the figures load. */
export function UsageKpiSkeletons(): JSX.Element {
  return (
    <div className="as-usage-dashboard__kpis">
      {KPI_NAMES.map((name) => (
        <Card size="2" key={name} className="as-usage-dashboard__kpi">
          <Flex direction="column" gap="2">
            <Skeleton width="60%" height="14px" />
            <Skeleton width="45%" height="30px" />
            <Skeleton width="75%" height="12px" />
          </Flex>
        </Card>
      ))}
    </div>
  );
}

export default UsageKpiTiles;
