import React, { useState } from 'react';
import { Badge, Box, Card, Flex, Heading, Link, SegmentedControl, Table, Text } from '@radix-ui/themes';
import { ArrowRight } from 'lucide-react';
import UIStrings from '../../../../../constants/UIStrings';
import { UsageBar } from './UsageBar';
import { componentName, formatDays, formatLabel, formatNumber, usageLinks } from './usageFormat';
import type { UsageComponent } from './usageDashboard.types';

export interface TopComponentsCardProps {
  days: number;
  inPeriod: UsageComponent[];
  allTime: UsageComponent[];
  canSearch: boolean;
}

/**
 * The components downloaded most, in the period or since downloads are counted, each linked to its node in the browse
 * tree.
 */
export function TopComponentsCard({ days, inPeriod, allTime, canSearch }: TopComponentsCardProps): JSX.Element {
  const strings = UIStrings.USAGE_DASHBOARD.TOP;
  const [scope, setScope] = useState<'period' | 'all'>('period');
  const components = scope === 'period' ? inPeriod : allTime;
  const most = components[0]?.downloads || 1;

  return (
    <Card size="3" className="as-usage-dashboard__panel as-usage-dashboard__wide" aria-label={strings.TITLE}>
      <Flex justify="between" align="center" gap="3" wrap="wrap" mb="3">
        <Heading as="h2" size="4">{strings.TITLE}</Heading>
        <SegmentedControl.Root size="1" value={scope} onValueChange={(value) => setScope(value as 'period' | 'all')}>
          <SegmentedControl.Item value="period">{strings.PERIOD(formatDays(days))}</SegmentedControl.Item>
          <SegmentedControl.Item value="all">{strings.ALL_TIME}</SegmentedControl.Item>
        </SegmentedControl.Root>
      </Flex>
      {components.length === 0 ? (
        <Flex align="center" justify="center" className="as-usage-dashboard__empty">
          <Text size="2" color="gray">{scope === 'period' ? strings.EMPTY : strings.EMPTY_ALL_TIME}</Text>
        </Flex>
      ) : (
        <Box className="as-usage-dashboard__table-scroll">
          <Table.Root size="1" variant="ghost" className="as-usage-dashboard__table">
            <Table.Header>
              <Table.Row>
                <Table.ColumnHeaderCell className="as-usage-dashboard__rank">{strings.RANK}</Table.ColumnHeaderCell>
                <Table.ColumnHeaderCell>{strings.COMPONENT}</Table.ColumnHeaderCell>
                <Table.ColumnHeaderCell>{strings.REPOSITORY}</Table.ColumnHeaderCell>
                <Table.ColumnHeaderCell justify="end">{strings.DOWNLOADS}</Table.ColumnHeaderCell>
              </Table.Row>
            </Table.Header>
            <Table.Body>
              {components.map((component, index) => (
                <Table.Row key={`${component.format}:${component.id}`} align="center">
                  <Table.Cell className="as-usage-dashboard__rank">
                    <Text size="2" color="gray">{index + 1}</Text>
                  </Table.Cell>
                  <Table.Cell className="as-usage-dashboard__component-cell">
                    <Link href={usageLinks.component(component)} color="gray" highContrast underline="hover"
                      weight="medium" size="2" title={componentName(component)}>
                      {componentName(component)}
                    </Link>
                    {component.version && <span className="as-usage-dashboard__mono">{component.version}</span>}
                  </Table.Cell>
                  <Table.Cell>
                    <Flex align="center" gap="2" wrap="nowrap">
                      <Text size="2" color="gray" className="as-usage-dashboard__nowrap">{component.repository}</Text>
                      <Badge color="gray" variant="soft" size="1">{formatLabel(component.format)}</Badge>
                    </Flex>
                  </Table.Cell>
                  <Table.Cell justify="end">
                    <Flex align="center" justify="end" gap="3">
                      <UsageBar share={component.downloads / most} className="as-usage-dashboard__inline-bar" />
                      <Text size="2" weight="medium" className="as-usage-dashboard__count">
                        {formatNumber(component.downloads)}
                      </Text>
                    </Flex>
                  </Table.Cell>
                </Table.Row>
              ))}
            </Table.Body>
          </Table.Root>
        </Box>
      )}
      {canSearch && (
        <Flex mt="3">
          <Link href={usageLinks.search()} size="2" className="as-usage-dashboard__search-link">
            <Flex align="center" gap="1" asChild>
              <span>{strings.SEARCH_LINK}<ArrowRight size={14} aria-hidden="true" /></span>
            </Flex>
          </Link>
        </Flex>
      )}
    </Card>
  );
}

export default TopComponentsCard;
