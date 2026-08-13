

import React from 'react';
import { Badge } from '@radix-ui/themes';
import type { FormatBadgeProps } from '../browse.types';

/**
 * Color map for repository formats.
 */
type BadgeColor = 'blue' | 'green' | 'orange' | 'purple' | 'red' | 'gray' | 'cyan' | 'pink';

const formatColorMap: Record<string, BadgeColor> = {
  // Package managers
  maven2: 'orange',
  npm: 'red',
  nuget: 'purple',
  pypi: 'blue',
  rubygems: 'red',
  go: 'cyan',
  cargo: 'orange',
  composer: 'purple',
  cocoapods: 'orange',
  conan: 'blue',
  conda: 'green',
  r: 'blue',

  // Container/Cloud
  docker: 'blue',
  helm: 'blue',

  // OS packages
  apt: 'green',
  yum: 'green',
  raw: 'gray',

  // Other
  gitlfs: 'orange',
  p2: 'purple',
  terraform: 'purple',
  swift: 'orange',
  huggingface: 'pink',
};

/**
 * Display name overrides (e.g., "maven2" -> "maven").
 */
const formatDisplayMap: Record<string, string> = {
  maven2: 'maven',
  gitlfs: 'git-lfs',
  huggingface: 'hugging-face',
};

/**
 * Renders a colored badge for a repository format.
 *
 * @example
 * <FormatBadge format="maven2" />
 * <FormatBadge format="npm" />
 */
export function FormatBadge({ format }: FormatBadgeProps): JSX.Element {
  const lowerFormat = format.toLowerCase();
  const color = formatColorMap[lowerFormat] || 'gray';
  const displayName = formatDisplayMap[lowerFormat] || lowerFormat;

  return (
    <Badge color={color} variant="soft" size="1">
      {displayName}
    </Badge>
  );
}

export default FormatBadge;
