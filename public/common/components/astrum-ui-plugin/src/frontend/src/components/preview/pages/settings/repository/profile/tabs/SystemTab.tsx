

import React from 'react';
import { Box, Flex, Text, Badge, Table } from '@radix-ui/themes';
import {
  Clock,
  Zap,
  Globe,
  CheckCircle,
  XCircle,
  AlertTriangle,
  Loader2,
} from 'lucide-react';
import { SettingsLink } from './settingsLink';
import type {
  RepositoryProfileData,
  TaskInfo,
  CapabilityInfo,
  HttpSettingsInfo,
} from '../types';
import RepositoryProfileStrings from '../../../../../constants/pages/settings/repository/RepositoryProfileStrings';
import CommonStrings from '../../../../../constants/CommonStrings';
import plural from '../../../../../../../interface/plural';
import RepositoriesPageStrings from '../../../../../constants/pages/settings/repository/RepositoriesPageStrings';

// =============================================================================
// Types
// =============================================================================

interface SystemTabProps {
  repository: RepositoryProfileData;
  tasks: TaskInfo[];
  capabilities: CapabilityInfo[];
  httpSettings: HttpSettingsInfo | null;
  loading?: boolean;
}

// =============================================================================
// Helper Components
// =============================================================================

interface ProfileSectionProps {
  title: string;
  icon: React.ElementType;
  editPath?: string;
  children: React.ReactNode;
}

function ProfileSection({ title, icon: Icon, editPath, children }: ProfileSectionProps): JSX.Element {
  return (
    <Box className="profile-section__card" mb="4">
      <Flex align="center" justify="between" mb="4" className="profile-section__header">
        <Flex align="center" gap="2">
          <Icon size={18} />
          <Text weight="bold">{title}</Text>
        </Flex>
        {editPath && (
          <SettingsLink previewPath={editPath} label={RepositoryProfileStrings.SYSTEM_TAB.VIEW} />
        )}
      </Flex>
      {children}
    </Box>
  );
}

// =============================================================================
// Helper Functions
// =============================================================================

function getStatusBadge(status: string): JSX.Element {
  switch (status?.toLowerCase()) {
    case 'ok':
    case 'success':
    case 'completed':
      return <Badge color="green" size="1"><CheckCircle size={12} /> {RepositoryProfileStrings.SYSTEM_TAB.SUCCESS}</Badge>;
    case 'running':
    case 'waiting':
      return <Badge color="blue" size="1"><Loader2 size={12} /> {RepositoryProfileStrings.SYSTEM_TAB.RUNNING}</Badge>;
    case 'failed':
    case 'error':
      return <Badge color="red" size="1"><XCircle size={12} /> {RepositoryProfileStrings.SYSTEM_TAB.FAILED}</Badge>;
    case 'blocked':
    case 'waiting_to_run':
      return <Badge color="yellow" size="1"><AlertTriangle size={12} /> {RepositoryProfileStrings.SYSTEM_TAB.WAITING}</Badge>;
    default:
      return <Badge color="gray" size="1">{status || '—'}</Badge>;
  }
}

function formatRelativeTime(dateString: string | undefined): string {
  if (!dateString) return '—';
  const date = new Date(dateString);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffMins = Math.floor(diffMs / (1000 * 60));
  const diffHours = Math.floor(diffMs / (1000 * 60 * 60));
  const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

  if (diffMins < 60) return plural(RepositoriesPageStrings.TASKS_CAPABILITIES.MINUTES_AGO, diffMins);
  if (diffHours < 24) return plural(RepositoriesPageStrings.TASKS_CAPABILITIES.HOURS_AGO, diffHours);
  return plural(RepositoriesPageStrings.TASKS_CAPABILITIES.DAYS_AGO, diffDays);
}

// =============================================================================
// Component
// =============================================================================

