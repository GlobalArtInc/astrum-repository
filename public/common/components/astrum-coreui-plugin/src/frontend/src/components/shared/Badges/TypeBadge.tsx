

import React from 'react';
import { Badge, Flex } from '@radix-ui/themes';
import { Cloud, Database, FolderSync, LucideIcon } from 'lucide-react';
import { TYPE_LABELS } from '@globalart/astrum-ui-plugin';

import './Badges.scss';

export interface TypeBadgeProps {
  /** Repository type (hosted, proxy, group) */
  type: 'hosted' | 'proxy' | 'group';
  /** Custom class name */
  className?: string;
  /** Whether to show the icon */
  showIcon?: boolean;
}

const TYPE_CONFIG: Record<string, { color: any; icon: LucideIcon }> = {
  proxy: { color: 'blue', icon: Cloud },
  hosted: { color: 'green', icon: Database },
  group: { color: 'purple', icon: FolderSync },
};

/**
 * TypeBadge - A standardized semantic badge for repository types.
 * Blue = Proxy, Green = Hosted, Purple = Group.
 */
export function TypeBadge({
  type,
  className = '',
  showIcon = true,
}: TypeBadgeProps): JSX.Element {
  const config = TYPE_CONFIG[type] || { color: 'gray', icon: Database };
  const Icon = config.icon;
  const label = TYPE_LABELS[type] || type;

  return (
    <Badge
      color={config.color}
      variant="soft"
      className={`type-badge type-badge--${type} ${className}`}
    >
      <Flex align="center" gap="1">
        {showIcon && <Icon size={12} />}
        {label}
      </Flex>
    </Badge>
  );
}

export default TypeBadge;
