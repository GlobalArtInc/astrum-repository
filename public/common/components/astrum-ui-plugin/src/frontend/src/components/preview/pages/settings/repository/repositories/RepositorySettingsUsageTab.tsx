

import React from 'react';
import { Box, Flex, Text, Card, Grid, Tooltip } from '@radix-ui/themes';
import { HardDrive, Package, FileBox, FolderTree } from 'lucide-react';

import { LoadingState, ErrorState } from '../../../../shared';
import HumanReadableUtils from '../../../../../../interface/HumanReadableUtils';
import UIStrings from '../../../../../../constants/UIStrings';
import { useRepositorySettingsUsageTab } from './useRepositorySettingsUsageTab';
import type { RepositoryUsageKind } from './repositoryUsageMachine';

import './RepositorySettingsUsageTab.scss';
import RepositoriesPageStrings from '../../../../constants/pages/settings/repository/RepositoriesPageStrings';
import CommonStrings from '../../../../constants/CommonStrings';

export interface RepositorySettingsUsageTabProps {
  repositoryName: string;
  repositoryType: RepositoryUsageKind;
}

function formatBytes(bytes: number | undefined | null): string {
  if (bytes === undefined || bytes === null || Number.isNaN(bytes)) return UIStrings.UNAVAILABLE;
  return HumanReadableUtils.bytesToString(bytes);
}

function formatNumber(value: number | undefined | null): string {
  if (value === undefined || value === null || Number.isNaN(value)) return UIStrings.UNAVAILABLE;
  return value.toLocaleString();
}

interface MembershipSectionProps {
  title: string;
  items: string[];
  emptyMessage: string;
  error: string | null;
}

function MembershipSection({ title, items, emptyMessage, error }: MembershipSectionProps): JSX.Element {
  const showPlaceholder = error !== null || items.length === 0;
  const placeholderMessage = error ?? emptyMessage;

  return (
    <Box>
      <Text size="3" weight="bold" as="div" mb="3">
        {title}
      </Text>
      {showPlaceholder ? (
        <Flex
          direction="column"
          align="center"
          justify="center"
          p="6"
          className="repository-settings-usage-tab__membership-placeholder"
        >
          <Text color="gray" size="2">{placeholderMessage}</Text>
        </Flex>
      ) : (
        <Flex direction="column" gap="2">
          {items.map((name) => (
            <Flex
              key={name}
              align="center"
              gap="2"
              py="2"
              px="3"
              className="repository-settings-usage-tab__membership-item"
            >
              <FolderTree size={16} color="var(--blue-9)" />
              <Text weight="medium">{name}</Text>
            </Flex>
          ))}
        </Flex>
      )}
    </Box>
  );
}

export function RepositorySettingsUsageTab({
  repositoryName,
  repositoryType,
}: RepositorySettingsUsageTabProps): JSX.Element {
  const {
    metrics,
    groupMembers,
    whereUsed,
    loading,
    loaded,
    error,
    membershipError,
    componentCountPending,
    assetCountPending,
    totalSizePending,
    retry,
  } = useRepositorySettingsUsageTab(repositoryName, repositoryType);

  if (loading && !loaded) {
    return <LoadingState message={RepositoriesPageStrings.SETTINGS_USAGE.LOADING_USAGE_DATA} />;
  }

  if (error && !loaded) {
    return (
      <ErrorState
        message={error}
        retryText={CommonStrings.ACTIONS.RETRY}
        onRetry={retry}
      />
    );
  }

  const showGroupMembers = repositoryType === 'group';

  return (
    <Flex direction="column" gap="5" p="4">
      <Box>
        <Text size="3" weight="bold" as="div" mb="3">
          {RepositoriesPageStrings.SETTINGS_USAGE.STORAGE_METRICS}
        </Text>
        <Grid columns="3" gap="4">
          <Card size="2">
            <Flex direction="column" gap="2" align="center" py="2">
              <HardDrive size={22} color="var(--accent-9)" />
              <Tooltip content={totalSizePending ? RepositoriesPageStrings.SETTINGS_USAGE.REPOSITORY_SIZE_NOT_YET_CALCULATED : RepositoriesPageStrings.SETTINGS_USAGE.TOTAL_SIZE_ALL_ASSETS_REPOSITORY}>
                <Text size="5" weight="bold">
                  {formatBytes(metrics?.totalSize)}
                </Text>
              </Tooltip>
              <Text size="1" color="gray">{RepositoriesPageStrings.SETTINGS_USAGE.REPOSITORY_SIZE}</Text>
            </Flex>
          </Card>

          <Card size="2">
            <Flex direction="column" gap="2" align="center" py="2">
              <Package size={22} color="var(--accent-9)" />
              <Tooltip content={componentCountPending ? RepositoriesPageStrings.SETTINGS_USAGE.COMPONENT_COUNT_NOT_YET_CALCULATED : undefined}>
                <Text size="5" weight="bold">
                  {formatNumber(metrics?.componentCount)}
                </Text>
              </Tooltip>
              <Text size="1" color="gray">{RepositoriesPageStrings.SETTINGS_USAGE.COMPONENTS}</Text>
            </Flex>
          </Card>

          <Card size="2">
            <Flex direction="column" gap="2" align="center" py="2">
              <FileBox size={22} color="var(--accent-9)" />
              <Tooltip content={assetCountPending ? RepositoriesPageStrings.SETTINGS_USAGE.ASSET_COUNT_NOT_YET_CALCULATED : undefined}>
                <Text size="5" weight="bold">
                  {formatNumber(metrics?.assetCount)}
                </Text>
              </Tooltip>
              <Text size="1" color="gray">{RepositoriesPageStrings.SETTINGS_USAGE.ASSETS}</Text>
            </Flex>
          </Card>
        </Grid>
      </Box>

      {showGroupMembers ? (
        <MembershipSection
          title={RepositoriesPageStrings.SETTINGS_USAGE.MEMBER_REPOSITORIES}
          items={groupMembers}
          emptyMessage={RepositoriesPageStrings.SETTINGS_USAGE.GROUP_HAS_NO_MEMBER_REPOSITORIES}
          error={membershipError}
        />
      ) : (
        <MembershipSection
          title={RepositoriesPageStrings.SETTINGS_USAGE.GROUP_MEMBERSHIP}
          items={whereUsed}
          emptyMessage={RepositoriesPageStrings.SETTINGS_USAGE.REPOSITORY_NOT_MEMBER_ANY_GROUPS}
          error={membershipError}
        />
      )}
    </Flex>
  );
}

export default RepositorySettingsUsageTab;
