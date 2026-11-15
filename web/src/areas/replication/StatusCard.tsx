import { Callout, Card, Flex, Grid, Text } from '@radix-ui/themes';
import type { ReactNode } from 'react';
import { formatDateTime, useI18n, useT } from '../../i18n';
import { RelativeTime } from '../../ui';
import type { ReplicationStatus } from './api';
import { isSuspended, lagText } from './model';
import { ReplicationState } from './Status';

interface Field {
  label: string;
  value: ReactNode;
}

interface StatusCardProps {
  status: ReplicationStatus;
}

export function StatusCard({ status }: StatusCardProps) {
  const t = useT();
  const { language } = useI18n();
  const never = <Text color="gray">{t('replication.status.never')}</Text>;
  const suspended =
    status.remote.suspendedUntil !== null && isSuspended(status.remote.suspendedUntil, new Date());
  const fields: Field[] = [
    { label: t('replication.replications.pending'), value: status.pending },
    { label: t('replication.detail.leased'), value: status.leased },
    { label: t('replication.replications.dead'), value: status.dead },
    { label: t('replication.replications.lag'), value: lagText(t, status.lagSeconds) },
    {
      label: t('replication.replications.lastSuccess'),
      value: status.lastSuccessAt === null ? never : <RelativeTime value={status.lastSuccessAt} />,
    },
    {
      label: t('replication.detail.lastFailure'),
      value: status.lastFailureAt === null ? never : <RelativeTime value={status.lastFailureAt} />,
    },
    { label: t('replication.detail.lastError'), value: status.lastError ?? '—' },
    {
      label: t('replication.detail.lastReconcile'),
      value: status.lastReconcileAt === null ? never : <RelativeTime value={status.lastReconcileAt} />,
    },
    { label: t('replication.detail.remoteFailures'), value: status.remote.failures },
    { label: t('replication.detail.remoteError'), value: status.remote.lastError ?? '—' },
  ];
  return (
    <Card>
      <Flex direction="column" gap="4">
        <Flex>
          <ReplicationState enabled={status.enabled} result={{ data: status, isPending: false }} />
        </Flex>
        {suspended && status.remote.suspendedUntil !== null && (
          <Callout.Root color="amber" role="status">
            <Callout.Text>
              {t('replication.detail.suspended', {
                count: status.remote.failures,
                time: formatDateTime(language, status.remote.suspendedUntil),
                error: status.remote.lastError ?? t('replication.status.noError'),
              })}
            </Callout.Text>
          </Callout.Root>
        )}
        <Grid columns={{ initial: '1', sm: '2', md: '3' }} gap="4">
          {fields.map((field) => (
            <Flex direction="column" gap="1" key={field.label}>
              <Text size="1" color="gray">
                {field.label}
              </Text>
              <Text size="2">{field.value}</Text>
            </Flex>
          ))}
        </Grid>
      </Flex>
    </Card>
  );
}
