

import React from 'react';
import {
  Box,
  Flex,
  Text,
  Badge,
  Button,
  Card,
  Spinner,
  Table,
  Separator,
} from '@radix-ui/themes';
import {
  CheckCircle,
  Clock,
  XCircle,
  Zap,
} from 'lucide-react';
import ExtJS from '../../../../../../interface/ExtJS';
import { useRepositoryTasksCapabilities } from './useRepositoryTasksCapabilities';
import RepositoriesPageStrings from '../../../../constants/pages/settings/repository/RepositoriesPageStrings';
import CommonStrings from '../../../../constants/CommonStrings';
import plural from '../../../../../../interface/plural';

interface RepositoryTasksCapabilitiesTabProps {
  repositoryName: string;
}

// Maps a task's `lastRunResult` value to a badge. Do NOT feed `currentState`
// values (e.g. WAITING/RUNNING) in here — those belong to a different field
// with a different vocabulary.
function getLastRunResultBadge(lastRunResult: string | undefined): JSX.Element {
  switch (lastRunResult?.toLowerCase()) {
    case 'ok':
    case 'success':
    case 'completed':
      return (
        <Badge color="green" size="1">
          <CheckCircle size={12} /> {RepositoriesPageStrings.TASKS_CAPABILITIES.SUCCESS}
        </Badge>
      );
    case 'failed':
    case 'error':
    case 'errorrunning':
      return (
        <Badge color="red" size="1">
          <XCircle size={12} /> {RepositoriesPageStrings.TASKS_CAPABILITIES.FAILED}
        </Badge>
      );
    default:
      return (
        <Badge color="gray" size="1">
          {lastRunResult || '—'}
        </Badge>
      );
  }
}

function formatRelativeTime(dateString: string | undefined): string {
  if (!dateString) return '—';
  const date = new Date(dateString);
  const diffMs = Date.now() - date.getTime();
  if (diffMs < 0) return RepositoriesPageStrings.TASKS_CAPABILITIES.JUST_NOW;
  const diffMins = Math.floor(diffMs / (1000 * 60));
  if (diffMins < 1) return RepositoriesPageStrings.TASKS_CAPABILITIES.JUST_NOW;
  const diffHours = Math.floor(diffMs / (1000 * 60 * 60));
  const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));
  if (diffMins < 60) return plural(RepositoriesPageStrings.TASKS_CAPABILITIES.MINUTES_AGO, diffMins);
  if (diffHours < 24) return plural(RepositoriesPageStrings.TASKS_CAPABILITIES.HOURS_AGO, diffHours);
  return plural(RepositoriesPageStrings.TASKS_CAPABILITIES.DAYS_AGO, diffDays);
}

/**
 * RepositoryTasksCapabilitiesTab — Settings-page tab that surfaces
 * repo-scoped scheduled tasks and capabilities. Self-fetching; mounts only
 * when the tab is opened (Radix Tabs unmounts inactive content), so the
 * two REST calls do not block Settings-page first paint.
 *
 * The tab does NOT surface HTTP settings (instance-wide, not repo-scoped).
 * Each section is gated on its own read permission so a user with only
 * `astrum:tasks:read` won't see an empty capabilities table (and vice versa).
 */
