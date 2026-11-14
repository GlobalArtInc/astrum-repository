import { Button, Callout, Flex, IconButton, Text } from '@radix-ui/themes';
import { Pencil, PlugZap, Plus, Trash2 } from 'lucide-react';
import { useState } from 'react';
import { api, useInvalidateApi } from '../../api';
import { useCan } from '../../auth/state';
import { useT } from '../../i18n';
import {
  type Column,
  ConfirmDialog,
  DataTable,
  DateTime,
  describeError,
  EmptyState,
  useToast,
} from '../../ui';
import styles from '../security/shared/Form.module.css';
import {
  CREATE,
  DELETE,
  REMOTES,
  type RemoteCheck,
  type ReplicationRemote,
  remoteCheckPath,
  remotePath,
  UPDATE,
  useReplicationRemotes,
} from './api';
import { RemoteDialog } from './RemoteDialog';

interface CheckReport {
  name: string;
  result?: RemoteCheck;
  error?: unknown;
}

export function RemotesPanel() {
  const t = useT();
  const toast = useToast();
  const invalidate = useInvalidateApi();
  const canCreate = useCan(CREATE);
  const canUpdate = useCan(UPDATE);
  const canDelete = useCan(DELETE);
  const remotes = useReplicationRemotes();
  const [editing, setEditing] = useState<ReplicationRemote | 'new' | null>(null);
  const [deleting, setDeleting] = useState<ReplicationRemote | null>(null);
  const [checking, setChecking] = useState<string | null>(null);
  const [report, setReport] = useState<CheckReport | null>(null);

  const check = async (remote: ReplicationRemote) => {
    setChecking(remote.id);
    setReport(null);
    try {
      const result = await api.post<RemoteCheck>(remoteCheckPath(remote.id));
      setReport({ name: remote.name, result });
      await invalidate(REMOTES);
    } catch (failure) {
      setReport({ name: remote.name, error: failure });
    } finally {
      setChecking(null);
    }
  };

  const remove = async () => {
    if (deleting === null) {
      return;
    }
    await api.delete(remotePath(deleting.id));
    toast.success(t('replication.remotes.deleted', { name: deleting.name }));
    await invalidate(REMOTES);
  };

  const columns: Column<ReplicationRemote>[] = [
    {
      id: 'name',
      header: t('replication.remotes.name'),
      primary: true,
      cell: (remote) => <Text weight="medium">{remote.name}</Text>,
      sortValue: (remote) => remote.name,
    },
    {
      id: 'url',
      header: t('replication.remotes.url'),
      cell: (remote) => (
        <Text size="2" className={styles.mono}>
          {remote.url}
        </Text>
      ),
      sortValue: (remote) => remote.url,
    },
    {
      id: 'checkedAt',
      header: t('replication.remotes.checkedAt'),
      cell: (remote) =>
        remote.checkedAt ? (
          <DateTime value={remote.checkedAt} />
        ) : (
          <Text size="2" color="gray">
            {t('replication.remotes.neverChecked')}
          </Text>
        ),
      sortValue: (remote) => (remote.checkedAt ? new Date(remote.checkedAt) : null),
    },
    {
      id: 'version',
      header: t('replication.remotes.version'),
      cell: (remote) => remote.remoteVersion ?? '—',
      sortValue: (remote) => remote.remoteVersion,
    },
    {
      id: 'actions',
      header: '',
      align: 'right',
      cell: (remote) => (
        <Flex gap="2" justify="end" align="center" wrap="wrap">
          {canUpdate && (
            <Button
              size="1"
              variant="soft"
              loading={checking === remote.id}
              onClick={() => void check(remote)}
            >
              <PlugZap size={14} aria-hidden="true" />
              {t('replication.remotes.check')}
            </Button>
          )}
          {canUpdate && (
            <IconButton
              size="1"
              variant="ghost"
              color="gray"
              aria-label={t('replication.remotes.edit', { name: remote.name })}
              onClick={() => setEditing(remote)}
            >
              <Pencil size={14} aria-hidden="true" />
            </IconButton>
          )}
          {canDelete && (
            <IconButton
              size="1"
              variant="ghost"
              color="red"
              aria-label={t('replication.remotes.delete', { name: remote.name })}
              onClick={() => setDeleting(remote)}
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
            {t('replication.remotes.add')}
          </Button>
        )}
      </Flex>
      {report !== null && <CheckResult report={report} />}
      <DataTable
        label={t('replication.remotes.table')}
        columns={columns}
        rows={remotes.data}
        getRowId={(remote) => remote.id}
        loading={remotes.isPending}
        error={remotes.error}
        onRetry={() => void remotes.refetch()}
        empty={<EmptyState title={t('replication.remotes.empty')} />}
      />
      <RemoteDialog
        open={editing !== null}
        remote={editing === 'new' ? null : editing}
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
        title={deleting === null ? '' : t('replication.remotes.deleteTitle', { name: deleting.name })}
        description={t('replication.remotes.deleteText')}
        confirmLabel={t('replication.remotes.deleteConfirm')}
        danger
      />
    </Flex>
  );
}

interface CheckResultProps {
  report: CheckReport;
}

function CheckResult({ report }: CheckResultProps) {
  const t = useT();
  if (report.error !== undefined) {
    return (
      <Callout.Root color="red" role="alert">
        <Callout.Text>
          {t('replication.remotes.checkError', {
            name: report.name,
            message: describeError(t, report.error),
          })}
        </Callout.Text>
      </Callout.Root>
    );
  }
  const result = report.result;
  if (result === undefined) {
    return null;
  }
  if (!result.ok) {
    return (
      <Callout.Root color="red" role="alert">
        <Callout.Text>
          {t('replication.remotes.checkFailed', {
            name: report.name,
            kind: result.error.kind,
            message: result.error.message,
          })}
        </Callout.Text>
      </Callout.Root>
    );
  }
  return (
    <Callout.Root color="green" role="status">
      <Callout.Text>
        <Flex direction="column" gap="1">
          <span>
            {t('replication.remotes.checkOk', {
              name: report.name,
              protocol: result.protocol,
              version: result.version,
            })}
          </span>
          <span>{t('replication.remotes.checkFormats', { formats: result.formats.join(', ') || '—' })}</span>
        </Flex>
      </Callout.Text>
    </Callout.Root>
  );
}
