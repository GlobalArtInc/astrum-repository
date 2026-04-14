import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { fakeServer, jsonResponse, renderWithApp } from '../../../test/render';
import DefaultRolePage from './DefaultRolePage';

const PATH = '/service/rest/internal/ui/default-role';
const ROLES = [
  { id: 'viewers', source: 'default', name: 'Viewers', description: '', readOnly: false },
  { id: 'as-admin', source: 'default', name: 'as-admin', description: '', readOnly: true },
];

afterEach(() => {
  vi.unstubAllGlobals();
});

function serve(current: unknown) {
  const fake = fakeServer({
    [`GET ${PATH}`]: () => jsonResponse(current),
    'GET /service/rest/v1/security/roles': () => jsonResponse(ROLES),
    [`PUT ${PATH}`]: () => new Response(null, { status: 204 }),
    [`DELETE ${PATH}`]: () => new Response(null, { status: 204 }),
  });
  vi.stubGlobal('fetch', fake.fetch);
  return fake;
}

describe('the default role page', () => {
  it('grants a chosen role to every signed-in user', async () => {
    const fake = serve({ role: null, enabled: false, missing: false });
    renderWithApp(<DefaultRolePage />);
    const save = await screen.findByRole('button', { name: 'Save' });
    expect(screen.getByRole('checkbox', { name: 'Enable this capability' })).toBeChecked();
    expect(screen.queryByRole('button', { name: 'Remove' })).not.toBeInTheDocument();
    await userEvent.click(save);
    expect(await screen.findByText('Select a role')).toBeInTheDocument();
    await userEvent.click(screen.getByRole('combobox', { name: 'Role' }));
    await userEvent.click(await screen.findByRole('option', { name: 'Viewers (viewers)' }));
    await userEvent.click(save);
    await waitFor(() =>
      expect(fake.calls.find((call) => call.method === 'PUT')?.body).toEqual({
        role: 'viewers',
        enabled: true,
      }),
    );
    expect(await screen.findByText('Default role saved')).toBeInTheDocument();
  });

  it('warns about a role that is gone and removes the setting', async () => {
    const fake = serve({ role: 'gone', enabled: true, missing: true });
    renderWithApp(<DefaultRolePage />);
    expect(
      await screen.findByText('The role gone no longer exists, so nobody receives it. Select another role.'),
    ).toBeInTheDocument();
    await userEvent.click(screen.getByRole('button', { name: 'Remove' }));
    const dialog = await screen.findByRole('alertdialog', { name: 'Remove the default role?' });
    await userEvent.click(dialog.querySelector('button.rt-variant-solid') as HTMLElement);
    expect(await screen.findByText('Default role removed')).toBeInTheDocument();
    expect(fake.calls.some((call) => call.method === 'DELETE')).toBe(true);
  });
});
