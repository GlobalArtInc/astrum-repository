import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { fakeServer, jsonResponse, renderWithApp } from '../../../test/render';
import PrivilegePage from './PrivilegePage';
import PrivilegesPage from './PrivilegesPage';

const TYPES = [
  {
    id: 'repository-view',
    fields: [
      { id: 'name', kind: 'text', required: true, any: false },
      { id: 'description', kind: 'text', required: false, any: false },
      { id: 'format', kind: 'format', required: true, values: ['*', 'raw', 'maven2'], any: true },
      { id: 'repository', kind: 'repository', required: true, any: true },
      {
        id: 'actions',
        kind: 'actions',
        required: true,
        values: ['BROWSE', 'READ', 'EDIT', 'ADD', 'DELETE', 'ALL'],
        any: false,
      },
    ],
  },
  {
    id: 'wildcard',
    fields: [
      { id: 'name', kind: 'text', required: true, any: false },
      { id: 'description', kind: 'text', required: false, any: false },
      { id: 'pattern', kind: 'permission', required: true, any: false },
    ],
  },
];

const ROUTES = (element: React.ReactNode) => [
  { path: '/admin/security/privileges/create/:type', element },
  { path: '/admin/security/privileges/:name', element },
  { path: '/admin/security/privileges', element: <p>list</p> },
];

afterEach(() => {
  vi.unstubAllGlobals();
});

describe('the privilege form', () => {
  it('is built from the type and sends what the type needs', async () => {
    const server = fakeServer({
      'GET /service/rest/internal/ui/privileges/types': () => jsonResponse(TYPES),
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
      await screen.findByRole('heading', { name: 'Create a privilege: Repository view' }),
    ).toBeInTheDocument();
    await userEvent.click(await screen.findByRole('button', { name: 'Create privilege' }));
    expect(screen.queryByLabelText(/Permission/)).toBeNull();
    expect(screen.getAllByText('Fill this in')).toHaveLength(1);
    expect(screen.getByText('Choose at least one action')).toBeInTheDocument();
    await userEvent.type(screen.getByRole('textbox', { name: /Name/ }), 'files-read');
    await userEvent.click(screen.getByRole('checkbox', { name: 'Browse' }));
    await userEvent.click(screen.getByRole('checkbox', { name: 'Read' }));
    expect(screen.getByText('astrum:repository-view:*:*:browse,read')).toBeInTheDocument();
    await userEvent.click(screen.getByRole('checkbox', { name: 'All' }));
    expect(screen.getByRole('checkbox', { name: 'Read' })).not.toBeChecked();
    await userEvent.click(screen.getByRole('checkbox', { name: 'Read' }));
    await userEvent.click(screen.getByRole('button', { name: 'Create privilege' }));
    await waitFor(() => expect(router.state.location.pathname).toBe('/admin/security/privileges'));
    const post = server.calls.find((call) => call.method === 'POST');
    expect(post?.body).toEqual({
      name: 'files-read',
      description: '',
      format: '*',
      repository: '*',
      actions: ['READ'],
    });
  });

  it('shows the refusal of the server for permissions the caller lacks', async () => {
    const server = fakeServer({
      'GET /service/rest/internal/ui/privileges/types': () => jsonResponse(TYPES),
      'POST /service/rest/v1/security/privileges/wildcard': () =>
        jsonResponse({ id: '*', message: 'it gives permissions you do not have' }, 403),
    });
    vi.stubGlobal('fetch', server.fetch);
    renderWithApp(null, {
      path: '/admin/security/privileges/create/wildcard',
      routes: ROUTES(<PrivilegePage />),
    });
    await userEvent.type(await screen.findByRole('textbox', { name: /Name/ }), 'all');
    await userEvent.type(screen.getByRole('textbox', { name: /Permission/ }), 'astrum:*');
    await userEvent.click(screen.getByRole('button', { name: 'Create privilege' }));
    expect(await screen.findByRole('alert')).toHaveTextContent(
      'The server refused: it gives permissions you do not have',
    );
  });

  it('shows built-in privileges read only', async () => {
    const server = fakeServer({
      'GET /service/rest/internal/ui/privileges/types': () => jsonResponse(TYPES),
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
    expect(await screen.findByText(/This privilege is built in/)).toBeInTheDocument();
    expect(screen.getByRole('textbox', { name: /Permission/ })).toBeDisabled();
    expect(screen.queryByRole('button', { name: 'Save' })).toBeNull();
    expect(screen.queryByRole('button', { name: 'Delete' })).toBeNull();
  });
});

describe('the privilege list', () => {
  it('links rows, shows permissions and hides delete for built-in privileges', async () => {
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
            actions: ['READ'],
          },
        ]),
    });
    vi.stubGlobal('fetch', server.fetch);
    renderWithApp(<PrivilegesPage />, { path: '/admin/security/privileges' });
    expect(await screen.findByRole('link', { name: 'mine' })).toHaveAttribute(
      'href',
      '/admin/security/privileges/mine',
    );
    expect(screen.getByText('astrum:users:read')).toBeInTheDocument();
    await userEvent.type(screen.getByRole('searchbox', { name: /Find by name/ }), 'users');
    expect(screen.queryByRole('link', { name: 'as-all' })).toBeNull();
  });
});
