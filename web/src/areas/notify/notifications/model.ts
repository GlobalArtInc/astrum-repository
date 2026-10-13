import type { Translate } from '../../../i18n';

export const CHANNELS = '/service/rest/internal/ui/notifications';
export const READ = 'astrum:settings:read';
export const UPDATE = 'astrum:settings:update';

export type ChannelType = 'telegram' | 'slack' | 'webhook' | 'email';
export type ProxyMode = 'global' | 'none' | 'custom';
export type ProxyType = 'http' | 'socks5';
export type EventId = 'published' | 'updated' | 'deleted';
export type Language = 'ru' | 'en';

export const TYPES: ChannelType[] = ['telegram', 'slack', 'webhook', 'email'];
export const EVENTS: EventId[] = ['published', 'updated', 'deleted'];

export interface Channel {
  id: string;
  name: string;
  type: ChannelType;
  enabled: boolean;
  chatId: string | null;
  messageThreadId: number | null;
  recipients: string[];
  target: string | null;
  repositories: string[];
  events: EventId[];
  language: Language;
  proxyMode: ProxyMode;
  proxyType: ProxyType | null;
  proxyHost: string | null;
  proxyPort: number | null;
  proxyUsername: string | null;
  hasSecret: boolean;
  hasProxyPassword: boolean;
  hasSigningSecret: boolean;
  hasAuthorization: boolean;
}

export interface TestResult {
  success: boolean;
  message: string | null;
}

export interface ChannelValues {
  name: string;
  type: ChannelType;
  enabled: boolean;
  secret: string;
  chatId: string;
  messageThreadId: string;
  recipients: string;
  signingSecret: string;
  removeSigningSecret: boolean;
  authorization: string;
  removeAuthorization: boolean;
  proxyMode: ProxyMode;
  proxyType: ProxyType;
  proxyHost: string;
  proxyPort: string;
  proxyUsername: string;
  proxyPassword: string;
  removeProxyPassword: boolean;
  events: EventId[];
  repositories: string[];
  language: Language;
}

export function initialValues(channel: Channel | null, language: Language): ChannelValues {
  return {
    name: channel?.name ?? '',
    type: channel?.type ?? 'telegram',
    enabled: channel?.enabled ?? true,
    secret: '',
    chatId: channel?.chatId ?? '',
    messageThreadId: channel?.messageThreadId ? String(channel.messageThreadId) : '',
    recipients: channel?.recipients.join(', ') ?? '',
    signingSecret: '',
    removeSigningSecret: false,
    authorization: '',
    removeAuthorization: false,
    proxyMode: channel?.proxyMode ?? 'global',
    proxyType: channel?.proxyType ?? 'http',
    proxyHost: channel?.proxyHost ?? '',
    proxyPort: channel?.proxyPort ? String(channel.proxyPort) : '',
    proxyUsername: channel?.proxyUsername ?? '',
    proxyPassword: '',
    removeProxyPassword: false,
    events: channel?.events ?? [...EVENTS],
    repositories: channel?.repositories ?? [],
    language: channel?.language ?? language,
  };
}

const CHAT_ID = /^(-?\d{1,20}|@[A-Za-z][A-Za-z0-9_]{3,31})$/;
const HEADER = /^[\x20-\x7E\t]+$/;
const HOST = /^([A-Za-z0-9_](?:[A-Za-z0-9_.-]*[A-Za-z0-9_])?|\[?[0-9A-Fa-f.]*:[0-9A-Fa-f:.]*\]?)$/;

export function splitRecipients(text: string): string[] {
  return text
    .split(/[\s,;]+/)
    .map((part) => part.trim())
    .filter((part) => part.length > 0);
}

