import { Badge, Button, Card, IconButton, Switch, Table, Tooltip } from '@radix-ui/themes';
import { Bell, BellRing, Hash, Mail, Network, Pencil, Plus, Send, Trash2, Webhook } from 'lucide-react';
import { useState } from 'react';
import { api, useApi, useInvalidateApi } from '../../../api';
import { useCan } from '../../../auth/state';
import { useT } from '../../../i18n';
import { ConfirmDialog, describeError, EmptyState, ErrorState, Page, useToast } from '../../../ui';
import styles from '../Notify.module.css';
import { ChannelDialog } from './ChannelDialog';
import {
  CHANNELS,
  type Channel,
  type ChannelType,
  EVENT_LABELS,
  fromChannel,
  type TestResult,
  TYPE_LABELS,
  UPDATE,
} from './model';

const MAX_REPOSITORIES = 3;
const TYPE_ICONS: Record<ChannelType, typeof Send> = {
  telegram: Send,
  slack: Hash,
  webhook: Webhook,
  email: Mail,
};
const EVENT_COLORS = { published: 'green', updated: 'blue', deleted: 'red' } as const;

function ChannelRow({
  channel,
  canWrite,
  onEdit,
  onDelete,
}: {
  channel: Channel;
  canWrite: boolean;
  onEdit(): void;
  onDelete(): void;
}) {
  const t = useT();
  const toast = useToast();
  const invalidate = useInvalidateApi();
  const [busy, setBusy] = useState(false);
  const Icon = TYPE_ICONS[channel.type];
  const shown = channel.repositories.slice(0, MAX_REPOSITORIES);
  const more = channel.repositories.length - shown.length;

  const toggle = async (enabled: boolean) => {
    setBusy(true);
    try {
      await api.put(`${CHANNELS}/${encodeURIComponent(channel.id)}`, fromChannel(channel, enabled));
      await invalidate(CHANNELS);
    } catch (error) {
      toast.error(`${t('notify.notifications.saveError')}: ${describeError(t, error)}`);
    } finally {
      setBusy(false);
    }
  };

  const test = async () => {
    setBusy(true);
    try {
      const result = await api.post<TestResult>(`${CHANNELS}/${encodeURIComponent(channel.id)}/test`);
      if (result.success) {
        toast.success(t('notify.notifications.testSuccess', { name: channel.name }));
      } else {
        toast.error(`${t('notify.notifications.testFailure')}: ${result.message ?? ''}`);
      }
    } catch (error) {
      toast.error(`${t('notify.notifications.testFailure')}: ${describeError(t, error)}`);
    } finally {
      setBusy(false);
    }
  };

  return (
    <Table.Row align="center">
      <Table.RowHeaderCell>
        <div className={styles.channelName}>{channel.name}</div>
        <div className={styles.badges}>
          <Badge
            color={
              channel.type === 'telegram'
                ? 'sky'
                : channel.type === 'slack'
                  ? 'purple'
                  : channel.type === 'email'
                    ? 'cyan'
                    : 'orange'
            }
          >
            <Icon size={12} aria-hidden="true" />
            {t(TYPE_LABELS[channel.type])}
          </Badge>
          {channel.proxyMode === 'custom' && (
            <Tooltip
              content={t('notify.notifications.proxyTooltip', {
                type: (channel.proxyType ?? 'http').toUpperCase(),
                address: `${channel.proxyHost ?? ''}:${channel.proxyPort ?? ''}`,
              })}
            >
              <Badge color="gray" variant="outline">
                <Network size={12} aria-hidden="true" />
                {t('notify.notifications.proxyBadge')}
              </Badge>
            </Tooltip>
          )}
        </div>
      </Table.RowHeaderCell>
      <Table.Cell>
        <code className={styles.target}>
          {channel.target}
          {channel.type === 'telegram' && channel.messageThreadId ? ` #${channel.messageThreadId}` : ''}
        </code>
      </Table.Cell>
      <Table.Cell>
        {channel.repositories.length === 0 ? (
          <span className={styles.muted}>{t('notify.notifications.allHosted')}</span>
        ) : (
          <div className={styles.badges}>
            {shown.map((name) => (
              <Badge key={name} color="gray" variant="outline">
                {name}
              </Badge>
            ))}
            {more > 0 && (
              <Badge color="gray" variant="soft">
                {t('notify.notifications.moreRepositories', { count: more })}
              </Badge>
            )}
          </div>
        )}
      </Table.Cell>
      <Table.Cell>
        <div className={styles.badges}>
          {channel.events.map((event) => (
            <Badge key={event} color={EVENT_COLORS[event]}>
              {t(EVENT_LABELS[event])}
            </Badge>
          ))}
        </div>
      </Table.Cell>
      <Table.Cell>
        <Switch
          checked={channel.enabled}
          disabled={!canWrite || busy}
          aria-label={t('notify.notifications.enableLabel', { name: channel.name })}
          onCheckedChange={(checked) => void toggle(checked)}
        />
      </Table.Cell>
      <Table.Cell>
        {canWrite && (
          <div className={styles.actions}>
            <IconButton
              variant="ghost"
              color="gray"
              size="2"
              aria-label={t('notify.notifications.testAction')}
              title={t('notify.notifications.testAction')}
              disabled={busy}
              onClick={() => void test()}
            >
              <BellRing size={16} />
            </IconButton>
            <IconButton
              variant="ghost"
              color="gray"
              size="2"
              aria-label={t('notify.notifications.edit')}
              title={t('notify.notifications.edit')}
              onClick={onEdit}
            >
              <Pencil size={16} />
            </IconButton>
            <IconButton
              variant="ghost"
              color="red"
              size="2"
              aria-label={t('notify.notifications.delete')}
              title={t('notify.notifications.delete')}
              onClick={onDelete}
            >
              <Trash2 size={16} />
            </IconButton>
          </div>
        )}
      </Table.Cell>
    </Table.Row>
  );
}

