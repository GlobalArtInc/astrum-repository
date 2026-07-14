

import React from 'react';
import { Flex, Text } from '@radix-ui/themes';
import { FormatIcon, FORMAT_LABELS } from '@globalart/astrum-ui-plugin';

import './Badges.scss';

export interface FormatBadgeProps {
  /** Repository format (e.g., 'maven2', 'npm', 'docker') */
  format: string;
  /** Visual variant */
  variant?: 'inline' | 'tile';
  /** Icon size */
  size?: number;
  /** Whether to show the icon */
  showIcon?: boolean;
  /** Whether to show the text label */
  showLabel?: boolean;
  /** Label text color (e.g. 'blue' for link-style) */
  labelColor?: 'gray' | 'blue';
  /** Custom class name */
  className?: string;
}

/**
 * FormatBadge - A standardized badge for displaying technology formats (Maven, npm, etc.)
 * using official brand logos.
 */
export function FormatBadge({
  format,
  variant = 'inline',
  size = 16,
  showIcon = true,
  showLabel = true,
  labelColor,
  className = '',
}: FormatBadgeProps): JSX.Element {
  const displayLabel = FORMAT_LABELS[format] || format;

  if (variant === 'tile') {
    return (
      <Flex
        direction="column"
        align="center"
        justify="center"
        gap="2"
        className={`format-badge format-badge--tile ${className}`}
      >
        {showIcon && <FormatIcon format={format} size={size} />}
        {showLabel && (
          <Text size="1" weight="medium" color={labelColor} className="format-badge__label">
            {displayLabel}
          </Text>
        )}
      </Flex>
    );
  }

  return (
    <Flex
      align="center"
      gap="2"
      display="inline-flex"
      className={`format-badge format-badge--inline ${className}`}
    >
      {showIcon && <FormatIcon format={format} size={size} />}
      {showLabel && (
        <Text size="2" weight="medium" color={labelColor} className="format-badge__label">
          {displayLabel}
        </Text>
      )}
    </Flex>
  );
}

export default FormatBadge;
