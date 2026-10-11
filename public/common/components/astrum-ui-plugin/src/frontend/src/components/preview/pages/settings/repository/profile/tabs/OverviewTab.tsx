

import React from 'react';
import { Box, Flex, Text, Badge } from '@radix-ui/themes';
import {
  Database,
  HardDrive,
  Package,
  FileBox,
  Globe,
  CheckCircle,
  XCircle,
} from 'lucide-react';
import type {
  RepositoryProfileData,
  BlobStoreInfo,
  RepositoryMetrics,
} from '../types';
import { ensureTrailingSlash } from '../../../../../../../utils/url';
import RepositoryProfileStrings from '../../../../../constants/pages/settings/repository/RepositoryProfileStrings';
import CommonStrings from '../../../../../constants/CommonStrings';
import { pageLanguage } from '../../../../../../../interface/plural';

// =============================================================================
// Types
// =============================================================================

interface OverviewTabProps {
  repository: RepositoryProfileData;
  blobStore: BlobStoreInfo | null;
  metrics: RepositoryMetrics | null;
}

// =============================================================================
// Helper Functions
// =============================================================================

function formatBytes(bytes: number | undefined): string {
  if (bytes === undefined || bytes === null) return '—';
  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  let unitIndex = 0;
  let size = bytes;
  while (size >= 1024 && unitIndex < units.length - 1) {
    size /= 1024;
    unitIndex++;
  }
  return `${size.toFixed(1)} ${units[unitIndex]}`;
}

// =============================================================================
// Component
// =============================================================================

/**
 * OverviewTab - Displays repository details and storage metrics
 */
