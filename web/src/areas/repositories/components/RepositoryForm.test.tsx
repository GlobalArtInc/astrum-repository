import { fireEvent, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { ApiError } from '../../../api';
import { renderWithApp } from '../../../test/render';
import type { UiRepository } from '../api';
import { canonical, getPath, type RepositoryDocument } from '../model/document';
import { template } from '../model/form';
import reference from '../model/reference-documents.json';
import type { Spec } from '../model/spec';
import { RepositoryForm, type RepositoryFormProps } from './RepositoryForm';

vi.setConfig({ testTimeout: 30_000 });

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
    const remote = screen.getByRole('textbox', { name: /Remote storage URL/ });
    await userEvent.clear(remote);
    await userEvent.type(remote, 'https://other.example.org/');
    await userEvent.click(screen.getByRole('button', { name: 'Save' }));
    await waitFor(() => expect(onSubmit).toHaveBeenCalledTimes(1));
    expect(getPath(onSubmit.mock.calls[0]?.[0] as RepositoryDocument, ['proxy', 'remoteUrl'])).toBe(
      'https://other.example.org/',
    );
    expect(await screen.findByRole('switch', { name: /Remember missing paths/ })).toHaveAccessibleDescription(
      /must not be null/,
    );
    expect(screen.getByText('routingRuleName: the routing rule "x" does not exist')).toBeInTheDocument();
  });

  it('stops at its own checks before asking the server', async () => {
    const document = BY_KEY['maven/proxy/min'] as RepositoryDocument;
    const { onSubmit } = renderForm({ document });
    await userEvent.clear(screen.getByRole('spinbutton', { name: /Time to live/ }));
    await userEvent.click(screen.getByRole('button', { name: 'Save' }));
    expect(await screen.findByText('Enter a whole number of minutes, 0 or more')).toBeInTheDocument();
    expect(onSubmit).not.toHaveBeenCalled();
  });

  it('shows settings without letting a reader change them', () => {
    renderForm({ document: BY_KEY['docker/hosted/max'] as RepositoryDocument, readOnly: true });
    expect(screen.getByText('You can read these settings but not change them.')).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'Save' })).not.toBeInTheDocument();
    expect(screen.getByRole('switch', { name: /Allow redeploying the latest tag/ })).toBeDisabled();
    expect(screen.getByRole('combobox', { name: /Deployment policy/ })).toBeDisabled();
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
    const members = screen.getByRole('list', { name: 'Members' });
    expect(
      within(members)
        .getAllByRole('listitem')
        .map((item) => item.textContent),
    ).toEqual([expect.stringContaining('a'), expect.stringContaining('b'), expect.stringContaining('c')]);
    expect(within(members).getByText('Offline')).toBeInTheDocument();
    await userEvent.click(screen.getByRole('button', { name: 'Move c up' }));
    await userEvent.click(screen.getByRole('button', { name: 'Remove a' }));
    await userEvent.click(screen.getByRole('combobox', { name: 'Add a repository…' }));
    const options = screen.getAllByRole('option').map((option) => option.textContent);
    expect(options).toEqual(['a', 'd']);
    await userEvent.click(screen.getByRole('option', { name: 'd' }));
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
    await userEvent.click(screen.getByRole('button', { name: 'Create repository' }));
    expect(await screen.findByText('Enter a name')).toBeInTheDocument();
    await userEvent.type(screen.getByRole('textbox', { name: /Name/ }), 'docker-hub');
    await userEvent.type(
      screen.getByRole('textbox', { name: /Remote storage URL/ }),
      'https://registry-1.docker.io',
    );
    await userEvent.click(screen.getByRole('button', { name: 'Create repository' }));
    await waitFor(() => expect(onSaved).toHaveBeenCalledTimes(1));
    const sent = onSubmit.mock.calls[0]?.[0] as RepositoryDocument;
    expect(sent.name).toBe('docker-hub');
    expect(getPath(sent, ['storage', 'blobStoreName'])).toBe('default');
    expect(getPath(sent, ['proxy', 'remoteUrl'])).toBe('https://registry-1.docker.io');
    expect(getPath(sent, ['dockerProxy', 'indexType'])).toBe('REGISTRY');
  });
});
