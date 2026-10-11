

import React from 'react';
import { Box, Flex, Text, Skeleton } from '@radix-ui/themes';
import { Package, FileBox, AlertCircle } from 'lucide-react';
import { useRepositoryMetrics } from '../../../../shared';
import './RepositoryMetricsCard.scss';
import RepositoriesPageStrings from '../../../../constants/pages/settings/repository/RepositoriesPageStrings';

interface RepositoryMetricsCardProps {
  repositoryName: string;
}

export function RepositoryMetricsCard({ repositoryName }: RepositoryMetricsCardProps): JSX.Element | null {
  const { data, isLoading, error } = useRepositoryMetrics(repositoryName);

  if (isLoading) {
    return (
      <Box className="metrics-card">
        <Box className="metrics-card__skeleton">
          <Skeleton height="20px" width="60%" />
          <Skeleton height="32px" width="40%" />
          <Skeleton height="32px" width="50%" />
        </Box>
      </Box>
    );
  }

  if (error) {
    return (
      <Box className="metrics-card">
        <Box className="metrics-card__error">
          <Flex align="center" gap="2">
            <AlertCircle size={16} color="var(--red-11)" />
            <Text size="2" color="red">
              {RepositoriesPageStrings.METRICS.FAILED_LOAD_REPOSITORY_METRICS}
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
    <Box className="metrics-card">
      <Flex className="metrics-card__header">
        <Package size={18} />
        <Text weight="bold" size="3">
          {RepositoriesPageStrings.METRICS.REPOSITORY_METRICS}
        </Text>
      </Flex>

      <Box className="metrics-card__grid">
        <Box className="metrics-card__item">
          <Text className="metrics-card__label">
            <Flex align="center" gap="1">
              <Package size={14} />
              {RepositoriesPageStrings.METRICS.COMPONENTS}
            </Flex>
          </Text>
          <Text className="metrics-card__value">
            {data.componentCount.toLocaleString()}
          </Text>
        </Box>

        <Box className="metrics-card__item">
          <Text className="metrics-card__label">
            <Flex align="center" gap="1">
              <FileBox size={14} />
              {RepositoriesPageStrings.METRICS.ASSETS}
            </Flex>
          </Text>
          <Text className="metrics-card__value">
            {data.assetCount.toLocaleString()}
          </Text>
        </Box>
      </Box>
    </Box>
  );
}
