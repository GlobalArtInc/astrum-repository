import { configure, fireEvent, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { ApiError } from '../../../api';
import { fakeServer, jsonResponse, renderWithApp } from '../../../test/render';
import type { UiRepository } from '../api';
import { canonical, getPath, type RepositoryDocument } from '../model/document';
import { template } from '../model/form';
import reference from '../model/reference-documents.json';
import type { Spec } from '../model/spec';
import { RepositoryForm, type RepositoryFormProps } from './RepositoryForm';

vi.setConfig({ testTimeout: 30_000 });
configure({ asyncUtilTimeout: 5_000 });

const DOCUMENTS = Object.entries(reference as unknown as Record<string, RepositoryDocument>);
const BY_KEY = reference as unknown as Record<string, RepositoryDocument>;

function row(name: string, format: string, type: UiRepository['type'], online = true): UiRepository {
  return {
    name,
    format,
    type,
    url: `http://localhost/repository/${name}`,
    online,
    blobStoreName: 'default',
    remoteUrl: null,
    memberOf: [],
    componentCount: 0,
    assetCount: 0,
    totalSize: 0,
    lastUploaded: null,
    limited: false,
    anonymousAccess: false,
    permissions: {
      browse: true,
      read: true,
      add: true,
      edit: true,
      delete: true,
      admin: true,
      adminEdit: true,
      adminDelete: true,
    },
  };
}

function specOf(document: RepositoryDocument): Spec {
  return { format: String(document.format), type: document.type as Spec['type'] };
}

function renderForm(props: Partial<RepositoryFormProps> & { document: RepositoryDocument }) {
  const onSubmit = vi.fn(async (_: RepositoryDocument) => {});
  const result = renderWithApp(
    <RepositoryForm
      spec={specOf(props.document)}
      creating={false}
      repositories={[]}
      blobStores={null}
      submitLabel="Save"
      onSubmit={onSubmit}
      {...props}
    />,
  );
  return { ...result, onSubmit };
}

afterEach(() => {
  vi.unstubAllGlobals();
});

describe('the repository form', () => {
  it.each(DOCUMENTS)('saves %s unchanged when nothing is edited', async (_, document) => {
    const { onSubmit } = renderForm({ document });
    expect(screen.getByRole('button', { name: 'Save' })).toBeDisabled();
    fireEvent.submit(screen.getByRole('form', { name: 'Save' }));
    await waitFor(() => expect(onSubmit).toHaveBeenCalledTimes(1));
    const sent = onSubmit.mock.calls[0]?.[0] as RepositoryDocument;
    expect(canonical(sent)).toEqual(canonical(document));
  });

  it('sends what was changed and puts server problems on their fields', async () => {
    const document = BY_KEY['raw/proxy/max'] as RepositoryDocument;
    const onSubmit = vi.fn(async (_: RepositoryDocument) => {
      throw new ApiError(400, 'invalid', {
        fieldErrors: [
          { id: 'PARAMETER enabled', message: 'must not be null' },
          { id: 'PARAMETER routingRuleName', message: 'the routing rule "x" does not exist' },
        ],
      });
    });
    renderForm({ document, onSubmit });
    const remote = screen.getByRole('textbox', { name: /Remote Storage/ });
    await userEvent.clear(remote);
    await userEvent.type(remote, 'https://other.example.org/');
    await userEvent.click(screen.getByRole('button', { name: 'Save' }));
    await waitFor(() => expect(onSubmit).toHaveBeenCalledTimes(1));
    expect(getPath(onSubmit.mock.calls[0]?.[0] as RepositoryDocument, ['proxy', 'remoteUrl'])).toBe(
      'https://other.example.org/',
    );
    expect(await screen.findByText('must not be null')).toBeInTheDocument();
    expect(screen.getByText('the routing rule "x" does not exist')).toBeInTheDocument();
  });

  it('shows a refused save next to the buttons and as a toast', async () => {
    const document = BY_KEY['raw/proxy/max'] as RepositoryDocument;
    const message = 'Proxy URL blocked: Host resolves to private/local IP address(es): 10.1.2.3';
    const onSubmit = vi.fn(async (_: RepositoryDocument) => {
      throw new ApiError(400, message, { fieldErrors: [{ id: 'PARAMETER remoteUrl', message }] });
    });
    renderForm({ document, onSubmit });
    const remote = screen.getByRole('textbox', { name: /Remote Storage/ });
    await userEvent.clear(remote);
    await userEvent.type(remote, 'http://10.1.2.3/');
    await userEvent.click(screen.getByRole('button', { name: 'Save' }));
    await waitFor(() => expect(onSubmit).toHaveBeenCalledTimes(1));
    const footer = screen.getByRole('button', { name: 'Save' }).parentElement as HTMLElement;
    expect(await within(footer).findByRole('alert')).toHaveTextContent(message);
    expect(screen.getAllByText(message).length).toBeGreaterThanOrEqual(3);
  });

  it('stops at its own checks before asking the server', async () => {
    const document = BY_KEY['maven/proxy/min'] as RepositoryDocument;
    const { onSubmit } = renderForm({ document });
    await userEvent.clear(screen.getByRole('spinbutton', { name: /Negative Cache TTL/ }));
    await userEvent.click(screen.getByRole('button', { name: 'Save' }));
    expect(await screen.findByText('Enter a whole number of minutes, 0 or more')).toBeInTheDocument();
    expect(onSubmit).not.toHaveBeenCalled();
  });

  it('shows settings without letting a reader change them', () => {
    renderForm({ document: BY_KEY['docker/hosted/max'] as RepositoryDocument, readOnly: true });
    expect(screen.getByText('You can read these settings but not change them.')).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'Save' })).not.toBeInTheDocument();
    expect(screen.getByRole('checkbox', { name: /Force Basic Authentication/ })).toBeDisabled();
    expect(screen.getByRole('combobox', { name: /Deployment Policy/ })).toBeDisabled();
    for (const input of screen.getAllByRole('textbox')) {
      expect(input).toBeDisabled();
    }
  });

  it('orders group members with the arrows and offers only repositories of the format', async () => {
    const document = structuredClone(BY_KEY['raw/group/min'] as RepositoryDocument);
    (document.group as Record<string, unknown>).memberNames = ['a', 'b', 'c'];
    const repositories = [
      row('a', 'raw', 'hosted'),
      row('b', 'raw', 'proxy', false),
      row('c', 'raw', 'hosted'),
      row('d', 'raw', 'hosted'),
      row('m', 'maven2', 'hosted'),
      row(String(document.name), 'raw', 'group'),
    ];
    const { onSubmit } = renderForm({ document, repositories });
    const members = screen.getByRole('list', { name: 'Member Repositories' });
    const items = within(members).getAllByRole('listitem');
    expect(items.map((item) => item.textContent)).toEqual([
      expect.stringContaining('a'),
      expect.stringContaining('b'),
      expect.stringContaining('c'),
    ]);
    await userEvent.click(within(items[2] as HTMLElement).getByRole('button', { name: 'Move Up' }));
    await userEvent.click(within(items[0] as HTMLElement).getByRole('button', { name: 'Remove' }));
    await userEvent.click(screen.getByRole('combobox', { name: /Member Repositories/ }));
    const options = screen.getAllByRole('option').map((option) => option.textContent);
    expect(options).toEqual(['a (hosted)', 'd (hosted)']);
    await userEvent.click(screen.getByRole('option', { name: 'd (hosted)' }));
    await userEvent.click(screen.getByRole('button', { name: 'Save' }));
    await waitFor(() => expect(onSubmit).toHaveBeenCalledTimes(1));
    expect(getPath(onSubmit.mock.calls[0]?.[0] as RepositoryDocument, ['group', 'memberNames'])).toEqual([
      'c',
      'b',
      'd',
    ]);
  });

  it('creates from a template and reports the saved document', async () => {
    const spec: Spec = { format: 'docker', type: 'proxy' };
    const onSaved = vi.fn();
    const { onSubmit } = renderForm({
      spec,
      creating: true,
      document: template(spec, 'default'),
      blobStores: [
        { name: 'default', groups: [] },
        { name: 'member', groups: ['pool'] },
      ],
      submitLabel: 'Create repository',
      onSaved,
    });
    const form = screen.getByRole('form', { name: 'Create repository' });
    fireEvent.submit(form);
    expect(await screen.findByText('Enter a name')).toBeInTheDocument();
    await userEvent.type(screen.getByRole('textbox', { name: /Name/ }), 'docker-hub');
    await userEvent.type(
      screen.getByRole('textbox', { name: /Remote Storage/ }),
      'https://registry-1.docker.io',
    );
    fireEvent.submit(form);
    await waitFor(() => expect(onSaved).toHaveBeenCalledTimes(1));
    const sent = onSubmit.mock.calls[0]?.[0] as RepositoryDocument;
    expect(sent.name).toBe('docker-hub');
    expect(getPath(sent, ['storage', 'blobStoreName'])).toBe('default');
    expect(getPath(sent, ['proxy', 'remoteUrl'])).toBe('https://registry-1.docker.io');
    expect(getPath(sent, ['dockerProxy', 'indexType'])).toBe('REGISTRY');
  });

  it.each([
    ['huggingface', 'https://huggingface.co/'],
    ['swift', 'https://packages.example.com/'],
    ['terraform', 'https://registry.terraform.io'],
  ])('suggests the remote of %s proxies that the old form named', (format, remote) => {
    const spec: Spec = { format, type: 'proxy' };
    renderForm({ spec, creating: true, document: template(spec, 'default') });
    expect(
      screen.getByText(`Address of the remote repository to proxy. for example, ${remote}`),
    ).toBeInTheDocument();
  });

  it('applies cleanup policies of the format with the transfer list', async () => {
    const server = fakeServer({
      'GET /service/rest/internal/ui/cleanup-policies/names?format=raw': () =>
        jsonResponse([
          { name: 'daily', format: 'raw', notes: '' },
          { name: 'everything', format: '*', notes: '' },
          { name: 'weekly', format: 'raw', notes: '' },
        ]),
    });
    vi.stubGlobal('fetch', server.fetch);
    const document = structuredClone(BY_KEY['raw/hosted/max'] as RepositoryDocument);
    document.cleanup = { policyNames: ['weekly'] };
    const { onSubmit } = renderForm({ document });
    const available = await screen.findByRole('listbox', { name: 'Available Policies' });
    const applied = screen.getByRole('listbox', { name: 'Applied Policies' });
    await waitFor(() => expect(within(available).getAllByRole('option')).toHaveLength(2));
    expect(
      within(available)
        .getAllByRole('option')
        .map((option) => option.textContent),
    ).toEqual(['daily', 'everything']);
    expect(
      within(applied)
        .getAllByRole('option')
        .map((option) => option.textContent),
    ).toEqual(['weekly']);
    expect(screen.getByRole('button', { name: 'Apply the selected policies' })).toBeDisabled();
    await userEvent.dblClick(within(available).getByRole('option', { name: 'everything' }));
    await userEvent.click(within(available).getByRole('option', { name: 'daily' }));
    expect(within(available).getByRole('option', { name: 'daily' })).toHaveAttribute('aria-selected', 'true');
    await userEvent.click(screen.getByRole('button', { name: 'Apply the selected policies' }));
    expect(within(available).queryAllByRole('option')).toHaveLength(0);
    expect(within(available).getByText('No policies available')).toBeInTheDocument();
    await userEvent.click(within(applied).getByRole('option', { name: 'weekly' }));
    await userEvent.click(screen.getByRole('button', { name: 'Remove the selected policies' }));
    await userEvent.type(screen.getByRole('textbox', { name: 'Applied Policies: Filter...' }), 'EVERY');
    expect(
      within(applied)
        .getAllByRole('option')
        .map((option) => option.textContent),
    ).toEqual(['everything']);
    expect(screen.getByText('1 item')).toBeInTheDocument();
    expect(screen.getByText('2 items')).toBeInTheDocument();
    await userEvent.click(screen.getByRole('button', { name: 'Save' }));
    await waitFor(() => expect(onSubmit).toHaveBeenCalledTimes(1));
    expect(getPath(onSubmit.mock.calls[0]?.[0] as RepositoryDocument, ['cleanup', 'policyNames'])).toEqual([
      'everything',
      'daily',
    ]);
  });

  it('has no cleanup for group repositories', () => {
    renderForm({ document: BY_KEY['raw/group/max'] as RepositoryDocument });
    expect(screen.queryByRole('listbox', { name: 'Available Policies' })).not.toBeInTheDocument();
  });

  it('offers the routing rules with their modes to proxies', async () => {
    const server = fakeServer({
      'GET /service/rest/v1/routing-rules': () =>
        jsonResponse([
          { name: 'block-sources', description: '', mode: 'BLOCK', matchers: ['.*-sources\\.jar'] },
          { name: 'only-org', description: '', mode: 'ALLOW', matchers: ['^/org/.*'] },
        ]),
    });
    vi.stubGlobal('fetch', server.fetch);
    const { onSubmit } = renderForm({ document: BY_KEY['raw/proxy/min'] as RepositoryDocument });
    const select = screen.getByRole('combobox', { name: /Routing Rule/ });
    await waitFor(() => expect(server.calls.some((call) => call.path.endsWith('/routing-rules'))).toBe(true));
    await userEvent.click(select);
    expect(screen.getAllByRole('option').map((option) => option.textContent)).toEqual([
      'None (allow every request)',
      'block-sources (BLOCK)',
      'only-org (ALLOW)',
    ]);
    await userEvent.click(screen.getByRole('option', { name: 'only-org (ALLOW)' }));
    await userEvent.click(screen.getByRole('button', { name: 'Save' }));
    await waitFor(() => expect(onSubmit).toHaveBeenCalledTimes(1));
    expect(getPath(onSubmit.mock.calls[0]?.[0] as RepositoryDocument, ['routingRuleName'])).toBe('only-org');
  });

  it('keeps the current rule when the rules cannot be listed', async () => {
    const server = fakeServer({
      'GET /service/rest/v1/routing-rules': () => jsonResponse({ id: '*', message: 'forbidden' }, 403),
    });
    vi.stubGlobal('fetch', server.fetch);
    const document = structuredClone(BY_KEY['raw/proxy/min'] as RepositoryDocument);
    document.routingRuleName = 'secret-rule';
    const { onSubmit } = renderForm({ document });
    const select = screen.getByRole('combobox', { name: /Routing Rule/ });
    expect(select).toHaveTextContent('secret-rule');
    await waitFor(() => expect(server.calls.some((call) => call.path.endsWith('/routing-rules'))).toBe(true));
    await userEvent.click(select);
    await userEvent.click(screen.getByRole('option', { name: 'None (allow every request)' }));
    await userEvent.click(screen.getByRole('button', { name: 'Save' }));
    await waitFor(() => expect(onSubmit).toHaveBeenCalledTimes(1));
    expect(getPath(onSubmit.mock.calls[0]?.[0] as RepositoryDocument, ['routingRuleName'])).toBeNull();
  });

  it('has no routing rule for groups, like the old form', () => {
    const document = structuredClone(BY_KEY['raw/group/min'] as RepositoryDocument);
    document.routingRuleName = 'kept-rule';
    renderForm({ document });
    expect(screen.queryByRole('combobox', { name: /Routing Rule/ })).not.toBeInTheDocument();
  });

  it('has no routing rule for hosted repositories', () => {
    renderForm({ document: BY_KEY['raw/hosted/max'] as RepositoryDocument });
    expect(screen.queryByRole('combobox', { name: /Routing Rule/ })).not.toBeInTheDocument();
  });

  it('suggests free ports for the Docker connectors', async () => {
    const server = fakeServer({
      'GET /service/rest/internal/ui/docker/suggest-port': () => jsonResponse(8082),
      'GET /service/rest/internal/ui/docker/suggest-port?exclude=8082': () => jsonResponse(8083),
    });
    vi.stubGlobal('fetch', server.fetch);
    const document = structuredClone(BY_KEY['docker/hosted/max'] as RepositoryDocument);
    const { onSubmit } = renderForm({ document });
    const links = screen.getAllByRole('button', { name: 'Suggest available port' });
    expect(links).toHaveLength(2);
    await userEvent.click(links[0] as HTMLElement);
    await waitFor(() => expect(screen.getByRole('spinbutton', { name: /HTTP Connector/ })).toHaveValue(8082));
    await userEvent.click(links[1] as HTMLElement);
    await waitFor(() =>
      expect(screen.getByRole('spinbutton', { name: /HTTPS Connector/ })).toHaveValue(8083),
    );
    await userEvent.click(screen.getByRole('button', { name: 'Save' }));
    await waitFor(() => expect(onSubmit).toHaveBeenCalledTimes(1));
    const sent = onSubmit.mock.calls[0]?.[0] as RepositoryDocument;
    expect(getPath(sent, ['docker', 'httpPort'])).toBe(8082);
    expect(getPath(sent, ['docker', 'httpsPort'])).toBe(8083);
  });

  it('looks up a new domain before saving and saves it even without an A record', async () => {
    const check = vi.fn(() =>
      jsonResponse({
        host: 'docker-prem.example.org',
        status: 'missing',
        addresses: [],
        serverAddresses: ['10.0.0.1'],
      }),
    );
    const server = fakeServer({
      'GET /service/rest/internal/ui/domains/check?host=docker-prem.example.org': check,
    });
    vi.stubGlobal('fetch', server.fetch);
    const document = structuredClone(BY_KEY['docker/hosted/max'] as RepositoryDocument);
    const { onSubmit } = renderForm({ document });
    await userEvent.type(screen.getByRole('textbox', { name: 'Domain' }), 'Docker-Prem.example.org');
    expect(await screen.findByRole('status')).toHaveTextContent(
      'docker-prem.example.org has no A record yet',
    );
    expect(screen.getByText('Address on the domain: docker-prem.example.org/<image>')).toBeInTheDocument();
    await userEvent.click(screen.getByRole('button', { name: 'Save' }));
    await waitFor(() => expect(onSubmit).toHaveBeenCalledTimes(1));
    expect(getPath(onSubmit.mock.calls[0]?.[0] as RepositoryDocument, ['domain', 'host'])).toBe(
      'Docker-Prem.example.org',
    );
    expect(await screen.findByText('Saved, but the domain does not lead here yet')).toBeInTheDocument();
    expect(check).toHaveBeenCalledTimes(2);
  });

  it('refuses a domain that is not a domain name before asking the server', async () => {
    const document = structuredClone(BY_KEY['raw/hosted/max'] as RepositoryDocument);
    const { onSubmit } = renderForm({ document });
    await userEvent.type(screen.getByRole('textbox', { name: 'Domain' }), 'https://files.example.org');
    await userEvent.click(screen.getByRole('button', { name: 'Save' }));
    expect(await screen.findByText(/Enter a domain name such as docker.example.com/)).toBeInTheDocument();
    expect(onSubmit).not.toHaveBeenCalled();
  });

  it('keeps readers from asking for a port', () => {
    renderForm({ document: BY_KEY['docker/hosted/max'] as RepositoryDocument, readOnly: true });
    for (const link of screen.getAllByRole('button', { name: 'Suggest available port' })) {
      expect(link).toBeDisabled();
    }
  });
});
