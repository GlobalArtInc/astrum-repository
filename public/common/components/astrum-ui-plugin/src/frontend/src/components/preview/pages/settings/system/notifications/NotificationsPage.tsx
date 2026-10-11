import React, { useCallback, useEffect, useState } from 'react';
import {
  Badge,
  Box,
  Button,
  Callout,
  Card,
  Code,
  Flex,
  IconButton,
  Spinner,
  Switch,
  Table,
  Text,
  Tooltip,
  VisuallyHidden,
} from '@radix-ui/themes';
import { Bell, BellRing, Hash, Network, Pencil, Plus, Send, Trash2, Webhook, XCircle } from 'lucide-react';

import { ExtJS } from '../../../../../../interface/ExtJS';
import Permissions from '../../../../../../constants/Permissions';
import UIStrings from '../../../../../../constants/UIStrings';
import { ConfirmDialog, EmptyState, PageHeader, useToast } from '../../../../shared';
import { NotificationChannelDialog } from './NotificationChannelDialog';
import {
  deleteChannel,
  listChannels,
  listRepositories,
  serverError,
  testChannel,
  updateChannel,
} from './notificationsApi';
import { NOTIFICATION_EVENTS, NotificationChannel, NotificationEvent, RepositoryOption } from './types';

const LABELS = UIStrings.NOTIFICATIONS;

const MAX_REPOSITORY_BADGES = 3;

const EVENT_LABELS: Record<NotificationEvent, string> = {
  published: LABELS.EVENTS.PUBLISHED,
  updated: LABELS.EVENTS.UPDATED,
  deleted: LABELS.EVENTS.DELETED,
};

const EVENT_COLORS: Record<NotificationEvent, 'green' | 'blue' | 'red'> = {
  published: 'green',
  updated: 'blue',
  deleted: 'red',
};

const openSettingsHub = () => {
  window.location.hash = '#preview/admin/settings';
};

const TYPE_BADGES = {
  telegram: { color: 'sky', icon: Send, label: LABELS.TYPES.TELEGRAM },
  slack: { color: 'violet', icon: Hash, label: LABELS.TYPES.SLACK },
  webhook: { color: 'orange', icon: Webhook, label: LABELS.TYPES.WEBHOOK },
} as const;

function TypeBadge({ channel }: { channel: NotificationChannel }) {
  const { color, icon: Icon, label } = TYPE_BADGES[channel.type] ?? TYPE_BADGES.slack;
  return (
    <Badge color={color} variant="soft" data-testid={`notification-type-${channel.id}`}>
      <Icon size={12} />
      {label}
    </Badge>
  );
}

function ProxyBadge({ channel }: { channel: NotificationChannel }) {
  if (channel.proxyMode !== 'custom') {
    return null;
  }
  const type = channel.proxyType === 'socks5' ? LABELS.DIALOG.PROXY_TYPES.SOCKS5 : LABELS.DIALOG.PROXY_TYPES.HTTP;
  return (
    <Tooltip content={LABELS.PROXY_TOOLTIP(type, `${channel.proxyHost}:${channel.proxyPort}`)}>
      <Badge color="gray" variant="outline" data-testid={`notification-proxy-${channel.id}`}>
        <Network size={12} />
        {LABELS.PROXY_BADGE}
      </Badge>
    </Tooltip>
  );
}

function Repositories({ names }: { names: string[] }) {
  if (names.length === 0) {
    return (
      <Text size="2" color="gray">
        {LABELS.ALL_HOSTED}
      </Text>
    );
  }
  const shown = names.slice(0, MAX_REPOSITORY_BADGES);
  return (
    <Flex gap="1" wrap="wrap">
      {shown.map((name) => (
        <Badge key={name} color="gray" variant="surface">
          {name}
        </Badge>
      ))}
      {names.length > shown.length && (
        <Tooltip content={names.slice(MAX_REPOSITORY_BADGES).join(', ')}>
          <Badge color="gray" variant="soft">
            {LABELS.MORE_REPOSITORIES(names.length - shown.length)}
          </Badge>
        </Tooltip>
      )}
    </Flex>
  );
}

