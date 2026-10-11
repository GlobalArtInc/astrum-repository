

import React, { useMemo } from 'react';
import {
  Box,
  Flex,
  Text,
  Card,
  Badge,
  Button,
  Separator,
  Spinner,
  Table,
} from '@radix-ui/themes';
import { ExternalLink, ScrollText } from 'lucide-react';
import { useAuditLogApi } from '../../../../../../../utils/audit/useAuditLogApi';
import { formatAuditEvent, formatTimestamp } from '../../../../../../../utils/audit/auditEventFormatter';
import { CATEGORY_COLORS, CATEGORY_LABELS } from '../../../../../../../utils/audit/audit.constants';
import type { AuditFilters } from '../../../../../../../utils/audit/audit.types';
import RepositoriesPageStrings from '../../../../../constants/pages/settings/repository/RepositoriesPageStrings';
import plural from '../../../../../../../interface/plural';

interface RepositoryAuditTabProps {
  repositoryName: string;
}

export function RepositoryAuditTab({ repositoryName }: RepositoryAuditTabProps): JSX.Element {
  const filters: AuditFilters = useMemo(() => ({
    categories: [],
    domains: [],
    eventTypes: [],
    dateRange: 'last-30-days' as const,
    initiator: '',
    initiators: [],
    searchQuery: '',
    repositoryName,
  }), [repositoryName]);

  const { data, loading, error } = useAuditLogApi({ filters, page: 1, limit: 5 });

  const auditUrl = `#preview/browse/audit?repositoryName=${encodeURIComponent(repositoryName)}`;

  if (loading) {
    return (
      <Flex justify="center" align="center" py="9">
        <Spinner size="3" />
      </Flex>
    );
  }

  if (error) {
    return (
      <Card size="2">
        <Flex direction="column" align="center" gap="3" py="6">
          <Text color="red" size="2">{RepositoriesPageStrings.AUDIT.FAILED_LOAD_AUDIT_EVENTS}</Text>
          <Button variant="outline" size="2" asChild>
            <a href={auditUrl}>{RepositoriesPageStrings.AUDIT.OPEN_FULL_AUDIT_LOG} <ExternalLink size={14} /></a>
          </Button>
        </Flex>
      </Card>
    );
  }

  const totalEvents = data?.pagination.totalItems ?? 0;
  const recentEvents = data?.items ?? [];

  if (totalEvents === 0) {
    return (
      <Card size="2">
        <Flex direction="column" align="center" gap="3" py="8">
          <ScrollText size={40} color="var(--gray-6)" />
          <Text size="3" weight="medium" color="gray">{RepositoriesPageStrings.AUDIT.NO_AUDIT_EVENTS}</Text>
          <Text size="2" color="gray">{RepositoriesPageStrings.AUDIT.NO_AUDIT_ACTIVITY_RECORDED_REPOSITORY}</Text>
          <Button variant="outline" size="2" mt="2" asChild>
            <a href={auditUrl}>{RepositoriesPageStrings.AUDIT.OPEN_FULL_AUDIT_LOG} <ExternalLink size={14} /></a>
          </Button>
        </Flex>
      </Card>
    );
  }

  return (
    <Flex direction="column" gap="4">
      <Card size="2">
        <Flex direction="column" gap="3">
          <Flex justify="between" align="center">
            <Flex align="center" gap="2">
              <ScrollText size={18} color="var(--accent-9)" />
              <Text size="3" weight="bold">{RepositoriesPageStrings.AUDIT.RECENT_ACTIVITY}</Text>
              <Badge variant="soft" color="gray" size="1">
                {plural(RepositoriesPageStrings.AUDIT.EVENTS_30_DAYS, totalEvents, totalEvents.toLocaleString())}
              </Badge>
            </Flex>
            <Button variant="solid" size="2" asChild>
              <a href={auditUrl}>
                {RepositoriesPageStrings.AUDIT.VIEW_FULL_AUDIT_LOG} <ExternalLink size={14} />
              </a>
            </Button>
          </Flex>

          <Separator size="4" />

          <Table.Root variant="surface" size="1">
            <Table.Header>
              <Table.Row>
                <Table.ColumnHeaderCell>{RepositoriesPageStrings.AUDIT.TIME}</Table.ColumnHeaderCell>
                <Table.ColumnHeaderCell>{RepositoriesPageStrings.AUDIT.CATEGORY}</Table.ColumnHeaderCell>
                <Table.ColumnHeaderCell>{RepositoriesPageStrings.AUDIT.EVENT}</Table.ColumnHeaderCell>
                <Table.ColumnHeaderCell>{RepositoriesPageStrings.AUDIT.SUMMARY}</Table.ColumnHeaderCell>
                <Table.ColumnHeaderCell>{RepositoriesPageStrings.AUDIT.INITIATOR}</Table.ColumnHeaderCell>
              </Table.Row>
            </Table.Header>
            <Table.Body>
              {recentEvents.map((event) => {
                const display = formatAuditEvent(event);
                return (
                  <Table.Row key={event.id}>
                    <Table.Cell>
                      <Text size="1" color="gray">{formatTimestamp(event.timestamp)}</Text>
                    </Table.Cell>
                    <Table.Cell>
                      <Badge
                        color={CATEGORY_COLORS[display.category] as any}
                        variant="soft"
                        size="1"
                      >
                        {CATEGORY_LABELS[display.category]}
                      </Badge>
                    </Table.Cell>
                    <Table.Cell>
                      <Text size="1" weight="medium">{display.eventLabel}</Text>
                    </Table.Cell>
                    <Table.Cell>
                      <Text size="1">{display.summary}</Text>
                    </Table.Cell>
                    <Table.Cell>
                      <Text size="1" color="gray">{event.initiator || 'system'}</Text>
                    </Table.Cell>
                  </Table.Row>
                );
              })}
            </Table.Body>
          </Table.Root>

          {totalEvents > 5 && (
            <Box>
              <Text size="1" color="gray">
                {RepositoriesPageStrings.AUDIT.SHOWING_LATEST(
                  totalEvents.toLocaleString(),
                  <a href={auditUrl} style={{ color: 'var(--accent-11)' }}>
                    {RepositoriesPageStrings.AUDIT.VIEW_ALL_AUDIT_LOG}
                  </a>
                )}
              </Text>
            </Box>
          )}
        </Flex>
      </Card>
    </Flex>
  );
}

export default RepositoryAuditTab;
