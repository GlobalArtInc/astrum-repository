import React from 'react';
import { render, screen, fireEvent, waitFor, within } from '@testing-library/react';
import '@testing-library/jest-dom';
import { Theme } from '@radix-ui/themes';

import { NotificationsPage } from '../NotificationsPage';
import * as api from '../notificationsApi';
import { ToastProvider } from '../../../../../shared/Toast';
import UIStrings from '../../../../../../../constants/UIStrings';
import { NotificationChannel } from '../types';

jest.mock('../notificationsApi', () => {
  const actual = jest.requireActual('../notificationsApi');
  return {
    ...actual,
    listChannels: jest.fn(),
    listRepositories: jest.fn(),
    createChannel: jest.fn(),
    updateChannel: jest.fn(),
    deleteChannel: jest.fn(),
    testChannel: jest.fn(),
    testUnsavedChannel: jest.fn(),
  };
});

jest.mock('../../../../../../../interface/ExtJS', () => ({
  ExtJS: {
    checkPermission: jest.fn().mockReturnValue(true),
  },
}));

const LABELS = UIStrings.NOTIFICATIONS;
const PLACEHOLDER = '#~ASTRUM~PLACEHOLDER~PASSWORD~#';

const mocked = api as jest.Mocked<typeof api>;

const getCheckPermission = () => {
  const { ExtJS } = require('../../../../../../../interface/ExtJS');
  return ExtJS.checkPermission as jest.Mock;
};

const TELEGRAM: NotificationChannel = {
  id: 'c1',
  name: 'Releases',
  type: 'telegram',
  enabled: true,
  chatId: '-1001234567890',
  messageThreadId: null,
  secret: PLACEHOLDER,
  target: '-1001234567890',
  repositories: ['maven-releases'],
  events: ['published', 'deleted'],
  language: 'ru',
};

const SLACK: NotificationChannel = {
  id: 'c2',
  name: 'Builds',
  type: 'slack',
  enabled: false,
  secret: PLACEHOLDER,
  target: 'hooks.slack.com',
  repositories: [],
  events: ['published'],
  language: 'en',
};

const PROXIED: NotificationChannel = {
  ...TELEGRAM,
  id: 'c5',
  name: 'Through proxy',
  proxyMode: 'custom',
  proxyType: 'socks5',
  proxyHost: 'proxy.internal',
  proxyPort: 1080,
  proxyUsername: 'robot',
  proxyPassword: PLACEHOLDER,
};

function renderPage() {
  return render(
    <Theme>
      <ToastProvider>
        <NotificationsPage />
      </ToastProvider>
    </Theme>
  );
}

async function chooseConnection(dialog: HTMLElement, label: string) {
  const trigger = within(dialog).getByTestId('notification-proxy-mode');
  fireEvent.keyDown(trigger, { key: 'ArrowDown' });
  const option = await screen.findByRole('option', { name: label });
  fireEvent.keyDown(option, { key: 'Enter' });
  await waitFor(() => expect(trigger).toHaveTextContent(label));
}

async function editChannel(id: string) {
  fireEvent.click(await screen.findByTestId(`notification-edit-${id}`));
  return screen.findByTestId('notification-channel-dialog');
}

