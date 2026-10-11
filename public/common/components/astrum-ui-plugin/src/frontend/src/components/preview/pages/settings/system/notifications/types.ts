/** telegram: a chat through a bot; slack: a Slack-compatible incoming webhook; webhook: JSON for the user's service. */
export type NotificationChannelType = 'telegram' | 'slack' | 'webhook';

export type NotificationEvent = 'published' | 'updated' | 'deleted';

export type NotificationLanguage = 'ru' | 'en';

/** global: the System → HTTP settings; none: a direct connection; custom: the channel's own proxy. */
export type NotificationProxyMode = 'global' | 'none' | 'custom';

export type NotificationProxyType = 'http' | 'socks5';

export interface NotificationChannel {
  id?: string;
  name: string;
  type: NotificationChannelType;
  enabled: boolean;
  chatId?: string | null;
  messageThreadId?: number | null;
  /** Bot token or webhook URL; the server only ever returns a placeholder for a saved one. */
  secret?: string;
  /** Read only: the chat id for Telegram, the webhook host otherwise. */
  target?: string | null;
  repositories: string[];
  events: NotificationEvent[];
  language: NotificationLanguage;
  proxyMode?: NotificationProxyMode;
  proxyType?: NotificationProxyType | null;
  proxyHost?: string | null;
  proxyPort?: number | null;
  proxyUsername?: string | null;
  /** The server only ever returns a placeholder for a saved one; an empty value removes it. */
  proxyPassword?: string;
  /** JSON webhooks: the key of the X-Astrum-Signature HMAC; a placeholder or empty like proxyPassword. */
  signingSecret?: string;
  /** JSON webhooks: the Authorization header; a placeholder or empty like proxyPassword. */
  authorization?: string;
}

export interface NotificationTestResult {
  success: boolean;
  message?: string | null;
}

export interface RepositoryOption {
  name: string;
  format: string;
  type: string;
}

export const NOTIFICATION_EVENTS: NotificationEvent[] = ['published', 'updated', 'deleted'];
