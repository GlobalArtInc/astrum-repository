

import React from 'react';
import { Flex, Text } from '@radix-ui/themes';
import { Circle } from 'lucide-react';
import type { RepositoryStatusBadgeProps } from './repository-list.types';

import './RepositoryStatusBadge.scss';

/**
 * RepositoryStatusBadge displays the online/offline status of a repository.
 *
 * Features:
 * - Green indicator for online repositories
 * - Red indicator for offline repositories
 * - Optional description and reason text
 */
export function RepositoryStatusBadge({ status }: RepositoryStatusBadgeProps): JSX.Element {
  const isOnline = status?.online ?? false;
  const statusText = isOnline ? 'Online' : 'Offline';
  const statusClass = isOnline ? 'as-status-badge--online' : 'as-status-badge--offline';

  return (
    <Flex direction="column" gap="1" className="as-status-badge">
      <Flex align="center" gap="2">
        <Circle
          size={8}
          className={`as-status-badge__indicator ${statusClass}`}
          aria-hidden="true"
        />
        <Text size="2" weight="medium">
          {statusText}
        </Text>
        {status?.description && (
          <Text size="2" color="gray">
            - {status.description}
          </Text>
        )}
      </Flex>
      {status?.reason && (
        <Text size="1" color="gray" className="as-status-badge__reason">
          {status.reason}
        </Text>
      )}
    </Flex>
  );
}

export default RepositoryStatusBadge;

