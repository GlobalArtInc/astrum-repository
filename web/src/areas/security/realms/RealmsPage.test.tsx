import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { fakeServer, jsonResponse, renderWithApp } from '../../../test/render';
import RealmsPage from './RealmsPage';

const AVAILABLE = [
  { id: 'AccessTokenRealm', name: 'Access Token Realm' },
  { id: 'DockerToken', name: 'Docker Bearer Token Realm' },
  { id: 'AstrumAuthenticatingRealm', name: 'Local Authenticating Realm' },
];

afterEach(() => {
  vi.unstubAllGlobals();
});

function serve() {
  const server = fakeServer({
    'GET /service/rest/v1/security/realms/available': () => jsonResponse(AVAILABLE),
    'GET /service/rest/v1/security/realms/active': () =>
      jsonResponse(['AstrumAuthenticatingRealm', 'AccessTokenRealm']),
    'PUT /service/rest/v1/security/realms/active': () => new Response(null, { status: 204 }),
  });
  vi.stubGlobal('fetch', server.fetch);
  return server;
}

function names(list: HTMLElement) {
  return within(list)
    .getAllByRole('option')
    .map((option) => option.textContent);
}

describe('the realms page', () => {
  it('reorders and activates realms with the buttons and saves the order', async () => {
    const server = serve();
    renderWithApp(<RealmsPage />);
    const save = await screen.findByRole('button', { name: 'Save' });
    expect(save).toBeDisabled();
    const active = screen.getByRole('listbox', { name: 'Active realms' });
    const available = screen.getByRole('listbox', { name: 'Available realms' });
    expect(names(active)).toEqual(['Local Authenticating Realm1', 'Access Token Realm2']);
    expect(names(available)).toEqual(['Docker Bearer Token Realm']);
    expect(screen.getByRole('button', { name: 'Move the selected realm up' })).toBeDisabled();
    await userEvent.click(within(active).getByRole('option', { name: /Access Token Realm/ }));
    await userEvent.click(screen.getByRole('button', { name: 'Move the selected realm up' }));
    await userEvent.click(within(available).getByRole('option', { name: 'Docker Bearer Token Realm' }));
    await userEvent.click(screen.getByRole('button', { name: 'Activate the selected realm' }));
    expect(names(active)).toEqual([
      'Access Token Realm1',
      'Local Authenticating Realm2',
      'Docker Bearer Token Realm3',
    ]);
    expect(screen.getByText('0 realms')).toBeInTheDocument();
    await userEvent.click(save);
    await waitFor(() =>
      expect(server.calls.find((call) => call.method === 'PUT')?.body).toEqual([
        'AccessTokenRealm',
        'AstrumAuthenticatingRealm',
        'DockerToken',
      ]),
    );
    expect(await screen.findByText('The realms are saved')).toBeInTheDocument();
  });

  it('moves a realm on double click and filters a list', async () => {
    serve();
    renderWithApp(<RealmsPage />);
    const active = await screen.findByRole('listbox', { name: 'Active realms' });
    await userEvent.dblClick(within(active).getByRole('option', { name: /Access Token Realm/ }));
    const available = screen.getByRole('listbox', { name: 'Available realms' });
    expect(names(available)).toEqual(['Access Token Realm', 'Docker Bearer Token Realm']);
    await userEvent.type(screen.getByRole('textbox', { name: 'Filter Available realms' }), 'docker');
    expect(names(available)).toEqual(['Docker Bearer Token Realm']);
    await userEvent.click(screen.getByRole('button', { name: 'Discard Changes' }));
    expect(names(active)).toEqual(['Local Authenticating Realm1', 'Access Token Realm2']);
  });

  it('asks before saving without local users and refuses an empty list', async () => {
    const server = serve();
    renderWithApp(<RealmsPage />);
    const active = await screen.findByRole('listbox', { name: 'Active realms' });
    await userEvent.click(within(active).getByRole('option', { name: /Local Authenticating Realm/ }));
    await userEvent.click(screen.getByRole('button', { name: 'Deactivate the selected realm' }));
    await userEvent.click(screen.getByRole('button', { name: 'Save' }));
    const dialog = await screen.findByRole('alertdialog', { name: 'Deactivate local users?' });
    await userEvent.click(within(dialog).getByRole('button', { name: 'Cancel' }));
    expect(server.calls.some((call) => call.method === 'PUT')).toBe(false);
    await userEvent.click(screen.getByRole('button', { name: 'Deactivate all realms' }));
    await userEvent.click(screen.getByRole('button', { name: 'Save' }));
    expect(screen.getByRole('alert')).toHaveTextContent('Keep at least one realm active');
  });
});
