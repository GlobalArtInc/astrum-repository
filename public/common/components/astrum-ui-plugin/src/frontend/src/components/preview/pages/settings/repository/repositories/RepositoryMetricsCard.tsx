

import React from 'react';
import { Box, Flex, Text, Skeleton } from '@radix-ui/themes';
import { Package, FileBox, AlertCircle } from 'lucide-react';
import { useRepositoryMetrics } from '../../../../shared';
import './RepositoryMetricsCard.scss';

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
              Failed to load repository metrics
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
          Repository Metrics
        </Text>
      </Flex>

      <Box className="metrics-card__grid">
        <Box className="metrics-card__item">
          <Text className="metrics-card__label">
            <Flex align="center" gap="1">
              <Package size={14} />
              Components
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
              Assets
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
