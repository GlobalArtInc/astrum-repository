

import React from 'react';
import { Box, Button, Flex, Table, Text } from '@radix-ui/themes';
import type { ArtifactSecurityItem } from './useArtifactList';

import './ArtifactTable.scss';
import SecurityReportStrings from '../../constants/SecurityReportStrings';
import CommonStrings from '../../constants/CommonStrings';

export interface ArtifactTableProps {
  items: ArtifactSecurityItem[];
  loading: boolean;
  error: string | null;
  hasMore: boolean;
  onLoadMore: () => Promise<void>;
}

export function ArtifactTable({
  items,
  loading,
  error,
  hasMore,
  onLoadMore,
}: ArtifactTableProps): JSX.Element {
  const [loadingMore, setLoadingMore] = React.useState(false);

  const handleLoadMore = async () => {
    setLoadingMore(true);
    try {
      await onLoadMore();
    } finally {
      setLoadingMore(false);
    }
  };

  if (loading && items.length === 0) {
    return (
      <Box className="artifact-table artifact-table--loading">
        <Text size="2" color="gray">
          {SecurityReportStrings.ARTIFACTS.LOADING_ARTIFACTS}
        </Text>
      </Box>
    );
  }

  if (error) {
    return (
      <Box className="artifact-table artifact-table--error">
        <Text size="2" color="red">
          {error}
        </Text>
      </Box>
    );
  }

  if (items.length === 0) {
    return (
      <Box className="artifact-table artifact-table--empty">
        <Text size="2" color="gray">
          {SecurityReportStrings.ARTIFACTS.NO_ARTIFACTS_FOUND}
        </Text>
      </Box>
    );
  }

  return (
    <Box className="artifact-table">
      <Table.Root variant="surface" size="1" className="artifact-table__table">
        <Table.Header>
          <Table.Row>
            <Table.ColumnHeaderCell>{SecurityReportStrings.ARTIFACTS.GROUP}</Table.ColumnHeaderCell>
            <Table.ColumnHeaderCell>{CommonStrings.LABELS.NAME}</Table.ColumnHeaderCell>
            <Table.ColumnHeaderCell>{SecurityReportStrings.ARTIFACTS.VERSION}</Table.ColumnHeaderCell>
            <Table.ColumnHeaderCell>{CommonStrings.LABELS.FORMAT}</Table.ColumnHeaderCell>
            <Table.ColumnHeaderCell className="artifact-table__severity">{SecurityReportStrings.ARTIFACTS.CRITICAL}</Table.ColumnHeaderCell>
            <Table.ColumnHeaderCell className="artifact-table__severity">{SecurityReportStrings.ARTIFACTS.SEVERE}</Table.ColumnHeaderCell>
            <Table.ColumnHeaderCell className="artifact-table__severity">{SecurityReportStrings.ARTIFACTS.MODERATE}</Table.ColumnHeaderCell>
            <Table.ColumnHeaderCell>{SecurityReportStrings.ARTIFACTS.LICENSE}</Table.ColumnHeaderCell>
          </Table.Row>
        </Table.Header>
        <Table.Body>
          {items.map((item, idx) => (
            <Table.Row key={item.id ?? `${item.name}-${item.version}-${idx}`}>
              <Table.Cell>
                <Text size="2">{item.group ?? '—'}</Text>
              </Table.Cell>
              <Table.Cell>
                <Text size="2" weight="medium">
                  {item.name ?? '—'}
                </Text>
              </Table.Cell>
              <Table.Cell>
                <Text size="2">{item.version ?? '—'}</Text>
              </Table.Cell>
              <Table.Cell>
                <Text size="2">{item.format ?? '—'}</Text>
              </Table.Cell>
              <Table.Cell className="artifact-table__severity">
                <SeverityBadge value={item.criticalCount} color="red" />
              </Table.Cell>
              <Table.Cell className="artifact-table__severity">
                <SeverityBadge value={item.severeCount} color="orange" />
              </Table.Cell>
              <Table.Cell className="artifact-table__severity">
                <SeverityBadge value={item.moderateCount} color="yellow" />
              </Table.Cell>
              <Table.Cell>
                <Text size="2" color="gray">
                  {item.licenseThreatName ?? (item.licenseThreatLevel != null ? SecurityReportStrings.ARTIFACTS.LEVEL(item.licenseThreatLevel) : '—')}
                </Text>
              </Table.Cell>
            </Table.Row>
          ))}
        </Table.Body>
      </Table.Root>
      {hasMore && (
        <Flex justify="center" mt="3">
          <Button
            variant="soft"
            disabled={loadingMore}
            onClick={handleLoadMore}
          >
            {loadingMore ? SecurityReportStrings.ARTIFACTS.LOADING : SecurityReportStrings.ARTIFACTS.LOAD_MORE}
          </Button>
        </Flex>
      )}
    </Box>
  );
}

function SeverityBadge({
  value,
  color,
}: {
  value?: number;
  color: 'red' | 'orange' | 'yellow';
}): JSX.Element {
  const n = value ?? 0;
  const cssVar = `var(--${color}-9)`;
  return (
    <Text size="2" weight="medium" style={{ color: n > 0 ? cssVar : 'var(--gray-9)' }}>
      {n}
    </Text>
  );
}
