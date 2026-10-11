

import React from 'react';
import { Flex, Text } from '@radix-ui/themes';
import { Circle } from 'lucide-react';

import './StatusBadge.scss';
import SharedStrings from '../../constants/SharedStrings';

export type StatusType = 'online' | 'offline' | 'warning' | 'unknown' | 'success' | 'error' | 'info';

export interface StatusBadgeProps {
  /** Status type determining the color */
  status: StatusType;
  /** Optional label to display (defaults to capitalized status) */
  label?: string;
  /** Optional description text */
  description?: string;
  /** Optional reason/detail text (displayed on second line) */
  reason?: string;
  /** Size variant */
  size?: 'small' | 'medium';
  /** Custom class name */
  className?: string;
}

/**
 * StatusBadge displays a colored indicator with optional label and description.
 *
 * Features:
 * - Multiple status types: online, offline, warning, unknown, success, error, info
 * - Optional description and reason text
 * - Two size variants
 * - Accessible with aria-label
 *
 * @example
 * ```tsx
 * <StatusBadge status="online" />
 * <StatusBadge status="offline" label="Server Offline" description="Connection lost" />
 * <StatusBadge status="warning" label="Warning" reason="High memory usage" />
 * ```
 */
export function StatusBadge({
  status,
  label,
  description,
  reason,
  size = 'medium',
  className = '',
}: StatusBadgeProps): JSX.Element {
  // Default label to capitalized status
  const displayLabel = label ?? status.charAt(0).toUpperCase() + status.slice(1);

  // Map status to CSS class
  const statusClass = `status-badge--${status}`;
  const sizeClass = `status-badge--${size}`;

  return (
    <Flex
      direction="column"
      gap="1"
      className={`status-badge ${statusClass} ${sizeClass} ${className}`}
      aria-label={SharedStrings.STATUS_BADGE.STATUS(displayLabel)}
    >
      <Flex align="center" gap="2">
        <Circle
          size={size === 'small' ? 6 : 8}
          className="status-badge__indicator"
          aria-hidden="true"
        />
        <Text size={size === 'small' ? '1' : '2'} weight="medium">
          {displayLabel}
        </Text>
        {description && (
          <Text size={size === 'small' ? '1' : '2'} color="gray">
            - {description}
          </Text>
        )}
      </Flex>
      {reason && (
        <Text size="1" color="gray" className="status-badge__reason">
          {reason}
        </Text>
      )}
    </Flex>
  );
}

export default StatusBadge;