export function NotificationsPage() {
  const canUpdate = ExtJS.checkPermission(Permissions.SETTINGS.UPDATE);
  const toast = useToast();

  const [channels, setChannels] = useState<NotificationChannel[]>([]);
  const [repositories, setRepositories] = useState<RepositoryOption[]>([]);
  const [loading, setLoading] = useState(true);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [editing, setEditing] = useState<NotificationChannel | 'new' | null>(null);
  const [deleting, setDeleting] = useState<NotificationChannel | null>(null);
  const [busy, setBusy] = useState<string | null>(null);

  const load = useCallback(async () => {
    setLoading(true);
    setLoadError(null);
    try {
      setChannels(await listChannels());
    }
    catch (error) {
      setLoadError(serverError(error, LABELS.LOAD_ERROR).message);
    }
    finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    load();
    if (canUpdate) {
      listRepositories()
        .then(setRepositories)
        .catch(() => setRepositories([]));
    }
  }, [load, canUpdate]);

  const handleSaved = (saved: NotificationChannel) => {
    setEditing(null);
    toast.success(LABELS.SAVE_SUCCESS(saved.name));
    load();
  };

  const handleToggle = async (channel: NotificationChannel, enabled: boolean) => {
    if (!channel.id) {
      return;
    }
    setBusy(channel.id);
    setChannels((current) => current.map((c) => (c.id === channel.id ? { ...c, enabled } : c)));
    try {
      await updateChannel(channel.id, { ...channel, enabled });
    }
    catch (error) {
      setChannels((current) => current.map((c) => (c.id === channel.id ? { ...c, enabled: !enabled } : c)));
      toast.error(LABELS.SAVE_ERROR, serverError(error, LABELS.SAVE_ERROR).message);
    }
    finally {
      setBusy(null);
    }
  };

  const handleTest = async (channel: NotificationChannel) => {
    if (!channel.id) {
      return;
    }
    setBusy(channel.id);
    try {
      const result = await testChannel(channel.id);
      if (result.success) {
        toast.success(LABELS.TEST_SUCCESS(channel.name));
      }
      else {
        toast.error(LABELS.TEST_FAILURE, result.message ?? undefined);
      }
    }
    catch (error) {
      toast.error(LABELS.TEST_FAILURE, serverError(error, LABELS.TEST_FAILURE).message);
    }
    finally {
      setBusy(null);
    }
  };

  const handleDelete = async () => {
    const channel = deleting;
    if (!channel?.id) {
      return;
    }
    setDeleting(null);
    try {
      await deleteChannel(channel.id);
      toast.success(LABELS.DELETE_SUCCESS(channel.name));
      setChannels((current) => current.filter((c) => c.id !== channel.id));
    }
    catch (error) {
      toast.error(LABELS.DELETE_ERROR, serverError(error, LABELS.DELETE_ERROR).message);
    }
  };

  const createButton = canUpdate ? (
    <Button onClick={() => setEditing('new')} data-testid="notification-create-button">
      <Plus size={16} />
      {LABELS.CREATE_BUTTON}
    </Button>
  ) : undefined;

  return (
    <Box
      className="notifications-page"
      data-testid="notifications-page"
      px={{ initial: '4', md: '6', lg: '6' }}
      py={{ initial: '4', md: '5', lg: '6' }}
    >
      <Box mb="4">
        <PageHeader
          title={LABELS.TITLE}
          description={LABELS.DESCRIPTION}
          actions={channels.length > 0 ? createButton : undefined}
          breadcrumbs={[
            { label: LABELS.BREADCRUMB_SETTINGS, onClick: openSettingsHub },
            { label: LABELS.TITLE },
          ]}
        />
      </Box>

      {loadError && (
        <Box mb="4">
          <Callout.Root color="red" role="alert" data-testid="notifications-load-error">
            <Callout.Icon>
              <XCircle size={16} />
            </Callout.Icon>
            <Callout.Text>{loadError}</Callout.Text>
          </Callout.Root>
        </Box>
      )}

      {loading ? (
        <Flex align="center" gap="2" py="6" justify="center">
          <Spinner size="2" />
          <Text size="2" color="gray">
            {LABELS.LOADING}
          </Text>
        </Flex>
      ) : (
        !loadError &&
        (channels.length === 0 ? (
          <Card>
            <EmptyState
              icon={Bell}
              title={LABELS.EMPTY.TITLE}
              description={LABELS.EMPTY.DESCRIPTION}
              action={canUpdate ? { label: LABELS.CREATE_BUTTON, onClick: () => setEditing('new') } : undefined}
            />
          </Card>
        ) : (
          <Card size="1">
            <Table.Root variant="ghost" aria-label={LABELS.TABLE_LABEL} data-testid="notifications-table">
              <Table.Header>
                <Table.Row>
                  <Table.ColumnHeaderCell>{LABELS.COLUMNS.CHANNEL}</Table.ColumnHeaderCell>
                  <Table.ColumnHeaderCell>{LABELS.COLUMNS.TARGET}</Table.ColumnHeaderCell>
                  <Table.ColumnHeaderCell>{LABELS.COLUMNS.REPOSITORIES}</Table.ColumnHeaderCell>
                  <Table.ColumnHeaderCell>{LABELS.COLUMNS.EVENTS}</Table.ColumnHeaderCell>
                  <Table.ColumnHeaderCell>{LABELS.COLUMNS.ENABLED}</Table.ColumnHeaderCell>
                  <Table.ColumnHeaderCell align="right">
                    <VisuallyHidden>{LABELS.COLUMNS.ACTIONS}</VisuallyHidden>
                  </Table.ColumnHeaderCell>
                </Table.Row>
              </Table.Header>
              <Table.Body>
                {channels.map((channel) => (
                  <Table.Row key={channel.id} align="center" data-testid={`notification-row-${channel.id}`}>
                    <Table.RowHeaderCell>
                      <Flex direction="column" gap="1" align="start">
                        <Text size="2" weight="medium">
                          {channel.name}
                        </Text>
                        <Flex gap="1" wrap="wrap">
                          <TypeBadge channel={channel} />
                          <ProxyBadge channel={channel} />
                        </Flex>
                      </Flex>
                    </Table.RowHeaderCell>
                    <Table.Cell>
                      <Code variant="ghost" size="2">
                        {channel.target || '—'}
                        {channel.type === 'telegram' && channel.messageThreadId ? ` #${channel.messageThreadId}` : ''}
                      </Code>
                    </Table.Cell>
                    <Table.Cell>
                      <Repositories names={channel.repositories || []} />
                    </Table.Cell>
                    <Table.Cell>
                      <Flex gap="1" wrap="wrap">
                        {NOTIFICATION_EVENTS.filter((event) => channel.events?.includes(event)).map((event) => (
                          <Badge key={event} color={EVENT_COLORS[event]} variant="soft">
                            {EVENT_LABELS[event]}
                          </Badge>
                        ))}
                      </Flex>
                    </Table.Cell>
                    <Table.Cell>
                      <Switch
                        checked={channel.enabled}
                        disabled={!canUpdate || busy === channel.id}
                        onCheckedChange={(checked) => handleToggle(channel, checked)}
                        aria-label={LABELS.ENABLE_LABEL(channel.name)}
                        data-testid={`notification-toggle-${channel.id}`}
                      />
                    </Table.Cell>
                    <Table.Cell justify="end">
                      {canUpdate && (
                        <Flex gap="1" justify="end">
                          <Tooltip content={LABELS.ACTIONS.TEST}>
                            <IconButton
                              variant="ghost"
                              color="gray"
                              aria-label={LABELS.ACTIONS.TEST}
                              disabled={busy === channel.id}
                              onClick={() => handleTest(channel)}
                              data-testid={`notification-test-${channel.id}`}
                            >
                              {busy === channel.id ? <Spinner size="1" /> : <BellRing size={16} />}
                            </IconButton>
                          </Tooltip>
                          <Tooltip content={LABELS.ACTIONS.EDIT}>
                            <IconButton
                              variant="ghost"
                              color="gray"
                              aria-label={LABELS.ACTIONS.EDIT}
                              onClick={() => setEditing(channel)}
                              data-testid={`notification-edit-${channel.id}`}
                            >
                              <Pencil size={16} />
                            </IconButton>
                          </Tooltip>
                          <Tooltip content={LABELS.ACTIONS.DELETE}>
                            <IconButton
                              variant="ghost"
                              color="red"
                              aria-label={LABELS.ACTIONS.DELETE}
                              onClick={() => setDeleting(channel)}
                              data-testid={`notification-delete-${channel.id}`}
                            >
                              <Trash2 size={16} />
                            </IconButton>
                          </Tooltip>
                        </Flex>
                      )}
                    </Table.Cell>
                  </Table.Row>
                ))}
              </Table.Body>
            </Table.Root>
          </Card>
        ))
      )}

      {editing && (
        <NotificationChannelDialog
          channel={editing === 'new' ? null : editing}
          repositories={repositories}
          onClose={() => setEditing(null)}
          onSaved={handleSaved}
        />
      )}

      <ConfirmDialog
        open={deleting !== null}
        onOpenChange={(open) => { if (!open) setDeleting(null); }}
        title={LABELS.DELETE_DIALOG.TITLE}
        message={deleting ? LABELS.DELETE_DIALOG.MESSAGE(deleting.name) : ''}
        confirmLabel={LABELS.DELETE_DIALOG.CONFIRM}
        cancelLabel={LABELS.DELETE_DIALOG.CANCEL}
        variant="danger"
        onConfirm={handleDelete}
      />
    </Box>
  );
}

export default NotificationsPage;
