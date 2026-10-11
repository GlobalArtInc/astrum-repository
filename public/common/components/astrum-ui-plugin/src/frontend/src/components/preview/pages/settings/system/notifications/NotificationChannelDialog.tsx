import React, { useEffect, useMemo, useState } from 'react';
import {
  Badge,
  Box,
  Button,
  Callout,
  Checkbox,
  Dialog,
  Flex,
  Link,
  RadioCards,
  ScrollArea,
  SegmentedControl,
  Select,
  Spinner,
  Switch,
  Text,
  TextField,
} from '@radix-ui/themes';
import { CheckCircle2, Hash, Info, Send, XCircle } from 'lucide-react';

import UIStrings from '../../../../../../constants/UIStrings';
import { createChannel, serverError, testUnsavedChannel, updateChannel } from './notificationsApi';
import {
  NOTIFICATION_EVENTS,
  NotificationChannel,
  NotificationChannelType,
  NotificationEvent,
  NotificationLanguage,
  NotificationProxyMode,
  NotificationProxyType,
  RepositoryOption,
} from './types';

const LABELS = UIStrings.NOTIFICATIONS;
const DIALOG = LABELS.DIALOG;

const EVENT_LABELS: Record<NotificationEvent, { label: string; help: string }> = {
  published: { label: LABELS.EVENTS.PUBLISHED, help: LABELS.EVENT_HELP.PUBLISHED },
  updated: { label: LABELS.EVENTS.UPDATED, help: LABELS.EVENT_HELP.UPDATED },
  deleted: { label: LABELS.EVENTS.DELETED, help: LABELS.EVENT_HELP.DELETED },
};

const THREAD_ID = /^\d+$/;

const PORT = /^\d{1,5}$/;

const PROXY_HOST = /^(\[[0-9A-Fa-f:.]+\]|[A-Za-z0-9_.:-]+)$/;

const CONNECTION_MODES: Array<{ value: NotificationProxyMode; label: string; help: string }> = [
  { value: 'global', label: DIALOG.CONNECTION_MODES.GLOBAL, help: DIALOG.CONNECTION_HELP.GLOBAL },
  { value: 'none', label: DIALOG.CONNECTION_MODES.NONE, help: DIALOG.CONNECTION_HELP.NONE },
  { value: 'custom', label: DIALOG.CONNECTION_MODES.CUSTOM, help: DIALOG.CONNECTION_HELP.CUSTOM },
];

const FIELDS = new Set(['name', 'chatId', 'messageThreadId', 'secret', 'events', 'repositories', 'type', 'language',
  'proxyMode', 'proxyType', 'proxyHost', 'proxyPort', 'proxyUsername', 'proxyPassword']);

interface FormState {
  name: string;
  type: NotificationChannelType;
  enabled: boolean;
  chatId: string;
  thread: string;
  secret: string;
  repositories: string[];
  events: NotificationEvent[];
  language: NotificationLanguage;
  proxyMode: NotificationProxyMode;
  proxyType: NotificationProxyType;
  proxyHost: string;
  proxyPort: string;
  proxyUsername: string;
  proxyPassword: string;
  removeProxyPassword: boolean;
}

type Errors = Partial<Record<'name' | 'chatId' | 'messageThreadId' | 'secret' | 'events' | 'repositories' | 'type'
  | 'language' | 'proxyMode' | 'proxyType' | 'proxyHost' | 'proxyPort' | 'proxyUsername' | 'proxyPassword', string>>;

type TestState = { status: 'idle' } | { status: 'running' } | { status: 'done'; success: boolean; message?: string };

