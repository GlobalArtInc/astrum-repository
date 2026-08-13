

import React from 'react';
import { Folder, Package, File } from 'lucide-react';
import type { NodeIconProps, NodeType } from '../browse.types';

/**
 * Icon component map by node type.
 */
const iconMap: Record<NodeType, React.ComponentType<{ size?: number; color?: string; className?: string }>> = {
  folder: Folder,
  component: Package,
  asset: File,
};

/**
 * Color map by node type.
 */
const colorMap: Record<NodeType, string> = {
  folder: 'var(--amber-9)',
  component: 'var(--blue-9)',
  asset: 'var(--gray-9)',
};

/**
 * Renders an icon for a browse tree node based on its type.
 *
 * @example
 * <NodeIcon type="folder" />
 * <NodeIcon type="component" size={20} />
 * <NodeIcon type="asset" className="my-icon" />
 */
export function NodeIcon({ type, size = 16, className }: NodeIconProps): JSX.Element {
  const Icon = iconMap[type];
  const color = colorMap[type];

  return <Icon size={size} color={color} className={className} />;
}

export default NodeIcon;
