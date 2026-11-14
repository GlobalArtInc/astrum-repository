import { Button, Flex, IconButton, Switch, Text } from '@radix-ui/themes';
import { useQueries } from '@tanstack/react-query';
import { Pencil, Plus, RotateCw, Trash2 } from 'lucide-react';
import { useState } from 'react';
import { api, useInvalidateApi } from '../../api';
import { useCan } from '../../auth/state';
import { useT } from '../../i18n';
import {
  type Column,
  ConfirmDialog,
  DataTable,
  describeError,
  EmptyState,
  RelativeTime,
  useToast,
} from '../../ui';
import {
  CREATE,
  DELETE,
  REPLICATIONS,
  type Replication,
  replicationAddress,
  replicationPath,
  statusQuery,
  UPDATE,
  useReplicationList,
  useReplicationRemotes,
} from './api';
import { lagText, remoteName, replicationBody, replicationValues } from './model';
import { ReplicationDialog } from './ReplicationDialog';
import { ReplicationState, StatusValue } from './Status';

export function ReplicationsPanel() {
  const t = useT();
  const toast = useToast();
  const invalidate = useInvalidateApi();
  const canCreate = useCan(CREATE);
  const canUpdate = useCan(UPDATE);
  const canDelete = useCan(DELETE);
  const replications = useReplicationList();
  const remotes = useReplicationRemotes();
  const rows = replications.data ?? [];
  const statuses = useQueries({ queries: rows.map((replication) => statusQuery(replication.id)) });
  const statusById = new Map(rows.map((replication, index) => [replication.id, statuses[index]] as const));
  const statusOf = (id: string) => statusById.get(id);
  const [editing, setEditing] = useState<Replication | 'new' | null>(null);
  const [deleting, setDeleting] = useState<Replication | null>(null);

  const toggle = async (replication: Replication) => {
    const enabled = !replication.enabled;
    try {
      await api.put(
        replicationPath(replication.id),
        replicationBody({ ...replicationValues(replication), enabled }),
      );
      toast.success(
        enabled
          ? t('replication.replications.enabledToast', { name: replication.name })
          : t('replication.replications.disabledToast', { name: replication.name }),
      );
      await invalidate(REPLICATIONS);
    } catch (failure) {
      toast.error(describeError(t, failure));
    }
  };

  const remove = async () => {
    if (deleting === null) {
      return;
    }
    await api.delete(replicationPath(deleting.id));
    toast.success(t('replication.replications.deleted', { name: deleting.name }));
    await invalidate(REPLICATIONS);
  };

  const filters = (replication: Replication): string => {
    const parts: string[] = [];
    if (replication.nameFilter) {
      parts.push(t('replication.replications.nameSummary', { pattern: replication.nameFilter }));
    }
    if (replication.versionFilter) {
      parts.push(t('replication.replications.versionSummary', { pattern: replication.versionFilter }));
    }
    return parts.length > 0 ? parts.join('; ') : t('replication.replications.noFilters');
  };

  const columns: Column<Replication>[] = [
    {
      id: 'name',
      header: t('replication.replications.name'),
      primary: true,
      cell: (replication) => <Text weight="medium">{replication.name}</Text>,
      sortValue: (replication) => replication.name,
    },
    {
      id: 'source',
      header: t('replication.replications.source'),
      cell: (replication) => replication.sourceRepository,
      sortValue: (replication) => replication.sourceRepository,
    },
    {
      id: 'remote',
      header: t('replication.replications.remote'),
      cell: (replication) => (
        <Text size="2">
          {remoteName(remotes.data, replication.remoteId)}
          {' · '}
          {replication.remoteRepository}
        </Text>
      ),
      sortValue: (replication) => remoteName(remotes.data, replication.remoteId),
    },
    {
      id: 'state',
      header: t('replication.replications.state'),
      cell: (replication) => (
        <ReplicationState enabled={replication.enabled} result={statusOf(replication.id)} />
      ),
      sortValue: (replication) => statusOf(replication.id)?.data?.alertState ?? '',
    },
    {
      id: 'pending',
      header: t('replication.replications.pending'),
      align: 'right',
      cell: (replication) => (
        <StatusValue result={statusOf(replication.id)}>{(status) => status.pending}</StatusValue>
      ),
      sortValue: (replication) => statusOf(replication.id)?.data?.pending ?? null,
    },
    {
      id: 'dead',
      header: t('replication.replications.dead'),
      align: 'right',
      cell: (replication) => (
        <StatusValue result={statusOf(replication.id)}>
          {(status) => (
            <Text size="2" color={status.dead > 0 ? 'red' : undefined}>
              {status.dead}
            </Text>
          )}
        </StatusValue>
      ),
      sortValue: (replication) => statusOf(replication.id)?.data?.dead ?? null,
    },
    {
      id: 'lag',
      header: t('replication.replications.lag'),
      cell: (replication) => (
        <StatusValue result={statusOf(replication.id)}>
          {(status) => <Text size="2">{lagText(t, status.lagSeconds)}</Text>}
        </StatusValue>
      ),
      sortValue: (replication) => statusOf(replication.id)?.data?.lagSeconds ?? null,
    },
    {
      id: 'lastSuccess',
      header: t('replication.replications.lastSuccess'),
      cell: (replication) => (
        <StatusValue result={statusOf(replication.id)}>
          {(status) =>
            status.lastSuccessAt === null ? (
              <Text size="2" color="gray">
                {t('replication.status.never')}
              </Text>
            ) : (
              <RelativeTime value={status.lastSuccessAt} />
            )
          }
        </StatusValue>
      ),
      sortValue: (replication) => {
        const at = statusOf(replication.id)?.data?.lastSuccessAt;
        return at ? new Date(at) : null;
      },
    },
    {
      id: 'filters',
      header: t('replication.replications.filters'),
      cell: (replication) => <Text size="2">{filters(replication)}</Text>,
    },
    {
      id: 'actions',
      header: '',
      align: 'right',
      cell: (replication) => (
        <Flex gap="2" justify="end" align="center" wrap="wrap">
          <Switch
            size="1"
            aria-label={
              replication.enabled
                ? t('replication.replications.disable', { name: replication.name })
                : t('replication.replications.enable', { name: replication.name })
            }
            checked={replication.enabled}
            disabled={!canUpdate}
            onCheckedChange={() => void toggle(replication)}
          />
          {canUpdate && (
            <IconButton
              size="1"
              variant="ghost"
              color="gray"
              aria-label={t('replication.replications.edit', { name: replication.name })}
              onClick={() => setEditing(replication)}
            >
              <Pencil size={14} aria-hidden="true" />
            </IconButton>
          )}
          {canDelete && (
            <IconButton
              size="1"
              variant="ghost"
              color="red"
              aria-label={t('replication.replications.delete', { name: replication.name })}
              onClick={() => setDeleting(replication)}
            >
              <Trash2 size={14} aria-hidden="true" />
            </IconButton>
          )}
        </Flex>
      ),
    },
  ];

  return (
    <Flex direction="column" gap="4">
      <Flex justify="end" gap="2">
        <Button
          variant="soft"
          color="gray"
          loading={replications.isFetching || statuses.some((status) => status.isFetching)}
          onClick={() => void invalidate(REPLICATIONS)}
        >
          <RotateCw size={16} aria-hidden="true" />
          {t('replication.replications.refresh')}
        </Button>
        {canCreate && (
          <Button onClick={() => setEditing('new')}>
            <Plus size={16} aria-hidden="true" />
            {t('replication.replications.add')}
          </Button>
        )}
      </Flex>
      <DataTable
        label={t('replication.replications.table')}
        columns={columns}
        rows={replications.data}
        getRowId={(replication) => replication.id}
        rowHref={(replication) => replicationAddress(replication.id)}
        loading={replications.isPending}
        error={replications.error}
        onRetry={() => void replications.refetch()}
        empty={<EmptyState title={t('replication.replications.empty')} />}
      />
      <ReplicationDialog
        open={editing !== null}
        replication={editing === 'new' ? null : editing}
        remotes={remotes.data ?? []}
        onOpenChange={(open) => {
          if (!open) {
            setEditing(null);
          }
        }}
      />
      <ConfirmDialog
        open={deleting !== null}
        onOpenChange={(open) => {
          if (!open) {
            setDeleting(null);
          }
        }}
        onConfirm={remove}
        title={deleting === null ? '' : t('replication.replications.deleteTitle', { name: deleting.name })}
        description={t('replication.replications.deleteText')}
        confirmLabel={t('replication.replications.deleteConfirm')}
        danger
      />
    </Flex>
  );
}
