import { Button, Flex, IconButton, Switch, Text } from '@radix-ui/themes';
import { Pencil, Plus, Trash2 } from 'lucide-react';
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
  StatusBadge,
  useToast,
} from '../../ui';
import {
  CREATE,
  DELETE,
  REPLICATIONS,
  type Replication,
  replicationPath,
  UPDATE,
  useReplicationList,
  useReplicationRemotes,
} from './api';
import { remoteName, replicationBody, replicationValues } from './model';
import { ReplicationDialog } from './ReplicationDialog';

export function ReplicationsPanel() {
  const t = useT();
  const toast = useToast();
  const invalidate = useInvalidateApi();
  const canCreate = useCan(CREATE);
  const canUpdate = useCan(UPDATE);
  const canDelete = useCan(DELETE);
  const replications = useReplicationList();
  const remotes = useReplicationRemotes();
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
      cell: (replication) =>
        replication.enabled ? (
          <StatusBadge tone="success">{t('replication.replications.enabled')}</StatusBadge>
        ) : (
          <StatusBadge tone="neutral">{t('replication.replications.disabled')}</StatusBadge>
        ),
      sortValue: (replication) => (replication.enabled ? 1 : 0),
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
      <Flex justify="end">
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
