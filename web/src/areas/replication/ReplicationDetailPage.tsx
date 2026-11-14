import { Button, Callout, Flex, Spinner } from '@radix-ui/themes';
import { RotateCw } from 'lucide-react';
import { useState } from 'react';
import { useParams } from 'react-router';
import { api, useInvalidateApi } from '../../api';
import { useCan } from '../../auth/state';
import { useT } from '../../i18n';
import { describeError, EmptyState, ErrorState, Page, useToast } from '../../ui';
import {
  REPLICATIONS,
  REPLICATIONS_ADDRESS,
  reconcilePath,
  UPDATE,
  useReplicationList,
  useReplicationRemotes,
  useReplicationStatus,
} from './api';
import { ItemsTable } from './ItemsTable';
import { remoteName } from './model';
import { StatusCard } from './StatusCard';

export default function ReplicationDetailPage() {
  const t = useT();
  const toast = useToast();
  const invalidate = useInvalidateApi();
  const { id = '' } = useParams();
  const canUpdate = useCan(UPDATE);
  const replications = useReplicationList();
  const remotes = useReplicationRemotes();
  const status = useReplicationStatus(id);
  const [notice, setNotice] = useState<string | null>(null);
  const replication = replications.data?.find((candidate) => candidate.id === id);
  const crumbs = [
    { label: t('replication.crumbs.settings'), to: '/settings' },
    { label: t('replication.page.title'), to: REPLICATIONS_ADDRESS },
    { label: replication?.name ?? id },
  ];

  const reconcile = async () => {
    setNotice(null);
    try {
      await api.post(reconcilePath(id));
      toast.success(t('replication.detail.reconcileStarted'));
      await invalidate(REPLICATIONS);
    } catch (failure) {
      setNotice(describeError(t, failure));
    }
  };

  if (replications.error) {
    return (
      <Page title={id} breadcrumbs={crumbs}>
        <ErrorState error={replications.error} onRetry={() => void replications.refetch()} />
      </Page>
    );
  }
  if (replications.isPending) {
    return (
      <Page title={id} breadcrumbs={crumbs}>
        <Flex justify="center" py="9">
          <Spinner size="3" />
        </Flex>
      </Page>
    );
  }
  if (replication === undefined) {
    return (
      <Page title={t('replication.detail.notFoundTitle')} breadcrumbs={crumbs}>
        <EmptyState title={t('replication.detail.notFound')} />
      </Page>
    );
  }

  return (
    <Page
      title={replication.name}
      breadcrumbs={crumbs}
      description={`${remoteName(remotes.data, replication.remoteId)} · ${replication.remoteRepository}`}
      actions={
        <Button variant="soft" disabled={!canUpdate} onClick={() => void reconcile()}>
          <RotateCw size={16} aria-hidden="true" />
          {t('replication.detail.reconcile')}
        </Button>
      }
    >
      <Flex direction="column" gap="4">
        {notice !== null && (
          <Callout.Root color="amber" role="status">
            <Callout.Text>{notice}</Callout.Text>
          </Callout.Root>
        )}
        {status.isError ? (
          <ErrorState error={status.error} onRetry={() => void status.refetch()} />
        ) : status.data === undefined ? (
          <Flex justify="center" py="5">
            <Spinner size="3" />
          </Flex>
        ) : (
          <StatusCard status={status.data} />
        )}
        <ItemsTable replicationId={id} dead={status.data?.dead ?? 0} canUpdate={canUpdate} />
      </Flex>
    </Page>
  );
}
