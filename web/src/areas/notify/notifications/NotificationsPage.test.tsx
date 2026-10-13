import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { fakeServer, jsonResponse, renderWithApp } from '../../../test/render';
import { CHANNELS } from './model';
import NotificationsPage from './NotificationsPage';

afterEach(() => {
  vi.unstubAllGlobals();
});

const CHANNEL = {
  id: 'c1',
  name: 'Releases',
  type: 'telegram',
  enabled: true,
  chatId: '-100123',
  messageThreadId: null,
  recipients: [],
  target: '-100123',
  repositories: [],
  events: ['published', 'deleted'],
  language: 'ru',
  proxyMode: 'global',
  proxyType: null,
  proxyHost: null,
  proxyPort: null,
  proxyUsername: null,
  hasSecret: true,
  hasProxyPassword: false,
  hasSigningSecret: false,
  hasAuthorization: false,
};

describe('the notifications page', () => {
  it('invites to add the first channel and creates it', async () => {
    let channels: unknown[] = [];
    const fake = fakeServer({
      [`GET ${CHANNELS}`]: () => jsonResponse(channels),
      'GET /service/rest/v1/repositories': () =>
        jsonResponse([{ name: 'releases', format: 'maven2', type: 'hosted' }]),
      [`POST ${CHANNELS}`]: () => {
        channels = [CHANNEL];
        return jsonResponse(CHANNEL);
      },
    });
    vi.stubGlobal('fetch', fake.fetch);
    renderWithApp(<NotificationsPage />);
    expect(await screen.findByText('No notification channels')).toBeInTheDocument();
    await userEvent.click(screen.getByRole('button', { name: 'Add Channel' }));
    const dialog = await screen.findByRole('dialog', { name: 'New Notification Channel' });
    await userEvent.type(screen.getByLabelText('Name'), 'Releases');
    await userEvent.click(screen.getByRole('button', { name: 'Save' }));
    expect(await screen.findByText('Enter the bot token')).toBeInTheDocument();
    await userEvent.type(screen.getByLabelText('Bot Token'), '123:abc');
    await userEvent.type(screen.getByLabelText('Chat ID'), '-100123');
    await userEvent.click(screen.getByRole('button', { name: 'Save' }));
    await waitFor(() => expect(dialog).not.toBeInTheDocument());
    const post = fake.calls.find((call) => call.method === 'POST');
    expect(post?.body).toMatchObject({
      name: 'Releases',
      type: 'telegram',
      secret: '123:abc',
      chatId: '-100123',
      events: ['published', 'updated', 'deleted'],
      repositories: [],
    });
    expect(await screen.findByRole('cell', { name: '-100123' })).toBeInTheDocument();
    expect(screen.getByText('All hosted repositories')).toBeInTheDocument();
  });
});