export function OverviewTab({
  repository,
  blobStore,
  metrics,
}: OverviewTabProps): JSX.Element {
  const storage = repository.attributes?.storage;
  const maven = repository.attributes?.maven;
  const proxy = repository.attributes?.proxy;
  const isOnline = repository.status?.online ?? repository.online ?? true;

  return (
    <Box>
      <Box className="profile-section__grid">
        {/* Repository Details */}
        <Box className="profile-section__card">
          <Flex align="center" gap="2" mb="4">
            <Database size={18} />
            <Text weight="bold">{RepositoryProfileStrings.OVERVIEW.REPOSITORY_DETAILS}</Text>
          </Flex>

          <Box className="profile-section__row">
            <Text className="profile-section__label">{CommonStrings.LABELS.TYPE}</Text>
            <Badge color={repository.type === 'hosted' ? 'blue' : repository.type === 'proxy' ? 'purple' : 'orange'}>
              {repository.type.charAt(0).toUpperCase() + repository.type.slice(1)}
            </Badge>
          </Box>

          <Box className="profile-section__row">
            <Text className="profile-section__label">{CommonStrings.LABELS.FORMAT}</Text>
            <Text className="profile-section__value">{repository.format}</Text>
          </Box>

          <Box className="profile-section__row">
            <Text className="profile-section__label">{CommonStrings.LABELS.STATUS}</Text>
            <Flex align="center" gap="1">
              {isOnline ? (
                <>
                  <CheckCircle size={14} color="var(--green-11)" />
                  <Text className="profile-section__value profile-section__value--success">{RepositoryProfileStrings.OVERVIEW.ONLINE}</Text>
                </>
              ) : (
                <>
                  <XCircle size={14} color="var(--red-11)" />
                  <Text className="profile-section__value profile-section__value--error">{RepositoryProfileStrings.OVERVIEW.OFFLINE}</Text>
                </>
              )}
            </Flex>
          </Box>

          <Box className="profile-section__row">
            <Text className="profile-section__label">{RepositoryProfileStrings.OVERVIEW.URL}</Text>
            <Text className="profile-section__value profile-section__value--code">
              {ensureTrailingSlash(repository.url)}
            </Text>
          </Box>

          {maven?.versionPolicy && (
            <Box className="profile-section__row">
              <Text className="profile-section__label">{RepositoryProfileStrings.OVERVIEW.VERSION_POLICY}</Text>
              <Text className="profile-section__value">{maven.versionPolicy}</Text>
            </Box>
          )}

          {maven?.layoutPolicy && (
            <Box className="profile-section__row">
              <Text className="profile-section__label">{RepositoryProfileStrings.OVERVIEW.LAYOUT_POLICY}</Text>
              <Text className="profile-section__value">{maven.layoutPolicy}</Text>
            </Box>
          )}

          {proxy?.remoteUrl && (
            <Box className="profile-section__row">
              <Text className="profile-section__label">
                <Globe size={14} />
                {RepositoryProfileStrings.OVERVIEW.REMOTE_URL}
              </Text>
              <Text className="profile-section__value profile-section__value--code">
                {proxy.remoteUrl}
              </Text>
            </Box>
          )}
        </Box>

        {/* Storage & Metrics */}
        <Box className="profile-section__card">
          <Flex align="center" gap="2" mb="4">
            <HardDrive size={18} />
            <Text weight="bold">{RepositoryProfileStrings.OVERVIEW.STORAGE_METRICS}</Text>
          </Flex>

          <Box className="profile-section__row">
            <Text className="profile-section__label">
              <Package size={14} />
              {RepositoryProfileStrings.OVERVIEW.COMPONENTS}
            </Text>
            <Text className="profile-section__value">
              {metrics?.componentCount?.toLocaleString(pageLanguage()) ?? '—'}
            </Text>
          </Box>

          <Box className="profile-section__row">
            <Text className="profile-section__label">
              <FileBox size={14} />
              {RepositoryProfileStrings.OVERVIEW.ASSETS}
            </Text>
            <Text className="profile-section__value">
              {metrics?.assetCount?.toLocaleString(pageLanguage()) ?? '—'}
            </Text>
          </Box>

          <Box className="profile-section__row">
            <Text className="profile-section__label">{RepositoryProfileStrings.OVERVIEW.BLOB_STORE}</Text>
            <Text className="profile-section__value">{storage?.blobStoreName ?? '—'}</Text>
          </Box>

          {blobStore && (
            <>
              <Box className="profile-section__row">
                <Text className="profile-section__label">{RepositoryProfileStrings.OVERVIEW.BLOB_STORE_TYPE}</Text>
                <Text className="profile-section__value">{blobStore.type}</Text>
              </Box>

              {blobStore.totalSizeInBytes !== undefined && (
                <Box className="profile-section__row">
                  <Text className="profile-section__label">{RepositoryProfileStrings.OVERVIEW.TOTAL_SIZE}</Text>
                  <Text className="profile-section__value">
                    {formatBytes(blobStore.totalSizeInBytes)}
                  </Text>
                </Box>
              )}

              {blobStore.availableSpaceInBytes !== undefined && (
                <Box className="profile-section__row">
                  <Text className="profile-section__label">{RepositoryProfileStrings.OVERVIEW.AVAILABLE_SPACE}</Text>
                  <Text className="profile-section__value">
                    {formatBytes(blobStore.availableSpaceInBytes)}
                  </Text>
                </Box>
              )}
            </>
          )}

          <Box className="profile-section__row">
            <Text className="profile-section__label">{RepositoryProfileStrings.OVERVIEW.STRICT_CONTENT_VALIDATION}</Text>
            <Text className="profile-section__value">
              {storage?.strictContentTypeValidation ? RepositoryProfileStrings.OVERVIEW.YES : RepositoryProfileStrings.OVERVIEW.NO}
            </Text>
          </Box>

          {storage?.writePolicy && (
            <Box className="profile-section__row">
              <Text className="profile-section__label">{RepositoryProfileStrings.OVERVIEW.WRITE_POLICY}</Text>
              <Text className="profile-section__value">{storage.writePolicy}</Text>
            </Box>
          )}
        </Box>
      </Box>
    </Box>
  );
}

export default OverviewTab;