describe('NotificationsPage', () => {
  beforeAll(() => {
    window.HTMLElement.prototype.hasPointerCapture = jest.fn();
    window.HTMLElement.prototype.releasePointerCapture = jest.fn();
    window.HTMLElement.prototype.scrollIntoView = jest.fn();
  });

  beforeEach(() => {
    getCheckPermission().mockReturnValue(true);
    mocked.listChannels.mockResolvedValue([TELEGRAM, SLACK]);
    mocked.listRepositories.mockResolvedValue([
      { name: 'maven-releases', format: 'maven2', type: 'hosted' },
      { name: 'docker-hosted', format: 'docker', type: 'hosted' },
    ]);
  });

  it('lists the channels with their type, target, repositories, events and state', async () => {
    renderPage();

    const telegramRow = await screen.findByTestId('notification-row-c1');
    expect(within(telegramRow).getByText('Releases')).toBeInTheDocument();
    expect(within(telegramRow).getByText(LABELS.TYPES.TELEGRAM)).toBeInTheDocument();
    expect(within(telegramRow).getByText('-1001234567890')).toBeInTheDocument();
    expect(within(telegramRow).getByText('maven-releases')).toBeInTheDocument();
    expect(within(telegramRow).getByText(LABELS.EVENTS.PUBLISHED)).toBeInTheDocument();
    expect(within(telegramRow).getByText(LABELS.EVENTS.DELETED)).toBeInTheDocument();
    expect(within(telegramRow).queryByText(LABELS.EVENTS.UPDATED)).not.toBeInTheDocument();
    expect(screen.getByTestId('notification-toggle-c1')).toHaveAttribute('data-state', 'checked');

    const slackRow = screen.getByTestId('notification-row-c2');
    expect(within(slackRow).getByText(LABELS.TYPES.SLACK)).toBeInTheDocument();
    expect(within(slackRow).getByText('hooks.slack.com')).toBeInTheDocument();
    expect(within(slackRow).getByText(LABELS.ALL_HOSTED)).toBeInTheDocument();
    expect(screen.getByTestId('notification-toggle-c2')).toHaveAttribute('data-state', 'unchecked');
  });

  it('shows an empty state with a button to add the first channel', async () => {
    mocked.listChannels.mockResolvedValue([]);
    renderPage();

    expect(await screen.findByText(LABELS.EMPTY.TITLE)).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: LABELS.CREATE_BUTTON }));
    expect(await screen.findByText(LABELS.DIALOG.CREATE_TITLE)).toBeInTheDocument();
  });

  it('creates a Telegram channel', async () => {
    const created = { ...TELEGRAM, id: 'c3', name: 'Docker images' };
    mocked.createChannel.mockResolvedValue(created);
    renderPage();
    await screen.findByTestId('notification-row-c1');

    fireEvent.click(screen.getByTestId('notification-create-button'));
    const dialog = await screen.findByTestId('notification-channel-dialog');
    expect(within(dialog).getByText(LABELS.DIALOG.TELEGRAM_HELP.TITLE)).toBeInTheDocument();

    fireEvent.change(within(dialog).getByLabelText(LABELS.DIALOG.NAME_LABEL), { target: { value: 'Docker images' } });
    fireEvent.change(within(dialog).getByLabelText(LABELS.DIALOG.BOT_TOKEN_LABEL), {
      target: { value: '123456:ABC-def' },
    });
    fireEvent.change(within(dialog).getByLabelText(LABELS.DIALOG.CHAT_ID_LABEL), { target: { value: '-100777' } });
    fireEvent.change(within(dialog).getByLabelText(LABELS.DIALOG.THREAD_LABEL), { target: { value: '12' } });
    fireEvent.click(within(dialog).getByRole('checkbox', { name: 'docker-hosted' }));
    fireEvent.click(within(dialog).getByRole('checkbox', { name: LABELS.EVENTS.UPDATED }));
    fireEvent.click(within(dialog).getByTestId('notification-save-button'));

    await waitFor(() => expect(mocked.createChannel).toHaveBeenCalledTimes(1));
    expect(mocked.createChannel).toHaveBeenCalledWith({
      id: undefined,
      name: 'Docker images',
      type: 'telegram',
      enabled: true,
      chatId: '-100777',
      messageThreadId: 12,
      secret: '123456:ABC-def',
      repositories: ['docker-hosted'],
      events: ['published', 'deleted'],
      language: 'ru',
      proxyMode: 'global',
      proxyType: null,
      proxyHost: null,
      proxyPort: null,
      proxyUsername: null,
      proxyPassword: '',
    });
    await waitFor(() => expect(screen.queryByTestId('notification-channel-dialog')).not.toBeInTheDocument());
    expect(await screen.findByText(LABELS.SAVE_SUCCESS('Docker images'))).toBeInTheDocument();
    expect(mocked.listChannels).toHaveBeenCalledTimes(2);
  });

  it('creates a Slack-compatible webhook channel', async () => {
    mocked.createChannel.mockResolvedValue({ ...SLACK, id: 'c4', name: 'Mattermost' });
    renderPage();
    await screen.findByTestId('notification-row-c1');

    fireEvent.click(screen.getByTestId('notification-create-button'));
    const dialog = await screen.findByTestId('notification-channel-dialog');
    fireEvent.click(within(dialog).getByRole('radio', { name: LABELS.TYPES.SLACK }));

    expect(await within(dialog).findByText(LABELS.DIALOG.SLACK_HELP.TITLE)).toBeInTheDocument();
    expect(within(dialog).queryByLabelText(LABELS.DIALOG.CHAT_ID_LABEL)).not.toBeInTheDocument();
    fireEvent.change(within(dialog).getByLabelText(LABELS.DIALOG.NAME_LABEL), { target: { value: 'Mattermost' } });
    fireEvent.change(within(dialog).getByLabelText(LABELS.DIALOG.WEBHOOK_LABEL), {
      target: { value: 'https://chat.example.com/hooks/abc' },
    });
    fireEvent.click(within(dialog).getByTestId('notification-save-button'));

    await waitFor(() => expect(mocked.createChannel).toHaveBeenCalledWith(expect.objectContaining({
      name: 'Mattermost',
      type: 'slack',
      chatId: null,
      messageThreadId: null,
      secret: 'https://chat.example.com/hooks/abc',
      repositories: [],
      events: ['published', 'updated', 'deleted'],
    })));
  });

  it('validates the form before saving', async () => {
    renderPage();
    await screen.findByTestId('notification-row-c1');

    fireEvent.click(screen.getByTestId('notification-create-button'));
    const dialog = await screen.findByTestId('notification-channel-dialog');
    fireEvent.click(within(dialog).getByTestId('notification-save-button'));

    expect(await within(dialog).findByText(LABELS.DIALOG.ERRORS.NAME)).toBeInTheDocument();
    expect(within(dialog).getByText(LABELS.DIALOG.ERRORS.CHAT_ID)).toBeInTheDocument();
    expect(within(dialog).getByText(LABELS.DIALOG.ERRORS.BOT_TOKEN)).toBeInTheDocument();
    expect(mocked.createChannel).not.toHaveBeenCalled();
  });

  it('shows field errors returned by the server', async () => {
    mocked.createChannel.mockRejectedValue({
      response: { status: 400, data: { id: 'chatId', message: 'Chat ID must be a number such as -1001234567890' } },
    });
    renderPage();
    await screen.findByTestId('notification-row-c1');

    fireEvent.click(screen.getByTestId('notification-create-button'));
    const dialog = await screen.findByTestId('notification-channel-dialog');
    fireEvent.change(within(dialog).getByLabelText(LABELS.DIALOG.NAME_LABEL), { target: { value: 'X' } });
    fireEvent.change(within(dialog).getByLabelText(LABELS.DIALOG.BOT_TOKEN_LABEL), { target: { value: '1:a' } });
    fireEvent.change(within(dialog).getByLabelText(LABELS.DIALOG.CHAT_ID_LABEL), { target: { value: 'chat' } });
    fireEvent.click(within(dialog).getByTestId('notification-save-button'));

    expect(await within(dialog).findByText('Chat ID must be a number such as -1001234567890')).toBeInTheDocument();
  });

  it('edits a Slack channel and keeps the saved webhook URL when the field is left empty', async () => {
    mocked.updateChannel.mockResolvedValue({ ...SLACK, name: 'CI builds' });
    renderPage();
    await screen.findByTestId('notification-row-c2');

    fireEvent.click(screen.getByTestId('notification-edit-c2'));
    const dialog = await screen.findByTestId('notification-channel-dialog');
    expect(within(dialog).getByText(LABELS.DIALOG.EDIT_TITLE)).toBeInTheDocument();
    expect(within(dialog).getByText(LABELS.DIALOG.SECRET_SAVED)).toBeInTheDocument();
    expect(within(dialog).getByText(LABELS.DIALOG.SLACK_HELP.TITLE)).toBeInTheDocument();

    fireEvent.change(within(dialog).getByLabelText(LABELS.DIALOG.NAME_LABEL), { target: { value: 'CI builds' } });
    fireEvent.click(within(dialog).getByTestId('notification-save-button'));

    await waitFor(() => expect(mocked.updateChannel).toHaveBeenCalledTimes(1));
    expect(mocked.updateChannel).toHaveBeenCalledWith('c2', expect.objectContaining({
      id: 'c2',
      name: 'CI builds',
      type: 'slack',
      chatId: null,
      secret: PLACEHOLDER,
      events: ['published'],
      language: 'en',
    }));
  });

  it('sends a test message with the settings of the form and shows the result', async () => {
    mocked.testUnsavedChannel.mockResolvedValueOnce({ success: false, message: 'Telegram answered 400: chat not found' });
    mocked.testUnsavedChannel.mockResolvedValueOnce({ success: true });
    renderPage();
    await screen.findByTestId('notification-row-c1');

    fireEvent.click(screen.getByTestId('notification-edit-c1'));
    const dialog = await screen.findByTestId('notification-channel-dialog');
    fireEvent.click(within(dialog).getByTestId('notification-test-button'));

    const failure = await within(dialog).findByTestId('notification-test-result');
    expect(failure).toHaveTextContent(`${LABELS.TEST_FAILURE}: Telegram answered 400: chat not found`);
    expect(mocked.testUnsavedChannel).toHaveBeenCalledWith(expect.objectContaining({
      id: 'c1',
      chatId: '-1001234567890',
      secret: PLACEHOLDER,
    }));

    fireEvent.change(within(dialog).getByLabelText(LABELS.DIALOG.CHAT_ID_LABEL), { target: { value: '-100555' } });
    fireEvent.click(within(dialog).getByTestId('notification-test-button'));

    await waitFor(() =>
      expect(within(dialog).getByTestId('notification-test-result')).toHaveTextContent(LABELS.DIALOG.TEST_SUCCESS));
  });

  it('sends a test message to a saved channel from the list', async () => {
    mocked.testChannel.mockResolvedValue({ success: true });
    renderPage();
    await screen.findByTestId('notification-row-c1');

    fireEvent.click(screen.getByTestId('notification-test-c1'));

    await waitFor(() => expect(mocked.testChannel).toHaveBeenCalledWith('c1'));
    expect(await screen.findByText(LABELS.TEST_SUCCESS('Releases'))).toBeInTheDocument();
  });

  it('deletes a channel after confirmation', async () => {
    mocked.deleteChannel.mockResolvedValue(undefined);
    renderPage();
    await screen.findByTestId('notification-row-c2');

    fireEvent.click(screen.getByTestId('notification-delete-c2'));
    expect(await screen.findByText(LABELS.DELETE_DIALOG.MESSAGE('Builds'))).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: LABELS.DELETE_DIALOG.CONFIRM }));

    await waitFor(() => expect(mocked.deleteChannel).toHaveBeenCalledWith('c2'));
    await waitFor(() => expect(screen.queryByTestId('notification-row-c2')).not.toBeInTheDocument());
    expect(screen.getByTestId('notification-row-c1')).toBeInTheDocument();
  });

  it('turns a channel on from the list', async () => {
    mocked.updateChannel.mockResolvedValue({ ...SLACK, enabled: true });
    renderPage();
    await screen.findByTestId('notification-row-c2');

    fireEvent.click(screen.getByTestId('notification-toggle-c2'));

    await waitFor(() => expect(mocked.updateChannel).toHaveBeenCalledWith('c2', { ...SLACK, enabled: true }));
    expect(screen.getByTestId('notification-toggle-c2')).toHaveAttribute('data-state', 'checked');
  });

  it('is read only without the permission to change settings', async () => {
    getCheckPermission().mockReturnValue(false);
    renderPage();
    await screen.findByTestId('notification-row-c1');

    expect(screen.queryByTestId('notification-create-button')).not.toBeInTheDocument();
    expect(screen.queryByTestId('notification-edit-c1')).not.toBeInTheDocument();
    expect(screen.getByTestId('notification-toggle-c1')).toBeDisabled();
    expect(mocked.listRepositories).not.toHaveBeenCalled();
  });

  it('reports when the channels cannot be loaded', async () => {
    mocked.listChannels.mockRejectedValue(new Error('boom'));
    renderPage();

    expect(await screen.findByTestId('notifications-load-error')).toHaveTextContent(LABELS.LOAD_ERROR);
  });

  describe('connection', () => {
    const DIALOG = LABELS.DIALOG;

    it('creates a Telegram channel that sends through a SOCKS5 proxy', async () => {
      mocked.createChannel.mockResolvedValue({ ...PROXIED, id: 'c6' });
      renderPage();
      await screen.findByTestId('notification-row-c1');

      fireEvent.click(screen.getByTestId('notification-create-button'));
      const dialog = await screen.findByTestId('notification-channel-dialog');
      expect(within(dialog).getByTestId('notification-proxy-mode')).toHaveTextContent(DIALOG.CONNECTION_MODES.GLOBAL);
      expect(within(dialog).getByText(DIALOG.CONNECTION_HELP.GLOBAL)).toBeInTheDocument();
      expect(within(dialog).queryByTestId('notification-proxy')).not.toBeInTheDocument();

      fireEvent.change(within(dialog).getByLabelText(DIALOG.NAME_LABEL), { target: { value: 'Russia' } });
      fireEvent.change(within(dialog).getByLabelText(DIALOG.BOT_TOKEN_LABEL), { target: { value: '123:abc' } });
      fireEvent.change(within(dialog).getByLabelText(DIALOG.CHAT_ID_LABEL), { target: { value: '-100777' } });
      await chooseConnection(dialog, DIALOG.CONNECTION_MODES.CUSTOM);

      const proxy = within(dialog).getByTestId('notification-proxy');
      expect(within(dialog).getByText(DIALOG.CONNECTION_HELP.CUSTOM)).toBeInTheDocument();
      expect(within(proxy).getByRole('radio', { name: DIALOG.PROXY_TYPES.HTTP })).toHaveAttribute('data-state', 'on');
      fireEvent.click(within(proxy).getByRole('radio', { name: DIALOG.PROXY_TYPES.SOCKS5 }));
      expect(within(proxy).getByRole('radio', { name: DIALOG.PROXY_TYPES.SOCKS5 })).toHaveAttribute('data-state', 'on');
      fireEvent.change(within(proxy).getByLabelText(DIALOG.PROXY_HOST_LABEL), { target: { value: ' [fd00::1] ' } });
      fireEvent.change(within(proxy).getByLabelText(DIALOG.PROXY_PORT_LABEL), { target: { value: '1080' } });
      fireEvent.change(within(proxy).getByLabelText(DIALOG.PROXY_USERNAME_LABEL), { target: { value: ' robot ' } });
      fireEvent.change(within(proxy).getByLabelText(DIALOG.PROXY_PASSWORD_LABEL), { target: { value: ' pass ' } });
      fireEvent.click(within(dialog).getByTestId('notification-save-button'));

      await waitFor(() => expect(mocked.createChannel).toHaveBeenCalledWith(expect.objectContaining({
        name: 'Russia',
        proxyMode: 'custom',
        proxyType: 'socks5',
        proxyHost: 'fd00::1',
        proxyPort: 1080,
        proxyUsername: 'robot',
        proxyPassword: ' pass ',
      })));
    });

    it('validates the proxy before saving or testing', async () => {
      renderPage();
      await screen.findByTestId('notification-row-c1');

      const dialog = await editChannel('c1');
      await chooseConnection(dialog, DIALOG.CONNECTION_MODES.CUSTOM);
      fireEvent.change(within(dialog).getByLabelText(DIALOG.PROXY_PORT_LABEL), { target: { value: '70000' } });
      fireEvent.change(within(dialog).getByLabelText(DIALOG.PROXY_PASSWORD_LABEL), { target: { value: 'x' } });
      fireEvent.click(within(dialog).getByTestId('notification-test-button'));

      expect(await within(dialog).findByText(DIALOG.ERRORS.PROXY_HOST)).toBeInTheDocument();
      expect(within(dialog).getByText(DIALOG.ERRORS.PROXY_PORT)).toBeInTheDocument();
      expect(within(dialog).getByText(DIALOG.ERRORS.PROXY_USERNAME)).toBeInTheDocument();

      fireEvent.change(within(dialog).getByLabelText(DIALOG.PROXY_HOST_LABEL), {
        target: { value: 'http://proxy:3128' },
      });
      fireEvent.click(within(dialog).getByTestId('notification-save-button'));
      expect(await within(dialog).findByText(DIALOG.ERRORS.PROXY_HOST_FORMAT)).toBeInTheDocument();
      expect(mocked.testUnsavedChannel).not.toHaveBeenCalled();
      expect(mocked.updateChannel).not.toHaveBeenCalled();
    });

    it('keeps the saved proxy password when the field is left empty', async () => {
      mocked.listChannels.mockResolvedValue([TELEGRAM, PROXIED]);
      mocked.updateChannel.mockResolvedValue(PROXIED);
      renderPage();

      const dialog = await editChannel('c5');
      const proxy = within(dialog).getByTestId('notification-proxy');
      expect(within(dialog).getByTestId('notification-proxy-mode')).toHaveTextContent(DIALOG.CONNECTION_MODES.CUSTOM);
      expect(within(proxy).getByRole('radio', { name: DIALOG.PROXY_TYPES.SOCKS5 })).toHaveAttribute('data-state', 'on');
      expect(within(proxy).getByLabelText(DIALOG.PROXY_HOST_LABEL)).toHaveValue('proxy.internal');
      expect(within(proxy).getByLabelText(DIALOG.PROXY_PORT_LABEL)).toHaveValue('1080');
      expect(within(proxy).getByLabelText(DIALOG.PROXY_PASSWORD_LABEL)).toHaveValue('');
      expect(within(proxy).getByText(DIALOG.SECRET_SAVED, { exact: false })).toBeInTheDocument();
      fireEvent.click(within(dialog).getByTestId('notification-save-button'));

      await waitFor(() => expect(mocked.updateChannel).toHaveBeenCalledWith('c5', expect.objectContaining({
        proxyMode: 'custom',
        proxyType: 'socks5',
        proxyHost: 'proxy.internal',
        proxyPort: 1080,
        proxyUsername: 'robot',
        proxyPassword: PLACEHOLDER,
      })));
    });

    it('asks for the proxy password again when the proxy changes, and can remove it', async () => {
      mocked.listChannels.mockResolvedValue([PROXIED]);
      mocked.testUnsavedChannel.mockResolvedValue({ success: false, message: 'The proxy rejected the username or password' });
      renderPage();

      const dialog = await editChannel('c5');
      const proxy = within(dialog).getByTestId('notification-proxy');
      fireEvent.change(within(proxy).getByLabelText(DIALOG.PROXY_HOST_LABEL), { target: { value: 'other.internal' } });
      expect(within(proxy).getByText(DIALOG.PROXY_PASSWORD_AGAIN)).toBeInTheDocument();
      fireEvent.click(within(dialog).getByTestId('notification-test-button'));

      await waitFor(() => expect(mocked.testUnsavedChannel).toHaveBeenCalledWith(expect.objectContaining({
        id: 'c5',
        proxyHost: 'other.internal',
        proxyPassword: '',
      })));
      expect(await within(dialog).findByTestId('notification-test-result'))
        .toHaveTextContent('The proxy rejected the username or password');

      fireEvent.change(within(proxy).getByLabelText(DIALOG.PROXY_HOST_LABEL), { target: { value: 'PROXY.internal' } });
      fireEvent.click(within(proxy).getByTestId('notification-proxy-password-remove'));
      expect(within(proxy).getByText(DIALOG.PROXY_PASSWORD_REMOVED)).toBeInTheDocument();
      fireEvent.click(within(dialog).getByTestId('notification-save-button'));

      await waitFor(() => expect(mocked.updateChannel).toHaveBeenCalledWith('c5', expect.objectContaining({
        proxyHost: 'PROXY.internal',
        proxyPassword: '',
      })));
    });

    it('connects directly without proxy settings', async () => {
      mocked.listChannels.mockResolvedValue([PROXIED]);
      mocked.updateChannel.mockResolvedValue({ ...PROXIED, proxyMode: 'none' });
      renderPage();

      const dialog = await editChannel('c5');
      await chooseConnection(dialog, DIALOG.CONNECTION_MODES.NONE);
      expect(within(dialog).queryByTestId('notification-proxy')).not.toBeInTheDocument();
      expect(within(dialog).getByText(DIALOG.CONNECTION_HELP.NONE)).toBeInTheDocument();
      fireEvent.click(within(dialog).getByTestId('notification-save-button'));

      await waitFor(() => expect(mocked.updateChannel).toHaveBeenCalledWith('c5', expect.objectContaining({
        proxyMode: 'none',
        proxyType: null,
        proxyHost: null,
        proxyPort: null,
        proxyUsername: null,
        proxyPassword: '',
      })));
    });

    it('marks channels with their own proxy in the list', async () => {
      mocked.listChannels.mockResolvedValue([TELEGRAM, PROXIED]);
      renderPage();

      const row = await screen.findByTestId('notification-row-c5');
      expect(within(row).getByTestId('notification-proxy-c5')).toHaveTextContent(LABELS.PROXY_BADGE);
      expect(within(screen.getByTestId('notification-row-c1')).queryByText(LABELS.PROXY_BADGE)).not.toBeInTheDocument();
    });

    it('shows proxy errors returned by the server at their field', async () => {
      mocked.listChannels.mockResolvedValue([PROXIED]);
      mocked.updateChannel.mockRejectedValue({
        response: { status: 400, data: { id: 'proxyPassword', message: 'Enter the proxy password again: the proxy has changed' } },
      });
      renderPage();

      const dialog = await editChannel('c5');
      fireEvent.click(within(dialog).getByTestId('notification-save-button'));

      expect(await within(dialog).findByTestId('notification-proxy-password-error'))
        .toHaveTextContent('Enter the proxy password again: the proxy has changed');
    });
  });
});
