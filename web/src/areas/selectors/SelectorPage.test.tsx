import { screen, waitFor, within } from '@testing-library/react';
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
  it('previews as you type, shows problems and creates the selector', async () => {
    const server = fakeServer({
      'GET /service/rest/v1/repositories': () =>
        jsonResponse([{ name: 'files', format: 'raw', type: 'hosted' }]),
      'POST /service/rest/internal/ui/content-selectors/preview': ({ body }) => {
        const { expression } = body as { expression: string };
        return expression.endsWith('"raw"')
          ? jsonResponse({
              items: [{ repository: 'files', format: 'raw', path: '/docs/a.txt', assetId: 'x1' }],
              more: true,
            })
          : jsonResponse([{ id: 'expression', message: 'expected a string after ==' }], 400);
      },
      'POST /service/rest/v1/security/content-selectors': () => new Response(null, { status: 204 }),
      'GET /service/rest/v1/security/content-selectors/raw-only': () =>
        jsonResponse({ name: 'raw-only', type: 'csel', description: null, expression: 'format == "raw"' }),
    });
    vi.stubGlobal('fetch', server.fetch);
    const { router } = renderWithApp(null, { path: '/admin/repository/selectors/create', routes: ROUTES });
    await userEvent.type(await screen.findByRole('textbox', { name: /Name/ }), 'raw-only');
    const expression = screen.getByRole('textbox', { name: /Expression/ });
    await userEvent.click(screen.getByRole('button', { name: /^Insert format/ }));
    await userEvent.click(screen.getByRole('button', { name: /^Insert ==/ }));
    expect(expression).toHaveValue('format ==');
    expect(
      await screen.findByText(/Not valid yet: expected a string after ==/, {}, { timeout: 3000 }),
    ).toBeInTheDocument();
    await userEvent.type(expression, ' "raw"');
    expect(await screen.findByText('/docs/a.txt', {}, { timeout: 3000 })).toBeInTheDocument();
    expect(screen.getByText('The expression is valid')).toBeInTheDocument();
    expect(screen.getByText(/Only the first 1 match is shown/)).toBeInTheDocument();
    expect(screen.getByRole('link', { name: '/docs/a.txt' })).toHaveAttribute(
      'href',
      '/browse/files/docs/a.txt',
    );
    await userEvent.click(screen.getByRole('button', { name: 'Create selector' }));
    await waitFor(() => expect(router.state.location.pathname).toBe('/admin/repository/selectors/raw-only'));
    expect(
      server.calls.find((call) => call.path === '/service/rest/v1/security/content-selectors')?.body,
    ).toEqual({
      name: 'raw-only',
      description: '',
      expression: 'format == "raw"',
    });
  }, 20_000);

  it('shows why the server refuses to delete a selector in use', async () => {
    const server = fakeServer({
      'GET /service/rest/v1/security/content-selectors': () =>
        jsonResponse([{ name: 'team', type: 'csel', description: 'Team', expression: 'format == "raw"' }]),
      'DELETE /service/rest/v1/security/content-selectors/team': () =>
        jsonResponse(
          [{ id: '*', message: 'the content selector team is used by the privilege(s) team-read' }],
          400,
        ),
    });
    vi.stubGlobal('fetch', server.fetch);
    renderWithApp(null, { path: '/admin/repository/selectors', routes: ROUTES });
    expect(await screen.findByRole('link', { name: 'team' })).toBeInTheDocument();
    await userEvent.click(screen.getByRole('button', { name: 'Actions' }));
    await userEvent.click(await screen.findByRole('menuitem', { name: 'Delete' }));
    const dialog = await screen.findByRole('alertdialog');
    await userEvent.click(within(dialog).getByRole('button', { name: 'Delete' }));
    expect(await within(dialog).findByRole('alert')).toHaveTextContent('used by the privilege(s) team-read');
  });
});
