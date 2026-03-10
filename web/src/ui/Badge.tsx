import { Badge } from '@radix-ui/themes';
import type { ReactNode } from 'react';
import type { RepositoryType } from '../api';
import { useT } from '../i18n';
import styles from './Badge.module.css';
import { formatColor, formatLabel, formatName } from './formats';

const TYPE_LABELS: Record<RepositoryType, string> = {
  hosted: 'ui.repositoryType.hosted',
  proxy: 'ui.repositoryType.proxy',
  group: 'ui.repositoryType.group',
};

export type Tone = 'neutral' | 'info' | 'success' | 'warning' | 'danger';

const TONE_COLORS = {
  neutral: 'gray',
  info: 'blue',
  success: 'green',
  warning: 'amber',
  danger: 'red',
} as const;

export function FormatBadge({ format, variant = 'id' }: { format: string; variant?: 'id' | 'name' }) {
  if (variant === 'name') {
    return (
      <Badge color="gray" variant="soft" radius="full">
        {formatName(format)}
      </Badge>
    );
  }
  return (
    <Badge color={formatColor(format)} variant="soft" radius="full">
      {formatLabel(format)}
    </Badge>
  );
}

export function TypeBadge({
  type,
  variant = 'pill',
}: {
  type: RepositoryType;
  variant?: 'pill' | 'outline';
}) {
  const t = useT();
  if (variant === 'outline') {
    return (
      <Badge color="gray" variant="outline" radius="full">
        {t(TYPE_LABELS[type])}
      </Badge>
    );
  }
  return (
    <Badge color="gray" variant="soft" radius="full" className={styles.lower}>
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