export function validateChannel(
  t: Translate,
  values: ChannelValues,
  existing: Channel | null,
  forTest = false,
): Partial<Record<keyof ChannelValues, string>> {
  const errors: Partial<Record<keyof ChannelValues, string>> = {};
  const keepsSecret = existing?.hasSecret && existing.type === values.type;
  if (!forTest && values.name.trim() === '') {
    errors.name = t('notify.notifications.errors.name');
  }
  if (values.type === 'telegram') {
    if (values.secret.trim() === '' && !keepsSecret) {
      errors.secret = t('notify.notifications.errors.botToken');
    }
    if (!CHAT_ID.test(values.chatId.trim())) {
      errors.chatId = t('notify.notifications.errors.chatId');
    }
    if (values.messageThreadId.trim() !== '' && !/^[1-9]\d*$/.test(values.messageThreadId.trim())) {
      errors.messageThreadId = t('notify.notifications.errors.thread');
    }
  } else if (values.type === 'email') {
    if (splitRecipients(values.recipients).length === 0) {
      errors.recipients = t('notify.notifications.errors.recipients');
    }
  } else if (values.secret.trim() === '' && !keepsSecret) {
    errors.secret = t('notify.notifications.errors.webhook');
  }
  if (
    values.type === 'webhook' &&
    values.authorization.trim() !== '' &&
    !HEADER.test(values.authorization.trim())
  ) {
    errors.authorization = t('notify.notifications.errors.authorization');
  }
  if (values.proxyMode === 'custom') {
    const host = values.proxyHost.trim();
    if (host === '') {
      errors.proxyHost = t('notify.notifications.errors.proxyHost');
    } else if (!HOST.test(host)) {
      errors.proxyHost = t('notify.notifications.errors.proxyHostFormat');
    }
    const port = Number(values.proxyPort);
    if (!/^\d+$/.test(values.proxyPort.trim()) || port < 1 || port > 65535) {
      errors.proxyPort = t('notify.notifications.errors.proxyPort');
    }
    if (values.proxyPassword !== '' && values.proxyUsername.trim() === '') {
      errors.proxyUsername = t('notify.notifications.errors.proxyUsername');
    }
  }
  if (!forTest && values.events.length === 0) {
    errors.events = t('notify.notifications.errors.events');
  }
  return errors;
}

export function toRequest(values: ChannelValues, existing: Channel | null): Record<string, unknown> {
  const body: Record<string, unknown> = {
    id: existing?.id,
    name: values.name.trim(),
    type: values.type,
    enabled: values.enabled,
    language: values.language,
    events: EVENTS.filter((event) => values.events.includes(event)),
    repositories: values.repositories,
    proxyMode: values.proxyMode,
  };
  if (values.secret.trim() !== '') {
    body.secret = values.secret.trim();
  }
  if (values.type === 'telegram') {
    body.chatId = values.chatId.trim();
    body.messageThreadId = values.messageThreadId.trim() === '' ? null : Number(values.messageThreadId);
  }
  if (values.type === 'email') {
    body.recipients = splitRecipients(values.recipients);
  }
  if (values.type === 'webhook') {
    if (values.signingSecret !== '') {
      body.signingSecret = values.signingSecret;
    } else if (values.removeSigningSecret) {
      body.signingSecret = '';
    }
    if (values.authorization.trim() !== '') {
      body.authorization = values.authorization.trim();
    } else if (values.removeAuthorization) {
      body.authorization = '';
    }
  }
  if (values.proxyMode === 'custom') {
    body.proxyType = values.proxyType;
    body.proxyHost = values.proxyHost.trim();
    body.proxyPort = Number(values.proxyPort);
    body.proxyUsername = values.proxyUsername.trim();
    if (values.proxyPassword !== '') {
      body.proxyPassword = values.proxyPassword;
    } else if (values.removeProxyPassword) {
      body.proxyPassword = '';
    }
  }
  return body;
}

export function fromChannel(channel: Channel, enabled: boolean): Record<string, unknown> {
  return {
    name: channel.name,
    type: channel.type,
    enabled,
    chatId: channel.chatId,
    messageThreadId: channel.messageThreadId,
    recipients: channel.recipients,
    repositories: channel.repositories,
    events: channel.events,
    language: channel.language,
    proxyMode: channel.proxyMode,
    proxyType: channel.proxyType,
    proxyHost: channel.proxyHost,
    proxyPort: channel.proxyPort,
    proxyUsername: channel.proxyUsername,
  };
}

export const EXAMPLE_BODY = `{
  "event": "component.published",
  "deliveryId": "0b4f7c9e-3d61-4a8e-9c1e-6f2d1a7b5e40",
  "timestamp": "2026-10-11T08:15:30.123Z",
  "repository": {"name": "maven-releases", "format": "maven2", "type": "hosted"},
  "component": {
    "id": "bWF2ZW4tcmVsZWFzZXM6MmE",
    "group": "org.example",
    "name": "demo",
    "version": "1.0.0",
    "format": "maven2",
    "coordinates": "org.example:demo:1.0.0"
  },
  "initiator": "admin",
  "url": "https://repo.example.com/search/component/maven2%3Aorg.example%3Ademo?version=1.0.0",
  "dockerPull": null,
  "text": "📦 *Component published*\\n\`org.example:demo:1.0.0\`\\n…"
}`;
