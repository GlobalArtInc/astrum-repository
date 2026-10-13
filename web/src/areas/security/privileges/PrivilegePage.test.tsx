import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { fakeServer, jsonResponse, renderWithApp } from '../../../test/render';
import PrivilegePage from './PrivilegePage';
import PrivilegesPage from './PrivilegesPage';

const ROUTES = (element: React.ReactNode) => [
  { path: '/admin/security/privileges/create/:type', element },
  { path: '/admin/security/privileges/create', element: <p>types</p> },
  { path: '/admin/security/privileges/:name', element },
  { path: '/admin/security/privileges/:name/profile', element: <p>profile</p> },
  { path: '/admin/security/privileges', element: <p>list</p> },
];

afterEach(() => {
  vi.unstubAllGlobals();
});

describe('the privilege wizard', () => {
  it('asks for the name, then for the content, and sends what the type needs', async () => {
    const server = fakeServer({
      'GET /service/rest/v1/repositories': () =>
        jsonResponse([{ name: 'files', format: 'raw', type: 'hosted' }]),
      'POST /service/rest/v1/security/privileges/repository-view': () => new Response(null, { status: 201 }),
      'GET /service/rest/v1/security/privileges': () => jsonResponse([]),
    });
    vi.stubGlobal('fetch', server.fetch);
    const { router } = renderWithApp(null, {
      path: '/admin/security/privileges/create/repository-view',
      routes: ROUTES(<PrivilegePage />),
    });
    expect(
      await screen.findByRole('heading', { name: 'Create Privilege: Repository View' }),
    ).toBeInTheDocument();
    const next = screen.getByRole('button', { name: 'Continue' });
    expect(next).toBeDisabled();
    await userEvent.type(screen.getByRole('textbox', { name: /Name/ }), 'files-read');
    await userEvent.click(next);
    const create = await screen.findByRole('button', { name: 'Create Privilege' });
    expect(create).toBeDisabled();
    await userEvent.click(screen.getByRole('checkbox', { name: 'Browse' }));
    await userEvent.click(screen.getByRole('checkbox', { name: 'Read' }));
    await userEvent.click(create);
    await waitFor(() => expect(router.state.location.pathname).toBe('/admin/security/privileges'));
    const post = server.calls.find((call) => call.method === 'POST');
    expect(post?.body).toEqual({
      name: 'files-read',
      description: '',
      format: '*',
      repository: '*',
      actions: ['BROWSE', 'READ'],
    });
  });

  it('shows the refusal of the server for permissions the caller lacks', async () => {
    const server = fakeServer({
      'POST /service/rest/v1/security/privileges/wildcard': () =>
        jsonResponse({ id: '*', message: 'it gives permissions you do not have' }, 403),
    });
    vi.stubGlobal('fetch', server.fetch);
    renderWithApp(null, {
      path: '/admin/security/privileges/create/wildcard',
      routes: ROUTES(<PrivilegePage />),
    });
    await userEvent.type(await screen.findByRole('textbox', { name: /Name/ }), 'all');
    await userEvent.click(screen.getByRole('button', { name: 'Continue' }));
    await userEvent.type(await screen.findByRole('textbox', { name: /Permission String/ }), 'astrum:*');
    await userEvent.click(screen.getByRole('button', { name: 'Create Privilege' }));
    expect(await screen.findByRole('alert')).toHaveTextContent(
      'The server refused: it gives permissions you do not have',
    );
  });
});

describe('the privilege editor', () => {
  it('shows built-in privileges read only', async () => {
    const server = fakeServer({
      'GET /service/rest/v1/security/privileges/as-all': () =>
        jsonResponse({
          type: 'wildcard',
          name: 'as-all',
          description: 'Every permission',
          readOnly: true,
          pattern: 'astrum:*',
        }),
    });
    vi.stubGlobal('fetch', server.fetch);
    renderWithApp(null, { path: '/admin/security/privileges/as-all', routes: ROUTES(<PrivilegePage />) });
    expect(await screen.findByRole('heading', { name: 'Edit: as-all' })).toBeInTheDocument();
    expect(screen.getByRole('textbox', { name: /Permission String/ })).toBeDisabled();
    expect(screen.queryByRole('button', { name: 'Save' })).toBeNull();
    expect(screen.queryByRole('button', { name: 'Delete' })).toBeNull();
  });

  it('saves a changed description and keeps the name', async () => {
    const server = fakeServer({
      'GET /service/rest/v1/security/privileges/mine': () =>
        jsonResponse({
          type: 'wildcard',
          name: 'mine',
          description: 'Mine',
          readOnly: false,
          pattern: 'astrum:search:read',
        }),
      'PUT /service/rest/v1/security/privileges/wildcard/mine': () => new Response(null, { status: 204 }),
    });
    vi.stubGlobal('fetch', server.fetch);
    renderWithApp(null, { path: '/admin/security/privileges/mine', routes: ROUTES(<PrivilegePage />) });
    const save = await screen.findByRole('button', { name: 'Save' });
    expect(save).toBeDisabled();
    expect(screen.getByRole('textbox', { name: /Name/ })).toBeDisabled();
    await userEvent.type(screen.getByRole('textbox', { name: /Description/ }), ' too');
    await userEvent.click(save);
    await waitFor(() => expect(server.calls.some((call) => call.method === 'PUT')).toBe(true));
    expect(server.calls.find((call) => call.method === 'PUT')?.body).toEqual({
      name: 'mine',
      description: 'Mine too',
      pattern: 'astrum:search:read',
    });
  });
});

describe('the privilege list', () => {
  it('filters by type, shows the old permission spelling and opens profiles', async () => {
    const server = fakeServer({
      'GET /service/rest/v1/security/privileges': () =>
        jsonResponse([
          {
            type: 'wildcard',
            name: 'as-all',
            description: 'Every permission',
            readOnly: true,
            pattern: 'astrum:*',
          },
          {
            type: 'application',
            name: 'mine',
            description: 'Mine',
            readOnly: false,
            domain: 'users',
            actions: ['ADD', 'READ'],
          },
        ]),
    });
    vi.stubGlobal('fetch', server.fetch);
    const { router } = renderWithApp(<PrivilegesPage />, { path: '/admin/security/privileges' });
    expect(await screen.findByText('astrum:users:add,read')).toBeInTheDocument();
    expect(screen.getByText('Privileges 1–2 of 2')).toBeInTheDocument();
    await userEvent.click(screen.getByRole('checkbox', { name: /Wildcard/ }));
    expect(screen.queryByText('astrum:users:add,read')).toBeNull();
    await userEvent.click(screen.getByRole('checkbox', { name: /Wildcard/ }));
    await userEvent.type(screen.getByRole('textbox', { name: /Find privileges/ }), 'users');
    expect(screen.queryByRole('button', { name: 'Open as-all' })).toBeNull();
    await userEvent.click(screen.getByRole('button', { name: 'Open mine' }));
    expect(router.state.location.pathname).toBe('/admin/security/privileges/mine/profile');
  });
});