export function RepositoryTasksCapabilitiesTab({
  repositoryName,
}: RepositoryTasksCapabilitiesTabProps): JSX.Element {
  const canReadTasks = ExtJS.checkPermission('astrum:tasks:read');
  const canReadCapabilities = ExtJS.checkPermission('astrum:capabilities:read');

  const { tasks, capabilities, loading, error, refetch } =
    useRepositoryTasksCapabilities(repositoryName, {
      canReadTasks,
      canReadCapabilities,
    });

  if (loading) {
    return (
      <Flex justify="center" align="center" py="9" data-testid="tasks-capabilities-loading">
        <Spinner size="3" />
      </Flex>
    );
  }

  if (error) {
    return (
      <Card size="2">
        <Flex direction="column" align="center" gap="3" py="6">
          <Text color="red" size="2">
            {RepositoriesPageStrings.TASKS_CAPABILITIES.FAILED_LOAD_TASKS_CAPABILITIES}
          </Text>
          <Text size="1" color="gray">
            {error}
          </Text>
          <Button variant="outline" size="2" onClick={refetch}>
            {CommonStrings.ACTIONS.RETRY}
          </Button>
        </Flex>
      </Card>
    );
  }

  return (
    <Flex direction="column" gap="4">
      {canReadTasks && (
        <Card size="2">
          <Flex direction="column" gap="3">
            <Flex align="center" gap="2">
              <Clock size={18} color="var(--accent-9)" />
              <Text size="3" weight="bold">
                {RepositoriesPageStrings.TASKS_CAPABILITIES.SCHEDULED_TASKS}
              </Text>
              <Badge variant="soft" color="gray" size="1">
                {plural(RepositoriesPageStrings.TASKS_CAPABILITIES.TASK_COUNT, tasks.length)}
              </Badge>
            </Flex>

            <Separator size="4" />

            {tasks.length > 0 ? (
              <Table.Root variant="surface" size="1">
                <Table.Header>
                  <Table.Row>
                    <Table.ColumnHeaderCell>{RepositoriesPageStrings.TASKS_CAPABILITIES.TASK_2}</Table.ColumnHeaderCell>
                    <Table.ColumnHeaderCell>{RepositoriesPageStrings.TASKS_CAPABILITIES.SCHEDULE}</Table.ColumnHeaderCell>
                    <Table.ColumnHeaderCell>{RepositoriesPageStrings.TASKS_CAPABILITIES.LAST_RUN}</Table.ColumnHeaderCell>
                    <Table.ColumnHeaderCell>{CommonStrings.LABELS.STATUS}</Table.ColumnHeaderCell>
                  </Table.Row>
                </Table.Header>
                <Table.Body>
                  {tasks.map((task) => (
                    <Table.Row key={task.id}>
                      <Table.Cell>
                        <Text size="1" weight="medium">
                          {task.name}
                        </Text>
                        <Text size="1" color="gray" as="div">
                          {task.type}
                        </Text>
                      </Table.Cell>
                      <Table.Cell>
                        <Text size="1">{task.schedule || RepositoriesPageStrings.TASKS_CAPABILITIES.MANUAL}</Text>
                      </Table.Cell>
                      <Table.Cell>
                        <Text size="1">{formatRelativeTime(task.lastRun)}</Text>
                      </Table.Cell>
                      <Table.Cell>{getLastRunResultBadge(task.lastRunResult)}</Table.Cell>
                    </Table.Row>
                  ))}
                </Table.Body>
              </Table.Root>
            ) : (
              <Box py="4">
                <Text size="2" color="gray">
                  {RepositoriesPageStrings.TASKS_CAPABILITIES.NO_SCHEDULED_TASKS_TARGET_REPOSITORY}
                </Text>
              </Box>
            )}
          </Flex>
        </Card>
      )}

      {canReadCapabilities && (
        <Card size="2">
          <Flex direction="column" gap="3">
            <Flex align="center" gap="2">
              <Zap size={18} color="var(--accent-9)" />
              <Text size="3" weight="bold">
                {RepositoriesPageStrings.TASKS_CAPABILITIES.CAPABILITIES}
              </Text>
              <Badge variant="soft" color="gray" size="1">
                {plural(RepositoriesPageStrings.TASKS_CAPABILITIES.CAPABILITY_COUNT, capabilities.length)}
              </Badge>
            </Flex>

            <Separator size="4" />

            {capabilities.length > 0 ? (
              <Table.Root variant="surface" size="1">
                <Table.Header>
                  <Table.Row>
                    <Table.ColumnHeaderCell>{RepositoriesPageStrings.TASKS_CAPABILITIES.CAPABILITY}</Table.ColumnHeaderCell>
                    <Table.ColumnHeaderCell>{CommonStrings.LABELS.STATUS}</Table.ColumnHeaderCell>
                    <Table.ColumnHeaderCell>{RepositoriesPageStrings.TASKS_CAPABILITIES.NOTES}</Table.ColumnHeaderCell>
                  </Table.Row>
                </Table.Header>
                <Table.Body>
                  {capabilities.map((capability) => (
                    <Table.Row key={capability.id}>
                      <Table.Cell>
                        <Text size="1" weight="medium">
                          {capability.type}
                        </Text>
                      </Table.Cell>
                      <Table.Cell>
                        <Badge color={capability.enabled ? 'green' : 'gray'} size="1">
                          {capability.enabled ? RepositoriesPageStrings.TASKS_CAPABILITIES.ACTIVE : RepositoriesPageStrings.TASKS_CAPABILITIES.DISABLED}
                        </Badge>
                      </Table.Cell>
                      <Table.Cell>
                        <Text size="1" color="gray">
                          {capability.notes || '—'}
                        </Text>
                      </Table.Cell>
                    </Table.Row>
                  ))}
                </Table.Body>
              </Table.Root>
            ) : (
              <Box py="4">
                <Text size="2" color="gray">
                  {RepositoriesPageStrings.TASKS_CAPABILITIES.NO_CAPABILITIES_SCOPED_REPOSITORY}
                </Text>
              </Box>
            )}
          </Flex>
        </Card>
      )}
    </Flex>
  );
}

export default RepositoryTasksCapabilitiesTab;
