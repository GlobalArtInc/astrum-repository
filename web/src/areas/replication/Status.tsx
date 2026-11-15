import { Skeleton, Text } from '@radix-ui/themes';
import type { ReactNode } from 'react';
import { useT } from '../../i18n';
import { StatusBadge, type Tone } from '../../ui';
import type { ReplicationStatus } from './api';

export interface StatusResult {
  data: ReplicationStatus | undefined;
  isPending: boolean;
}

const TONES: Record<string, Tone> = {
  ok: 'success',
  failing: 'warning',
  dead_items: 'danger',
};

const LABELS: Record<string, string> = {
  ok: 'replication.status.ok',
  failing: 'replication.status.failing',
  dead_items: 'replication.status.deadItems',
};

interface StatusValueProps {
  result: StatusResult | undefined;
  children: (status: ReplicationStatus) => ReactNode;
}

export function StatusValue({ result, children }: StatusValueProps) {
  if (result?.isPending) {
    return <Skeleton width="48px" height="16px" />;
  }
  if (result?.data === undefined) {
    return (
      <Text size="2" color="gray">
        —
      </Text>
    );
  }
  return <>{children(result.data)}</>;
}

interface ReplicationStateProps {
  enabled: boolean;
  result: StatusResult | undefined;
}

export function ReplicationState({ enabled, result }: ReplicationStateProps) {
  const t = useT();
  if (!enabled) {
    return <StatusBadge tone="neutral">{t('replication.replications.disabled')}</StatusBadge>;
  }
  return (
    <StatusValue result={result}>
      {(status) => {
        const label = LABELS[status.alertState];
        return (
          <StatusBadge tone={TONES[status.alertState] ?? 'neutral'}>
            {label === undefined ? status.alertState : t(label)}
          </StatusBadge>
        );
      }}
    </StatusValue>
  );
}
