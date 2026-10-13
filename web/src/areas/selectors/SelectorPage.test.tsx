import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { fakeServer, jsonResponse, renderWithApp } from '../../test/render';
import SelectorPage from './SelectorPage';
import SelectorsPage from './SelectorsPage';

const ROUTES = [
  { path: '/admin/repository/selectors/create', element: <SelectorPage /> },
  { path: '/admin/repository/selectors/:name', element: <SelectorPage /> },
  { path: '/admin/repository/selectors', element: <SelectorsPage /> },
];

afterEach(() => {
  vi.unstubAllGlobals();
});

describe('the content selector editor', () => {
  it('checks and explains the expression and creates the selector', async () => {
    const server = fakeServer({
      'GET /service/rest/v1/repositories': () =>
        jsonResponse([{ name: 'files', format: 'raw', type: 'hosted' }]),
      'POST /service/rest/internal/ui/content-selectors/preview': ({ body }) => {
        const { expression } = body as { expression: string };
        return expression.endsWith('"raw"')
          ? jsonResponse({ items: [], more: false })
          : jsonResponse([{ id: 'expression', message: 'expected a string after ==' }], 400);
      },
      'POST /service/rest/v1/security/content-selectors': () => new Response(null, { status: 204 }),
      'GET /service/rest/v1/security/content-selectors': () => jsonResponse([]),
    });
    vi.stubGlobal('fetch', server.fetch);
    const { router } = renderWithApp(null, { path: '/admin/repository/selectors/create', routes: ROUTES });
    await userEvent.type(await screen.findByRole('textbox', { name: /^Name/ }), 'raw-only');
    const expression = screen.getByRole('textbox', { name: /Expression/ });
    await userEvent.type(expression, 'format ==');
    expect(await screen.findByText(/expected a string after ==/, {}, { timeout: 3000 })).toBeInTheDocument();
    await userEvent.type(expression, ' "raw"');
    expect(await screen.findByText('The expression is valid', {}, { timeout: 3000 })).toBeInTheDocument();
    expect(screen.getByText('Content matches when: format is "raw"')).toBeInTheDocument();
    await userEvent.click(screen.getByRole('button', { name: 'Create' }));
    await waitFor(() => expect(router.state.location.pathname).toBe('/admin/repository/selectors'));
    expect(
      server.calls.find(
        (call) => call.method === 'POST' && call.path === '/service/rest/v1/security/content-selectors',
      )?.body,
    ).toEqual({
      name: 'raw-only',
      description: '',
      expression: 'format == "raw"',
    });
  }, 20_000);

  it('names the privileges that use a selector and does not offer to delete it', async () => {
    const server = fakeServer({
      'GET /service/rest/v1/security/content-selectors/team': () =>
        jsonResponse({ name: 'team', type: 'csel', description: 'Team', expression: 'format == "raw"' }),
      'GET /service/rest/v1/security/privileges': () =>
        jsonResponse([
          {
            type: 'repository-content-selector',
            name: 'team-read',
            description: '',
            readOnly: false,
            contentSelector: 'team',
            format: '*',
            repository: '*',
            actions: ['READ'],
          },
        ]),
      'POST /service/rest/internal/ui/content-selectors/preview': () =>
        jsonResponse({ items: [], more: false }),
      'GET /service/rest/v1/repositories': () => jsonResponse([]),
    });
    vi.stubGlobal('fetch', server.fetch);
    renderWithApp(null, { path: '/admin/repository/selectors/team', routes: ROUTES });
    expect(await screen.findByText('team-read')).toBeInTheDocument();
    expect(screen.getByText('The selector is used by 1 privilege:')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Delete' })).toBeDisabled();
  });

  it('lists the selectors and opens one with a click', async () => {
    const server = fakeServer({
      'GET /service/rest/v1/security/content-selectors': () =>
        jsonResponse([{ name: 'team', type: 'csel', description: 'Team', expression: 'format == "raw"' }]),
    });
    vi.stubGlobal('fetch', server.fetch);
    const { router } = renderWithApp(null, { path: '/admin/repository/selectors', routes: ROUTES });
    await userEvent.click(await screen.findByRole('button', { name: 'Open team' }));
    expect(router.state.location.pathname).toBe('/admin/repository/selectors/team');
  });
});
