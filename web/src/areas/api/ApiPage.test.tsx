import { configure, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { fakeServer, jsonResponse, renderWithApp } from '../../test/render';
import ApiPage from './ApiPage';
import { DOC } from './fixture';

vi.setConfig({ testTimeout: 30_000 });
configure({ asyncUtilTimeout: 5_000 });

afterEach(() => {
  vi.unstubAllGlobals();
});

function serve(handlers: Parameters<typeof fakeServer>[0] = {}) {
  const server = fakeServer({ 'GET /service/rest/swagger.json': () => jsonResponse(DOC), ...handlers });
  vi.stubGlobal('fetch', server.fetch);
  return server;
}

function page(path = '/api') {
  return renderWithApp(null, { path, routes: [{ path: '/api', element: <ApiPage /> }] });
}

describe('the API page', () => {
  it('lists endpoints grouped by tag and explains how to start', async () => {
    serve();
    page();
    const list = await screen.findByRole('navigation', { name: 'Endpoints' });
    expect(await within(list).findByText('/service/rest/v1/status')).toBeInTheDocument();
    const tags = within(list)
      .getAllByText(/^(assets|Blob store|Status)$/)
      .map((element) => element.textContent);
    expect(tags).toEqual(['assets', 'Blob store', 'Status']);
    expect(screen.getByRole('heading', { level: 1, name: 'API' })).toBeInTheDocument();
    expect(screen.getByText('Choose an endpoint')).toBeInTheDocument();
    expect(screen.getByText('About the API Documentation')).toBeInTheDocument();
  });

  it('filters by text and method', async () => {
    const user = userEvent.setup();
    serve();
    page();
    const list = await screen.findByRole('navigation', { name: 'Endpoints' });
    await within(list).findByText('/service/rest/v1/status');
    await user.type(screen.getByRole('textbox', { name: 'Search by path or summary…' }), 'blob');
    expect(within(list).queryByText('/service/rest/v1/status')).not.toBeInTheDocument();
    expect(within(list).getAllByRole('button')).toHaveLength(4);
    await user.click(screen.getByRole('combobox', { name: 'Method' }));
    await user.click(await screen.findByRole('option', { name: 'DELETE' }));
    await waitFor(() => expect(within(list).getAllByRole('button')).toHaveLength(1));
    expect(within(list).getByText('/service/rest/v1/blobstores/{name}')).toBeInTheDocument();
  });

  it('opens an endpoint from the address and executes it', async () => {
    const user = userEvent.setup();
    const server = serve({
      'GET /service/rest/v1/blobstores/my%20store': () => jsonResponse({ name: 'my store', type: 'File' }),
    });
    page('/api?method=GET&path=%2Fv1%2Fblobstores%2F%7Bname%7D');
    expect(
      await screen.findByRole('heading', { name: '/service/rest/v1/blobstores/{name}' }),
    ).toBeInTheDocument();
    expect(screen.getByRole('tab', { selected: true })).toHaveTextContent('Try It');
    await user.click(screen.getByRole('button', { name: 'Execute' }));
    expect(await screen.findByRole('alert')).toHaveTextContent('Fill in the required parameters: name');
    await user.type(screen.getByRole('textbox', { name: 'name' }), 'my store');
    await user.click(screen.getByRole('button', { name: 'Execute' }));
    expect(await screen.findByText('200 OK', { exact: false })).toBeInTheDocument();
    expect(screen.getByText(/"type": "File"/)).toBeInTheDocument();
    const call = server.calls.find((entry) => entry.path.includes('my%20store'));
    expect(call?.method).toBe('GET');
  });

  it('selects an endpoint from the list and sends its JSON body', async () => {
    const user = userEvent.setup();
    const server = serve({
      'POST /service/rest/v1/blobstores/file': () => new Response(null, { status: 204 }),
    });
    page();
    const list = await screen.findByRole('navigation', { name: 'Endpoints' });
    await user.click(await within(list).findByText('/service/rest/v1/blobstores/file'));
    const body = await screen.findByRole('textbox', { name: /Request body/ });
    expect((body as HTMLTextAreaElement).value).toContain('"name": "default"');
    expect(screen.getByText('Responses')).toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: 'Execute' }));
    expect(await screen.findByText('The response has no body')).toBeInTheDocument();
    const call = server.calls.find((entry) => entry.method === 'POST');
    expect(call?.body).toEqual({ name: 'default', path: '', softQuota: { limit: 0 } });
  });
});