function initialState(channel?: NotificationChannel | null): FormState {
  return {
    name: channel?.name ?? '',
    type: channel?.type ?? 'telegram',
    enabled: channel?.enabled ?? true,
    chatId: channel?.chatId ?? '',
    thread: channel?.messageThreadId ? String(channel.messageThreadId) : '',
    secret: '',
    repositories: channel?.repositories ?? [],
    events: channel?.events ?? [...NOTIFICATION_EVENTS],
    language: channel?.language ?? 'ru',
    proxyMode: channel?.proxyMode ?? 'global',
    proxyType: channel?.proxyType ?? 'http',
    proxyHost: channel?.proxyHost ?? '',
    proxyPort: channel?.proxyPort ? String(channel.proxyPort) : '',
    proxyUsername: channel?.proxyUsername ?? '',
    proxyPassword: '',
    removeProxyPassword: false,
  };
}

function proxyHost(value: string): string {
  const host = value.trim();
  return host.startsWith('[') && host.endsWith(']') ? host.slice(1, -1) : host;
}

function Field({
  label,
  htmlFor,
  help,
  error,
  children,
}: {
  label: string;
  htmlFor?: string;
  help?: React.ReactNode;
  error?: string;
  children: React.ReactNode;
}) {
  return (
    <Flex direction="column" gap="1">
      <Text as="label" size="2" weight="medium" htmlFor={htmlFor}>
        {label}
      </Text>
      {children}
      {error ? (
        <Text size="1" color="red" role="alert" data-testid={htmlFor ? `${htmlFor}-error` : undefined}>
          {error}
        </Text>
      ) : (
        help && (
          <Text size="1" color="gray">
            {help}
          </Text>
        )
      )}
    </Flex>
  );
}

function ConnectionHelp({ type }: { type: NotificationChannelType }) {
  const help = type === 'telegram' ? DIALOG.TELEGRAM_HELP : DIALOG.SLACK_HELP;
  const steps: Array<[string, React.ReactNode]> =
    type === 'telegram'
      ? [
          ['bot', DIALOG.TELEGRAM_HELP.STEP_BOT],
          ['chat', DIALOG.TELEGRAM_HELP.STEP_CHAT],
          ['id', DIALOG.TELEGRAM_HELP.STEP_ID],
        ]
      : [
          ['webhook', DIALOG.SLACK_HELP.STEP_WEBHOOK],
          ['url', DIALOG.SLACK_HELP.STEP_URL],
        ];
  return (
    <Callout.Root color="gray" variant="surface" size="1" data-testid="notification-help">
      <Callout.Icon>
        <Info size={16} />
      </Callout.Icon>
      <Box>
        <Text as="div" size="2" weight="medium" mb="1">
          {help.TITLE}
        </Text>
        <ol
          style={{
            margin: 0,
            paddingLeft: 'var(--space-4)',
            fontSize: 'var(--font-size-1)',
            lineHeight: 'var(--line-height-1)',
          }}
        >
          {steps.map(([key, step]) => (
            <li key={key} style={{ marginBottom: 'var(--space-1)' }}>
              {step}
            </li>
          ))}
        </ol>
      </Box>
    </Callout.Root>
  );
}

export interface NotificationChannelDialogProps {
  /** The channel to edit, or null to create one. */
  channel: NotificationChannel | null;
  repositories: RepositoryOption[];
  onClose: () => void;
  onSaved: (channel: NotificationChannel) => void;
}

