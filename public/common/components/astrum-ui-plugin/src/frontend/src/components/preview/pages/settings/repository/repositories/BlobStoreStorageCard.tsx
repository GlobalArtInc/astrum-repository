

import React from 'react';
import { Box, Flex, Text, Skeleton } from '@radix-ui/themes';
import { HardDrive, AlertCircle, CheckCircle } from 'lucide-react';
import { useBlobStoreInfo } from '../../../../shared';
import './BlobStoreStorageCard.scss';
import RepositoriesPageStrings from '../../../../constants/pages/settings/repository/RepositoriesPageStrings';
import CommonStrings from '../../../../constants/CommonStrings';
import { pageLanguage } from '../../../../../../interface/plural';

interface BlobStoreStorageCardProps {
  blobStoreName: string;
}

function formatBytes(bytes: number | undefined): string {
  if (bytes === undefined) return '—';
  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  let unitIndex = 0;
  let size = bytes;
  while (size >= 1024 && unitIndex < units.length - 1) {
    size /= 1024;
    unitIndex++;
  }
  return `${size.toFixed(1)} ${units[unitIndex]}`;
}

export function BlobStoreStorageCard({ blobStoreName }: BlobStoreStorageCardProps): JSX.Element | null {
  const { data, isLoading, error } = useBlobStoreInfo(blobStoreName);

  if (isLoading) {
    return (
      <Box className="blob-store-card">
        <Box className="blob-store-card__skeleton">
          <Skeleton height="20px" width="60%" />
          <Skeleton height="16px" width="40%" />
          <Skeleton height="16px" width="80%" />
          <Skeleton height="16px" width="50%" />
        </Box>
      </Box>
    );
  }

  if (error) {
    return (
      <Box className="blob-store-card">
        <Box className="blob-store-card__error">
          <Flex align="center" gap="2">
            <AlertCircle size={16} color="var(--red-11)" />
            <Text size="2" color="red">
              {RepositoriesPageStrings.BLOB_STORE_CARD.FAILED_LOAD_BLOB_STORE_INFORMATION}
            </Text>
          </Flex>
        </Box>
      </Box>
    );
  }

  if (!data) {
    return null;
  }

  return (
    <Box className="blob-store-card">
      <Flex className="blob-store-card__header">
        <HardDrive size={18} />
        <Text weight="bold" size="3">
          {RepositoriesPageStrings.BLOB_STORE_CARD.BLOB_STORE}
        </Text>
      </Flex>

      <Box className="blob-store-card__grid">
        <Box className="blob-store-card__item">
          <Text className="blob-store-card__label">{CommonStrings.LABELS.NAME}</Text>
          <Text className="blob-store-card__value">{data.name}</Text>
        </Box>

        <Box className="blob-store-card__item">
          <Text className="blob-store-card__label">{CommonStrings.LABELS.TYPE}</Text>
          <Text className="blob-store-card__value">{data.type}</Text>
        </Box>

        {data.totalSizeInBytes !== undefined && (
          <Box className="blob-store-card__item">
            <Text className="blob-store-card__label">{RepositoriesPageStrings.BLOB_STORE_CARD.TOTAL_SIZE}</Text>
            <Text className="blob-store-card__value">{formatBytes(data.totalSizeInBytes)}</Text>
          </Box>
        )}

        {data.availableSpaceInBytes !== undefined && (
          <Box className="blob-store-card__item">
            <Text className="blob-store-card__label">{RepositoriesPageStrings.BLOB_STORE_CARD.AVAILABLE_SPACE}</Text>
            <Text className="blob-store-card__value">{formatBytes(data.availableSpaceInBytes)}</Text>
          </Box>
        )}

        {data.blobCount !== undefined && (
          <Box className="blob-store-card__item">
            <Text className="blob-store-card__label">{RepositoriesPageStrings.BLOB_STORE_CARD.BLOB_COUNT}</Text>
            <Text className="blob-store-card__value">{data.blobCount.toLocaleString(pageLanguage())}</Text>
          </Box>
        )}

        <Box className="blob-store-card__item">
          <Text className="blob-store-card__label">{CommonStrings.LABELS.STATUS}</Text>
          <Flex className="blob-store-card__status">
            {data.unavailable ? (
              <>
                <AlertCircle size={14} color="var(--red-11)" />
                <Text size="2" color="red">
                  {RepositoriesPageStrings.BLOB_STORE_CARD.UNAVAILABLE}
                </Text>
              </>
            ) : (
              <>
                <CheckCircle size={14} color="var(--green-11)" />
                <Text size="2" color="green">
                  {RepositoriesPageStrings.BLOB_STORE_CARD.AVAILABLE}
                </Text>
              </>
            )}
          </Flex>
        </Box>
      </Box>
    </Box>
  );
}
