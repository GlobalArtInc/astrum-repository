export type NotificationChannelType = 'telegram' | 'slack';

export type NotificationEvent = 'published' | 'updated' | 'deleted';

export type NotificationLanguage = 'ru' | 'en';

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