export function NotificationChannelDialog({ channel, repositories, onClose, onSaved }: NotificationChannelDialogProps) {
  const [form, setForm] = useState<FormState>(() => initialState(channel));
  const [errors, setErrors] = useState<Errors>({});
  const [formError, setFormError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [test, setTest] = useState<TestState>({ status: 'idle' });
  const [filter, setFilter] = useState('');

  useEffect(() => {
    setForm(initialState(channel));
    setErrors({});
    setFormError(null);
    setTest({ status: 'idle' });
  }, [channel]);

  const keepsSecret = Boolean(channel?.secret) && channel?.type === form.type;
  const isTelegram = form.type === 'telegram';
  const isCustomProxy = form.proxyMode === 'custom';
  const savedProxyPassword = Boolean(channel?.proxyPassword) && channel?.proxyMode === 'custom';
  const sameProxy = savedProxyPassword
    && form.proxyType === channel?.proxyType
    && proxyHost(form.proxyHost).toLowerCase() === (channel?.proxyHost ?? '').toLowerCase()
    && Number(form.proxyPort) === channel?.proxyPort;
  const keepsProxyPassword = isCustomProxy && sameProxy && Boolean(form.proxyUsername.trim())
    && !form.removeProxyPassword;

  const repositoryOptions = useMemo(() => {
    const known = new Map(repositories.map((repository) => [repository.name, repository]));
    form.repositories.forEach((name) => {
      if (!known.has(name)) {
        known.set(name, { name, format: '', type: '' });
      }
    });
    const query = filter.trim().toLowerCase();
    return [...known.values()]
      .filter((repository) => !query || repository.name.toLowerCase().includes(query))
      .sort((a, b) => a.name.localeCompare(b.name));
  }, [repositories, form.repositories, filter]);

  const update = <K extends keyof FormState>(key: K, value: FormState[K]) => {
    setForm((current) => ({ ...current, [key]: value }));
    setErrors((current) => ({ ...current, [key === 'thread' ? 'messageThreadId' : key]: undefined }));
    setTest({ status: 'idle' });
  };

  const toggle = <T extends string>(values: T[], value: T, checked: boolean): T[] =>
    checked ? [...values.filter((v) => v !== value), value] : values.filter((v) => v !== value);

  const validateConnection = (): Errors => {
    const found: Errors = {};
    if (isTelegram && !form.chatId.trim()) {
      found.chatId = DIALOG.ERRORS.CHAT_ID;
    }
    if (isTelegram && form.thread.trim() && !(THREAD_ID.test(form.thread.trim()) && Number(form.thread) > 0)) {
      found.messageThreadId = DIALOG.ERRORS.THREAD;
    }
    if (!form.secret.trim() && !keepsSecret) {
      found.secret = isTelegram ? DIALOG.ERRORS.BOT_TOKEN : DIALOG.ERRORS.WEBHOOK;
    }
    if (isCustomProxy) {
      if (!form.proxyHost.trim()) {
        found.proxyHost = DIALOG.ERRORS.PROXY_HOST;
      }
      else if (!PROXY_HOST.test(form.proxyHost.trim())) {
        found.proxyHost = DIALOG.ERRORS.PROXY_HOST_FORMAT;
      }
      const port = form.proxyPort.trim();
      if (!PORT.test(port) || Number(port) < 1 || Number(port) > 65535) {
        found.proxyPort = DIALOG.ERRORS.PROXY_PORT;
      }
      if (form.proxyPassword && !form.proxyUsername.trim()) {
        found.proxyUsername = DIALOG.ERRORS.PROXY_USERNAME;
      }
    }
    return found;
  };

  const validate = (): Errors => {
    const found = validateConnection();
    if (!form.name.trim()) {
      found.name = DIALOG.ERRORS.NAME;
    }
    if (form.events.length === 0) {
      found.events = DIALOG.ERRORS.EVENTS;
    }
    return found;
  };

  const payload = (): NotificationChannel => ({
    id: channel?.id,
    name: form.name.trim(),
    type: form.type,
    enabled: form.enabled,
    chatId: isTelegram ? form.chatId.trim() : null,
    messageThreadId: isTelegram && form.thread.trim() ? Number(form.thread.trim()) : null,
    secret: form.secret.trim() || (keepsSecret ? channel?.secret : ''),
    repositories: form.repositories,
    events: NOTIFICATION_EVENTS.filter((event) => form.events.includes(event)),
    language: form.language,
    proxyMode: form.proxyMode,
    proxyType: isCustomProxy ? form.proxyType : null,
    proxyHost: isCustomProxy ? proxyHost(form.proxyHost) : null,
    proxyPort: isCustomProxy ? Number(form.proxyPort.trim()) : null,
    proxyUsername: isCustomProxy ? form.proxyUsername.trim() || null : null,
    proxyPassword: isCustomProxy ? form.proxyPassword || (keepsProxyPassword ? channel?.proxyPassword : '') : '',
  });

  let proxyPasswordHelp: React.ReactNode = DIALOG.PROXY_CREDENTIALS_HELP;
  if (keepsProxyPassword) {
    proxyPasswordHelp = (
      <>
        {DIALOG.SECRET_SAVED}{' '}
        <Link
          href="#"
          onClick={(event) => {
            event.preventDefault();
            update('removeProxyPassword', true);
          }}
          data-testid="notification-proxy-password-remove"
        >
          {DIALOG.PROXY_PASSWORD_REMOVE}
        </Link>
      </>
    );
  }
  else if (savedProxyPassword && form.removeProxyPassword && sameProxy) {
    proxyPasswordHelp = DIALOG.PROXY_PASSWORD_REMOVED;
  }
  else if (savedProxyPassword && !sameProxy && form.proxyUsername.trim()) {
    proxyPasswordHelp = DIALOG.PROXY_PASSWORD_AGAIN;
  }

  const showServerError = (error: unknown, fallback: string) => {
    const { field, message } = serverError(error, fallback);
    if (field && FIELDS.has(field)) {
      setErrors((current) => ({ ...current, [field]: message }));
    }
    else {
      setFormError(message);
    }
  };

  const handleSave = async () => {
    setFormError(null);
    const found = validate();
    setErrors(found);
    if (Object.keys(found).length > 0) {
      return;
    }
    setSaving(true);
    try {
      const data = payload();
      const saved = channel?.id ? await updateChannel(channel.id, data) : await createChannel(data);
      onSaved(saved);
    }
    catch (error) {
      showServerError(error, LABELS.SAVE_ERROR);
    }
    finally {
      setSaving(false);
    }
  };

  const handleTest = async () => {
    setFormError(null);
    const found = validateConnection();
    setErrors(found);
    if (Object.keys(found).length > 0) {
      return;
    }
    setTest({ status: 'running' });
    try {
      const result = await testUnsavedChannel(payload());
      setTest({ status: 'done', success: result.success, message: result.message ?? undefined });
    }
    catch (error) {
      setTest({ status: 'idle' });
      showServerError(error, LABELS.TEST_FAILURE);
    }
  };

  return (
    <Dialog.Root open onOpenChange={(open) => { if (!open && !saving) onClose(); }}>
      <Dialog.Content maxWidth="600px" data-testid="notification-channel-dialog">
        <Dialog.Title>{channel?.id ? DIALOG.EDIT_TITLE : DIALOG.CREATE_TITLE}</Dialog.Title>
        <Dialog.Description size="2" color="gray" mb="4">
          {LABELS.DESCRIPTION}
        </Dialog.Description>

        <Flex direction="column" gap="4">
          {formError && (
            <Callout.Root color="red" size="1" role="alert" data-testid="notification-form-error">
              <Callout.Icon>
                <XCircle size={16} />
              </Callout.Icon>
              <Callout.Text>{formError}</Callout.Text>
            </Callout.Root>
          )}

          <Field label={DIALOG.NAME_LABEL} htmlFor="notification-name" error={errors.name}>
            <TextField.Root
              id="notification-name"
              value={form.name}
              placeholder={DIALOG.NAME_PLACEHOLDER}
              onChange={(event) => update('name', event.target.value)}
              maxLength={100}
            />
          </Field>

          <Field label={DIALOG.TYPE_LABEL} error={errors.type}>
            <RadioCards.Root
              size="1"
              columns={{ initial: '1', sm: '2' }}
              value={form.type}
              onValueChange={(value) => update('type', value as NotificationChannelType)}
              aria-label={DIALOG.TYPE_LABEL}
              data-testid="notification-type"
            >
              <RadioCards.Item value="telegram" data-testid="notification-type-telegram">
                <Flex gap="2" align="center">
                  <Send size={16} />
                  <Text>{LABELS.TYPES.TELEGRAM}</Text>
                </Flex>
              </RadioCards.Item>
              <RadioCards.Item value="slack" data-testid="notification-type-slack">
                <Flex gap="2" align="center">
                  <Hash size={16} />
                  <Text>{LABELS.TYPES.SLACK}</Text>
                </Flex>
              </RadioCards.Item>
            </RadioCards.Root>
          </Field>

          <ConnectionHelp type={form.type} />

          {isTelegram ? (
            <>
              <Field
                label={DIALOG.BOT_TOKEN_LABEL}
                htmlFor="notification-secret"
                help={keepsSecret ? DIALOG.SECRET_SAVED : undefined}
                error={errors.secret}
              >
                <TextField.Root
                  id="notification-secret"
                  type="password"
                  autoComplete="new-password"
                  value={form.secret}
                  placeholder={keepsSecret ? '••••••••' : '123456789:AAH…'}
                  onChange={(event) => update('secret', event.target.value)}
                />
              </Field>
              <Flex gap="3" direction={{ initial: 'column', sm: 'row' }}>
                <Box flexGrow="1">
                  <Field
                    label={DIALOG.CHAT_ID_LABEL}
                    htmlFor="notification-chat-id"
                    help={DIALOG.CHAT_ID_HELP}
                    error={errors.chatId}
                  >
                    <TextField.Root
                      id="notification-chat-id"
                      value={form.chatId}
                      placeholder="-1001234567890"
                      onChange={(event) => update('chatId', event.target.value)}
                    />
                  </Field>
                </Box>
                <Box flexGrow="1">
                  <Field
                    label={DIALOG.THREAD_LABEL}
                    htmlFor="notification-thread"
                    help={DIALOG.THREAD_HELP}
                    error={errors.messageThreadId}
                  >
                    <TextField.Root
                      id="notification-thread"
                      inputMode="numeric"
                      value={form.thread}
                      onChange={(event) => update('thread', event.target.value)}
                    />
                  </Field>
                </Box>
              </Flex>
            </>
          ) : (
            <Field
              label={DIALOG.WEBHOOK_LABEL}
              htmlFor="notification-secret"
              help={keepsSecret ? DIALOG.SECRET_SAVED : DIALOG.WEBHOOK_HELP}
              error={errors.secret}
            >
              <TextField.Root
                id="notification-secret"
                type="password"
                autoComplete="new-password"
                value={form.secret}
                placeholder={keepsSecret ? '••••••••' : 'https://hooks.slack.com/services/…'}
                onChange={(event) => update('secret', event.target.value)}
              />
            </Field>
          )}

          <Field
            label={DIALOG.CONNECTION_LABEL}
            help={CONNECTION_MODES.find((mode) => mode.value === form.proxyMode)?.help}
            error={errors.proxyMode}
          >
            <Select.Root
              value={form.proxyMode}
              onValueChange={(value) => update('proxyMode', value as NotificationProxyMode)}
            >
              <Select.Trigger aria-label={DIALOG.CONNECTION_LABEL} data-testid="notification-proxy-mode" />
              <Select.Content position="popper">
                {CONNECTION_MODES.map((mode) => (
                  <Select.Item key={mode.value} value={mode.value}>{mode.label}</Select.Item>
                ))}
              </Select.Content>
            </Select.Root>
          </Field>

          {isCustomProxy && (
            <Box
              p="3"
              style={{ border: '1px solid var(--gray-a6)', borderRadius: 'var(--radius-3)' }}
              data-testid="notification-proxy"
            >
              <Flex direction="column" gap="3">
                <Field label={DIALOG.PROXY_TYPE_LABEL} error={errors.proxyType}>
                  <SegmentedControl.Root
                    size="1"
                    value={form.proxyType}
                    onValueChange={(value) => update('proxyType', value as NotificationProxyType)}
                    aria-label={DIALOG.PROXY_TYPE_LABEL}
                    data-testid="notification-proxy-type"
                    style={{ alignSelf: 'flex-start' }}
                  >
                    <SegmentedControl.Item value="http">{DIALOG.PROXY_TYPES.HTTP}</SegmentedControl.Item>
                    <SegmentedControl.Item value="socks5">{DIALOG.PROXY_TYPES.SOCKS5}</SegmentedControl.Item>
                  </SegmentedControl.Root>
                </Field>
                <Flex gap="3" direction={{ initial: 'column', sm: 'row' }}>
                  <Box flexGrow="1">
                    <Field label={DIALOG.PROXY_HOST_LABEL} htmlFor="notification-proxy-host" error={errors.proxyHost}>
                      <TextField.Root
                        id="notification-proxy-host"
                        value={form.proxyHost}
                        placeholder="proxy.example.com"
                        maxLength={255}
                        onChange={(event) => update('proxyHost', event.target.value)}
                      />
                    </Field>
                  </Box>
                  <Box width={{ initial: '100%', sm: '120px' }}>
                    <Field label={DIALOG.PROXY_PORT_LABEL} htmlFor="notification-proxy-port" error={errors.proxyPort}>
                      <TextField.Root
                        id="notification-proxy-port"
                        inputMode="numeric"
                        value={form.proxyPort}
                        placeholder={form.proxyType === 'socks5' ? '1080' : '3128'}
                        onChange={(event) => update('proxyPort', event.target.value)}
                      />
                    </Field>
                  </Box>
                </Flex>
                <Flex gap="3" direction={{ initial: 'column', sm: 'row' }}>
                  <Box flexGrow="1" flexBasis="0">
                    <Field
                      label={DIALOG.PROXY_USERNAME_LABEL}
                      htmlFor="notification-proxy-username"
                      error={errors.proxyUsername}
                    >
                      <TextField.Root
                        id="notification-proxy-username"
                        autoComplete="off"
                        value={form.proxyUsername}
                        maxLength={255}
                        onChange={(event) => update('proxyUsername', event.target.value)}
                      />
                    </Field>
                  </Box>
                  <Box flexGrow="1" flexBasis="0">
                    <Field
                      label={DIALOG.PROXY_PASSWORD_LABEL}
                      htmlFor="notification-proxy-password"
                      help={proxyPasswordHelp}
                      error={errors.proxyPassword}
                    >
                      <TextField.Root
                        id="notification-proxy-password"
                        type="password"
                        autoComplete="new-password"
                        value={form.proxyPassword}
                        placeholder={keepsProxyPassword ? '••••••••' : undefined}
                        maxLength={255}
                        onChange={(event) => update('proxyPassword', event.target.value)}
                      />
                    </Field>
                  </Box>
                </Flex>
              </Flex>
            </Box>
          )}

          <Field label={DIALOG.EVENTS_LABEL} error={errors.events}>
            <Flex direction="column" gap="2" data-testid="notification-events">
              {NOTIFICATION_EVENTS.map((event) => (
                <Text as="label" size="2" key={event}>
                  <Flex gap="2" align="start">
                    <Checkbox
                      checked={form.events.includes(event)}
                      onCheckedChange={(checked) => update('events', toggle(form.events, event, checked === true))}
                      aria-label={EVENT_LABELS[event].label}
                    />
                    <Box>
                      <Text as="div" size="2">{EVENT_LABELS[event].label}</Text>
                      <Text as="div" size="1" color="gray">{EVENT_LABELS[event].help}</Text>
                    </Box>
                  </Flex>
                </Text>
              ))}
            </Flex>
          </Field>

          <Field label={DIALOG.REPOSITORIES_LABEL} help={DIALOG.REPOSITORIES_HELP} error={errors.repositories}>
            <Box
              style={{ border: '1px solid var(--gray-a6)', borderRadius: 'var(--radius-2)' }}
              data-testid="notification-repositories"
            >
              <Box p="2">
                <TextField.Root
                  size="1"
                  value={filter}
                  placeholder={DIALOG.REPOSITORIES_FILTER}
                  aria-label={DIALOG.REPOSITORIES_FILTER}
                  onChange={(event) => setFilter(event.target.value)}
                />
              </Box>
              <ScrollArea type="auto" scrollbars="vertical" style={{ maxHeight: 168 }}>
                <Flex direction="column" gap="1" px="3" pb="2">
                  {repositoryOptions.length === 0 && (
                    <Text size="1" color="gray">{DIALOG.REPOSITORIES_EMPTY}</Text>
                  )}
                  {repositoryOptions.map((repository) => (
                    <Text as="label" size="2" key={repository.name}>
                      <Flex gap="2" align="center">
                        <Checkbox
                          checked={form.repositories.includes(repository.name)}
                          onCheckedChange={(checked) =>
                            update('repositories', toggle(form.repositories, repository.name, checked === true))}
                          aria-label={repository.name}
                        />
                        <Text>{repository.name}</Text>
                        {repository.format && (
                          <Badge size="1" color="gray" variant="soft">{repository.format}</Badge>
                        )}
                      </Flex>
                    </Text>
                  ))}
                </Flex>
              </ScrollArea>
            </Box>
          </Field>

          <Flex gap="4" align="end" wrap="wrap">
            <Box minWidth="200px">
              <Field label={DIALOG.LANGUAGE_LABEL} error={errors.language}>
                <Select.Root
                  value={form.language}
                  onValueChange={(value) => update('language', value as NotificationLanguage)}
                >
                  <Select.Trigger aria-label={DIALOG.LANGUAGE_LABEL} data-testid="notification-language" />
                  <Select.Content position="popper">
                    <Select.Item value="ru">{LABELS.LANGUAGES.RUSSIAN}</Select.Item>
                    <Select.Item value="en">{LABELS.LANGUAGES.ENGLISH}</Select.Item>
                  </Select.Content>
                </Select.Root>
              </Field>
            </Box>
            <Text as="label" size="2">
              <Flex gap="2" align="center" pb="1">
                <Switch
                  checked={form.enabled}
                  onCheckedChange={(checked) => update('enabled', checked)}
                  data-testid="notification-enabled"
                />
                {DIALOG.ENABLED_LABEL}
              </Flex>
            </Text>
          </Flex>

          {test.status === 'done' && (
            <Callout.Root
              color={test.success ? 'green' : 'red'}
              size="1"
              role="status"
              data-testid="notification-test-result"
            >
              <Callout.Icon>{test.success ? <CheckCircle2 size={16} /> : <XCircle size={16} />}</Callout.Icon>
              <Callout.Text>
                {test.success ? DIALOG.TEST_SUCCESS : `${LABELS.TEST_FAILURE}: ${test.message ?? ''}`}
              </Callout.Text>
            </Callout.Root>
          )}
        </Flex>

        <Flex gap="3" mt="5" justify="between" wrap="wrap">
          <Button
            variant="soft"
            color="gray"
            onClick={handleTest}
            disabled={saving || test.status === 'running'}
            data-testid="notification-test-button"
          >
            {test.status === 'running' && <Spinner size="1" />}
            {test.status === 'running' ? DIALOG.TESTING : DIALOG.TEST_BUTTON}
          </Button>
          <Flex gap="3">
            <Button variant="soft" color="gray" onClick={onClose} disabled={saving}>
              {DIALOG.CANCEL}
            </Button>
            <Button onClick={handleSave} disabled={saving} data-testid="notification-save-button">
              {saving && <Spinner size="1" />}
              {saving ? DIALOG.SAVING : DIALOG.SAVE}
            </Button>
          </Flex>
        </Flex>
      </Dialog.Content>
    </Dialog.Root>
  );
}

export default NotificationChannelDialog;
