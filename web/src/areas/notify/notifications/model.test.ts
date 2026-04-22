import { describe, expect, it } from 'vitest';
import { makeTranslate } from '../../../i18n';
import { type Channel, initialValues, splitRecipients, toRequest, validateChannel } from './model';

const t = makeTranslate('en');

const saved: Channel = {
  id: 'c1',
  name: 'Hooks',
  type: 'webhook',
  enabled: true,
  chatId: null,
  messageThreadId: null,
  recipients: [],
  target: 'hooks.example.com',
  repositories: ['releases'],
  events: ['published'],
  language: 'en',
  proxyMode: 'custom',
  proxyType: 'socks5',
  proxyHost: 'proxy',
  proxyPort: 1080,
  proxyUsername: 'me',
  hasSecret: true,
  hasProxyPassword: true,
  hasSigningSecret: true,
  hasAuthorization: false,
};

describe('notification channel forms', () => {
  it('asks for what each messenger needs', () => {
    const telegram = { ...initialValues(null, 'ru'), name: 'Chat', chatId: 'bad id', messageThreadId: '0' };
    expect(validateChannel(t, telegram, null)).toEqual({
      secret: 'Enter the bot token',
      chatId: 'Enter the chat ID',
      messageThreadId: 'Enter a positive whole number',
    });
    const mail = { ...initialValues(null, 'en'), type: 'email' as const, name: 'Mail', events: [] };
    expect(validateChannel(t, mail, null)).toEqual({
      recipients: 'Enter at least one email address',
      events: 'Choose at least one event',
    });
    expect(validateChannel(t, { ...mail, events: [] }, null, true)).toEqual({
      recipients: 'Enter at least one email address',
    });
  });

  it('keeps stored secrets unless they are replaced or removed', () => {
    const values = initialValues(saved, 'ru');
    expect(values.proxyPort).toBe('1080');
    expect(validateChannel(t, values, saved)).toEqual({});
    expect(toRequest(values, saved)).toEqual({
      id: 'c1',
      name: 'Hooks',
      type: 'webhook',
      enabled: true,
      language: 'en',
      events: ['published'],
      repositories: ['releases'],
      proxyMode: 'custom',
      proxyType: 'socks5',
      proxyHost: 'proxy',
      proxyPort: 1080,
      proxyUsername: 'me',
    });
    const changed = toRequest(
      { ...values, removeSigningSecret: true, proxyPassword: 'pw', authorization: 'Bearer x' },
      saved,
    );
    expect(changed.signingSecret).toBe('');
    expect(changed.proxyPassword).toBe('pw');
    expect(changed.authorization).toBe('Bearer x');
    expect(changed).not.toHaveProperty('secret');
  });

  it('splits recipients on commas, semicolons and spaces', () => {
    expect(splitRecipients(' a@example.com, b@example.com;c@example.com  ')).toEqual([
      'a@example.com',
      'b@example.com',
      'c@example.com',
    ]);
  });
});
