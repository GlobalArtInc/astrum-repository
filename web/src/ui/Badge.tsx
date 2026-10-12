import { Badge } from '@radix-ui/themes';
import type { ReactNode } from 'react';
import type { RepositoryType } from '../api';
import { useT } from '../i18n';
import { formatColor, formatName } from './formats';

const TYPE_LABELS: Record<RepositoryType, string> = {
  hosted: 'ui.repositoryType.hosted',
  proxy: 'ui.repositoryType.proxy',
  group: 'ui.repositoryType.group',
};

const TYPE_COLORS = { hosted: 'blue', proxy: 'teal', group: 'amber' } as const;

export type Tone = 'neutral' | 'info' | 'success' | 'warning' | 'danger';

const TONE_COLORS = {
  neutral: 'gray',
  info: 'blue',
  success: 'green',
  warning: 'amber',
  danger: 'red',
} as const;

export function FormatBadge({ format }: { format: string }) {
  return (
    <Badge color={formatColor(format)} variant="soft" radius="full">
      {formatName(format)}
    </Badge>
  );
}

export function TypeBadge({ type }: { type: RepositoryType }) {
  const t = useT();
  return (
    <Badge color={TYPE_COLORS[type]} variant="outline" radius="full">
      {t(TYPE_LABELS[type])}
    </Badge>
  );
}

export function StatusBadge({ tone, children }: { tone: Tone; children: ReactNode }) {
  return (
    <Badge color={TONE_COLORS[tone]} variant="soft">
      {children}
    </Badge>
  );
}