/**
 * SystemTab - Shows system-level settings affecting this repository
 *
 * Maps to Settings → System menu:
 * - Scheduled Tasks: Tasks that affect this repository
 * - Capabilities: Capabilities affecting this repo (Firewall, Health Check, etc.)
 * - HTTP Settings: Proxy, User Agent (if relevant for proxy repos)
 */
export function SystemTab({
  repository,
  tasks,
  capabilities,
  httpSettings,
  loading,
}: SystemTabProps): JSX.Element {
  if (loading) {
    return (
      <Box className="profile-empty-state">
        <Loader2 size={48} className="profile-empty-state__icon profile-empty-state__icon--spinning" />
        <Text className="profile-empty-state__title">{RepositoryProfileStrings.SYSTEM_TAB.LOADING_SYSTEM_DATA}</Text>
        <Text className="profile-empty-state__message">
          {RepositoryProfileStrings.SYSTEM_TAB.FETCHING_TASKS_CAPABILITIES_HTTP_SETTINGS}
        </Text>
      </Box>
    );
  }

  return (
    <Box>
      {/* Scheduled Tasks Section */}
      <ProfileSection
        title={RepositoryProfileStrings.SYSTEM_TAB.SCHEDULED_TASKS}
        icon={Clock}
        editPath="preview/admin/system/tasks"
      >
        {tasks.length > 0 ? (
          <Table.Root>
            <Table.Header>
              <Table.Row>
                <Table.ColumnHeaderCell>{RepositoryProfileStrings.SYSTEM_TAB.TASK}</Table.ColumnHeaderCell>
                <Table.ColumnHeaderCell>{RepositoryProfileStrings.SYSTEM_TAB.SCHEDULE}</Table.ColumnHeaderCell>
                <Table.ColumnHeaderCell>{RepositoryProfileStrings.SYSTEM_TAB.LAST_RUN}</Table.ColumnHeaderCell>
                <Table.ColumnHeaderCell>{CommonStrings.LABELS.STATUS}</Table.ColumnHeaderCell>
              </Table.Row>
            </Table.Header>
            <Table.Body>
              {tasks.map((task) => (
                <Table.Row key={task.id}>
                  <Table.Cell>
                    <Text weight="medium">{task.name}</Text>
                    <Text size="1" color="gray" as="div">{task.type}</Text>
                  </Table.Cell>
                  <Table.Cell>
                    <Text size="2">{task.schedule || RepositoryProfileStrings.SYSTEM_TAB.MANUAL}</Text>
                  </Table.Cell>
                  <Table.Cell>
                    <Text size="2">{formatRelativeTime(task.lastRun)}</Text>
                  </Table.Cell>
                  <Table.Cell>
                    {getStatusBadge(task.lastRunResult || '')}
                  </Table.Cell>
                </Table.Row>
              ))}
            </Table.Body>
          </Table.Root>
        ) : (
          <Text color="gray" size="2">
            {RepositoryProfileStrings.SYSTEM_TAB.NO_SCHEDULED_TASKS_FOUND_SPECIFICALLY}
          </Text>
        )}
      </ProfileSection>

      {/* Capabilities Section */}
      <ProfileSection
        title={RepositoryProfileStrings.SYSTEM_TAB.CAPABILITIES}
        icon={Zap}
        editPath="preview/admin/system/capabilities"
      >
        {capabilities.length > 0 ? (
          <Table.Root>
            <Table.Header>
              <Table.Row>
                <Table.ColumnHeaderCell>{RepositoryProfileStrings.SYSTEM_TAB.CAPABILITY}</Table.ColumnHeaderCell>
                <Table.ColumnHeaderCell>{CommonStrings.LABELS.STATUS}</Table.ColumnHeaderCell>
                <Table.ColumnHeaderCell>{RepositoryProfileStrings.SYSTEM_TAB.AFFECTS_REPO}</Table.ColumnHeaderCell>
              </Table.Row>
            </Table.Header>
            <Table.Body>
              {capabilities.map((capability) => (
                <Table.Row key={capability.id}>
                  <Table.Cell>
                    <Text weight="medium">{capability.type}</Text>
                    {capability.notes && (
                      <Text size="1" color="gray" as="div">{capability.notes}</Text>
                    )}
                  </Table.Cell>
                  <Table.Cell>
                    <Badge color={capability.enabled ? 'green' : 'gray'} size="1">
                      {capability.enabled ? RepositoryProfileStrings.SYSTEM_TAB.ACTIVE : RepositoryProfileStrings.SYSTEM_TAB.DISABLED}
                    </Badge>
                  </Table.Cell>
                  <Table.Cell>
                    {capability.affectsRepo ? (
                      <Flex align="center" gap="1">
                        <CheckCircle size={14} color="var(--green-11)" />
                        <Text size="2">{capability.affectsReason || RepositoryProfileStrings.SYSTEM_TAB.YES}</Text>
                      </Flex>
                    ) : (
                      <Text size="2" color="gray">{RepositoryProfileStrings.SYSTEM_TAB.NO}</Text>
                    )}
                  </Table.Cell>
                </Table.Row>
              ))}
            </Table.Body>
          </Table.Root>
        ) : (
          <Text color="gray" size="2">
            {RepositoryProfileStrings.SYSTEM_TAB.NO_CAPABILITIES_FOUND_AFFECT_REPOSITORY}
          </Text>
        )}
      </ProfileSection>

      {/* HTTP Settings Section (for proxy repositories) */}
      {repository.type === 'proxy' && (
        <ProfileSection
          title={RepositoryProfileStrings.SYSTEM_TAB.HTTP_SETTINGS}
          icon={Globe}
          editPath="preview/admin/system/http"
        >
          {httpSettings ? (
            <Flex direction="column" gap="2">
              <Box className="profile-section__row">
                <Text className="profile-section__label">{RepositoryProfileStrings.SYSTEM_TAB.USER_AGENT_SUFFIX}</Text>
                <Text className="profile-section__value">
                  {httpSettings.userAgentSuffix || '—'}
                </Text>
              </Box>
              <Box className="profile-section__row">
                <Text className="profile-section__label">{RepositoryProfileStrings.SYSTEM_TAB.HTTP_PROXY}</Text>
                <Text className="profile-section__value">
                  {httpSettings.httpProxy ? (
                    <code className="profile-section__value--code">
                      {httpSettings.httpProxy.host}:{httpSettings.httpProxy.port}
                    </code>
                  ) : (
                    RepositoryProfileStrings.SYSTEM_TAB.NOT_CONFIGURED
                  )}
                </Text>
              </Box>
              <Box className="profile-section__row">
                <Text className="profile-section__label">{RepositoryProfileStrings.SYSTEM_TAB.HTTPS_PROXY}</Text>
                <Text className="profile-section__value">
                  {httpSettings.httpsProxy ? (
                    <code className="profile-section__value--code">
                      {httpSettings.httpsProxy.host}:{httpSettings.httpsProxy.port}
                    </code>
                  ) : (
                    RepositoryProfileStrings.SYSTEM_TAB.NOT_CONFIGURED
                  )}
                </Text>
              </Box>
              {httpSettings.nonProxyHosts && httpSettings.nonProxyHosts.length > 0 && (
                <Box className="profile-section__row">
                  <Text className="profile-section__label">{RepositoryProfileStrings.SYSTEM_TAB.NON_PROXY_HOSTS}</Text>
                  <Text className="profile-section__value">
                    {httpSettings.nonProxyHosts.join(', ')}
                  </Text>
                </Box>
              )}
            </Flex>
          ) : (
            <Text color="gray" size="2">
              {RepositoryProfileStrings.SYSTEM_TAB.HTTP_SETTINGS_NOT_AVAILABLE_USING}
            </Text>
          )}
        </ProfileSection>
      )}
    </Box>
  );
}

export default SystemTab;


