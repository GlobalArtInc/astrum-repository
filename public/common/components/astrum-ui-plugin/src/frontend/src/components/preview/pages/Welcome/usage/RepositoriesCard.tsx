import React, { useState } from 'react';
import { Badge, Card, Flex, Heading, Link, SegmentedControl, Text } from '@radix-ui/themes';
import UIStrings from '../../../../../constants/UIStrings';
import { UsageBar } from './UsageBar';
import { formatBytes, formatLabel, formatNumber, usageLinks } from './usageFormat';
import type { RepositoryUsage, UsageDashboardVariant } from './usageDashboard.types';

export interface RepositoriesCardProps {
  byDownloads: RepositoryUsage[];
  byStorage: RepositoryUsage[];
  variant: UsageDashboardVariant;
}

/**
 * The repositories downloaded from most in the period, or taking up most storage.
 */
export function RepositoriesCard({ byDownloads, byStorage, variant }: RepositoriesCardProps): JSX.Element {
  const strings = UIStrings.USAGE_DASHBOARD.REPOSITORIES;
  const [order, setOrder] = useState<'downloads' | 'size'>('downloads');
  const repositories = order === 'downloads' ? byDownloads : byStorage;
  const measure = (repository: RepositoryUsage) =>
    order === 'downloads' ? repository.downloads : repository.storageBytes;
  const most = repositories.length ? measure(repositories[0]) || 1 : 1;

  return (
    <Card size="3" className="as-usage-dashboard__panel as-usage-dashboard__narrow" aria-label={strings.TITLE}>
      <Flex justify="between" align="center" gap="3" wrap="wrap" mb="4">
        <Heading as="h2" size="4">{strings.TITLE}</Heading>
        <SegmentedControl.Root size="1" value={order} aria-label={strings.SORT_LABEL}
          onValueChange={(value) => setOrder(value as 'downloads' | 'size')}>
          <SegmentedControl.Item value="downloads">{strings.BY_DOWNLOADS}</SegmentedControl.Item>
          <SegmentedControl.Item value="size">{strings.BY_SIZE}</SegmentedControl.Item>
        </SegmentedControl.Root>
      </Flex>
      {repositories.length === 0 ? (
        <Flex align="center" justify="center" className="as-usage-dashboard__empty">
          <Text size="2" color="gray">{order === 'downloads' ? strings.EMPTY_DOWNLOADS : strings.EMPTY_SIZE}</Text>
        </Flex>
      ) : (
        <Flex direction="column" gap="4" asChild>
          <ul className="as-usage-dashboard__list">
            {repositories.map((repository) => (
              <li key={repository.name}>
                <Flex direction="column" gap="2">
                  <Flex justify="between" align="center" gap="2">
                    <Flex align="center" gap="2" minWidth="0">
                      <Link href={usageLinks.repository(variant, repository.name)} color="gray" highContrast
                        underline="hover" weight="medium" size="2" truncate title={repository.name}>
                        {repository.name}
                      </Link>
                      <Badge color="gray" variant="soft" size="1">{formatLabel(repository.format)}</Badge>
                    </Flex>
                    <Text size="2" weight="medium" className="as-usage-dashboard__count">
                      {order === 'downloads' ? formatNumber(repository.downloads) : formatBytes(repository.storageBytes)}
                    </Text>
                  </Flex>
                  <UsageBar share={measure(repository) / most} />
                </Flex>
              </li>
            ))}
          </ul>
        </Flex>
      )}
    </Card>
  );
}

export default RepositoriesCard;