export default function NotificationsPage() {
  const t = useT();
  const toast = useToast();
  const invalidate = useInvalidateApi();
  const canWrite = useCan(UPDATE);
  const channels = useApi<Channel[]>(CHANNELS);
  const [editing, setEditing] = useState<Channel | null | undefined>(undefined);
  const [deleting, setDeleting] = useState<Channel | null>(null);
  const [dialogKey, setDialogKey] = useState(0);

  const open = (channel: Channel | null) => {
    setDialogKey((key) => key + 1);
    setEditing(channel);
  };

  const remove = async () => {
    if (!deleting) {
      return;
    }
    await api.delete(`${CHANNELS}/${encodeURIComponent(deleting.id)}`);
    toast.success(t('notify.notifications.deleted', { name: deleting.name }));
    await invalidate(CHANNELS);
  };

  const add = canWrite ? (
    <Button onClick={() => open(null)}>
      <Plus size={16} aria-hidden="true" />
      {t('notify.notifications.add')}
    </Button>
  ) : null;
  const list = channels.data ?? [];

  return (
    <Page
      className={styles.page}
      title={t('notify.notifications.title')}
      description={t('notify.notifications.description')}
      breadcrumbs={[
        { label: t('notify.crumbs.settings'), to: '/settings' },
        { label: t('notify.notifications.title') },
      ]}
      actions={list.length > 0 ? add : null}
    >
      {channels.error ? (
        <ErrorState error={channels.error} onRetry={() => void channels.refetch()} />
      ) : channels.isPending ? (
        <Card className={styles.card} aria-busy="true">
          <p className={styles.muted}>{t('notify.notifications.loading')}</p>
        </Card>
      ) : list.length === 0 ? (
        <Card className={styles.card}>
          <EmptyState
            icon={Bell}
            title={t('notify.notifications.emptyTitle')}
            description={t('notify.notifications.emptyText')}
            action={add}
          />
        </Card>
      ) : (
        <Card className={styles.card}>
          <div className={styles.scroll}>
            <Table.Root className={styles.table} aria-label={t('notify.notifications.tableLabel')}>
              <Table.Header>
                <Table.Row>
                  <Table.ColumnHeaderCell>{t('notify.notifications.columns.channel')}</Table.ColumnHeaderCell>
                  <Table.ColumnHeaderCell>{t('notify.notifications.columns.target')}</Table.ColumnHeaderCell>
                  <Table.ColumnHeaderCell>
                    {t('notify.notifications.columns.repositories')}
                  </Table.ColumnHeaderCell>
                  <Table.ColumnHeaderCell>{t('notify.notifications.columns.events')}</Table.ColumnHeaderCell>
                  <Table.ColumnHeaderCell>{t('notify.notifications.columns.enabled')}</Table.ColumnHeaderCell>
                  <Table.ColumnHeaderCell>
                    <span className={styles.hidden}>{t('notify.notifications.columns.actions')}</span>
                  </Table.ColumnHeaderCell>
                </Table.Row>
              </Table.Header>
              <Table.Body>
                {list.map((channel) => (
                  <ChannelRow
                    key={channel.id}
                    channel={channel}
                    canWrite={canWrite}
                    onEdit={() => open(channel)}
                    onDelete={() => setDeleting(channel)}
                  />
                ))}
              </Table.Body>
            </Table.Root>
          </div>
        </Card>
      )}
      {editing !== undefined && (
        <ChannelDialog key={dialogKey} channel={editing} open onClose={() => setEditing(undefined)} />
      )}
      <ConfirmDialog
        open={deleting !== null}
        title={t('notify.notifications.deleteTitle')}
        description={t('notify.notifications.deleteText', { name: deleting?.name ?? '' })}
        confirmLabel={t('notify.notifications.deleteConfirm')}
        danger
        onConfirm={remove}
        onOpenChange={(next) => !next && setDeleting(null)}
      />
    </Page>
  );
}
