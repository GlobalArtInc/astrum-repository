import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { ApiError } from '../../api';
import { ADMIN_STATE, jsonResponse, renderWithApp } from '../../test/render';
import { MAVEN, NPM, RAW } from './specs.fixture';
import { sha1Of, uploadComponent } from './transport';
import UploadListPage from './UploadListPage';
import UploadPage from './UploadPage';

vi.setConfig({ testTimeout: 20_000 });

const FILE = /^File\*?$/;

vi.mock('./transport', () => ({ uploadComponent: vi.fn(), sha1Of: vi.fn() }));

const repository = (name: string, format: string, type = 'hosted', add = true) => ({
  name,
  format,
  type,
  url: `http://localhost/repository/${name}`,
  online: true,
  componentCount: 3,
  lastUploaded: null,
  permissions: { add, edit: add },
});

const REPOSITORIES = [
  repository('files', 'raw'),
  repository('releases', 'maven2'),
  repository('packages', 'npm'),
  repository('cache', 'raw', 'proxy'),
  repository('locked', 'raw', 'hosted', false),
];

function stub(search: unknown = { items: [], continuationToken: null }) {
  const calls: URL[] = [];
  vi.stubGlobal('fetch', async (input: RequestInfo | URL) => {
    const url = new URL(String(input), 'http://localhost');
    calls.push(url);
    switch (url.pathname) {
      case '/service/rest/internal/ui/repositories':
        return jsonResponse(REPOSITORIES);
      case '/service/rest/v1/formats/upload-specs':
        return jsonResponse([RAW, MAVEN, NPM]);
      case '/service/rest/v1/search':
        return jsonResponse(search);
      default:
        return jsonResponse({ id: '*', message: 'missing' }, 404);
    }
  });
  return calls;
}

function page(path: string) {
  return renderWithApp(null, {
    state: ADMIN_STATE,
    path,
    routes: [
      { path: '/upload', element: <UploadListPage /> },
      { path: '/upload/:repository', element: <UploadPage /> },
      { path: '*', element: <p>elsewhere</p> },
    ],
  });
}

function sent(): Array<[string, string]> {
  const data = vi.mocked(uploadComponent).mock.calls[0]?.[1] as FormData;
  return [...data.entries()].map(([key, value]) => [
    key,
    typeof value === 'string' ? value : `file:${value.name}`,
  ]);
}

beforeEach(() => {
  vi.mocked(uploadComponent).mockReset();
  vi.mocked(sha1Of).mockReset();
  vi.mocked(sha1Of).mockResolvedValue(null);
});

describe('the upload list', () => {
  it('shows only hosted repositories the caller may add to', async () => {
    stub();
    page('/upload');
    expect(await screen.findByRole('link', { name: 'files' })).toHaveAttribute('href', '/upload/files');
    expect(screen.getByRole('link', { name: 'releases' })).toBeInTheDocument();
    expect(screen.queryByRole('link', { name: 'cache' })).toBeNull();
    expect(screen.queryByRole('link', { name: 'locked' })).toBeNull();
    await userEvent.type(screen.getByLabelText('Name'), 'rel');
    expect(screen.queryByRole('link', { name: 'files' })).toBeNull();
  });
});

describe('the upload form', () => {
  it('uploads raw files with names from the chosen files and links to the folder', async () => {
    stub();
    vi.mocked(uploadComponent).mockImplementation(async (_repository, _data, progress) => {
      progress({ loaded: 5, total: 10 });
    });
    page('/upload/files');
    await userEvent.type(await screen.findByLabelText(/Directory/), '/tools/linux');
    await userEvent.upload(screen.getByLabelText(FILE), new File(['hello'], 'hello.txt'));
    expect(screen.getByLabelText(/File name/)).toHaveValue('hello.txt');
    await userEvent.click(screen.getByRole('button', { name: 'Add another file' }));
    const second = screen.getByRole('group', { name: 'File 2' });
    await userEvent.upload(within(second).getByLabelText(FILE), new File(['bye'], 'bye.txt'));
    await userEvent.clear(within(second).getByLabelText(/File name/));
    await userEvent.type(within(second).getByLabelText(/File name/), 'docs/bye.txt');
    expect(screen.getByText('2 files ready, 8 B')).toBeInTheDocument();
    await userEvent.click(screen.getByRole('button', { name: 'Upload' }));
    expect(await screen.findByRole('link', { name: 'Show in Browse' })).toHaveAttribute(
      'href',
      '/browse/files/tools/linux',
    );
    expect(vi.mocked(uploadComponent).mock.calls[0]?.[0]).toBe('files');
    expect(sent()).toEqual([
      ['raw.directory', '/tools/linux'],
      ['raw.asset1.filename', 'hello.txt'],
      ['raw.asset1', 'file:hello.txt'],
      ['raw.asset2.filename', 'docs/bye.txt'],
      ['raw.asset2', 'file:bye.txt'],
    ]);
    expect(screen.getByText('No files chosen')).toBeInTheDocument();
  });

  it('checks the form before sending and shows server errors on their fields', async () => {
    stub();
    vi.mocked(uploadComponent).mockRejectedValue(
      new ApiError(400, 'x', {
        fieldErrors: [
          { id: '*', message: 'maven2.asset1.extension is not allowed here' },
          { id: '*', message: 'Version policy mismatch' },
        ],
      }),
    );
    page('/upload/releases');
    await userEvent.click(await screen.findByRole('button', { name: 'Upload' }));
    expect(screen.getByText('Fix the marked fields and try again.')).toBeInTheDocument();
    expect(screen.getAllByText('Fill in this field.')).toHaveLength(4);
    expect(screen.getByText('Choose a file.')).toBeInTheDocument();
    expect(uploadComponent).not.toHaveBeenCalled();
    await userEvent.type(screen.getByLabelText(/Group ID/), 'org.example');
    await userEvent.type(screen.getByLabelText(/Artifact ID/), 'lib');
    await userEvent.type(screen.getByLabelText(/^Version/), '1.0');
    await userEvent.click(screen.getByLabelText(/Generate a POM/));
    await userEvent.upload(screen.getByLabelText(FILE), new File(['jar'], 'lib-1.0-tests.jar'));
    expect(screen.getByLabelText(/Classifier/)).toHaveValue('tests');
    expect(screen.getByLabelText(/Extension/)).toHaveValue('jar');
    await userEvent.click(screen.getByRole('button', { name: 'Upload' }));
    expect(await screen.findByText('maven2.asset1.extension is not allowed here')).toBeInTheDocument();
    expect(screen.getByLabelText(/Extension/)).toHaveAttribute('aria-invalid', 'true');
    expect(screen.getByText('Version policy mismatch')).toBeInTheDocument();
    expect(sent()).toContainEqual(['maven2.generate-pom', 'true']);
  });

  it('lets a POM stand in for the coordinates and links to the uploaded component', async () => {
    const calls = stub({
      items: [
        {
          id: 'c1',
          repository: 'releases',
          format: 'maven2',
          group: 'org.example',
          name: 'lib',
          version: '2.0',
          assets: [
            {
              id: 'a1',
              path: '/org/example/lib/2.0/lib-2.0.pom',
              downloadUrl: '',
              repository: 'releases',
              format: 'maven2',
              contentType: 'application/xml',
              lastModified: null,
              fileSize: 3,
              checksum: { sha1: 'abc' },
            },
          ],
        },
      ],
      continuationToken: null,
    });
    vi.mocked(sha1Of).mockResolvedValue('abc');
    vi.mocked(uploadComponent).mockResolvedValue();
    page('/upload/releases');
    await userEvent.upload(await screen.findByLabelText(FILE), new File(['<project/>'], 'lib-2.0.pom'));
    expect(screen.getByLabelText(/Extension/)).toHaveValue('pom');
    expect(
      screen.getByText(
        'The coordinates come from the POM file among the files, so these fields can stay empty.',
      ),
    ).toBeInTheDocument();
    expect(screen.getByLabelText(/Generate a POM/)).toBeDisabled();
    await userEvent.click(screen.getByRole('button', { name: 'Upload' }));
    expect(await screen.findByRole('link', { name: 'Open lib 2.0' })).toHaveAttribute(
      'href',
      '/search/component/maven2%3Aorg.example%3Alib?version=2.0',
    );
    expect(screen.getByRole('link', { name: 'Show in Browse' })).toHaveAttribute(
      'href',
      '/browse/releases/org/example/lib/2.0',
    );
    const lookup = calls.find((url) => url.pathname === '/service/rest/v1/search');
    expect(lookup?.search).toBe('?repository=releases&sha1=abc');
  });

  it('refuses files of the wrong kind for npm', async () => {
    stub();
    page('/upload/packages');
    await userEvent.upload(await screen.findByLabelText(FILE), new File(['x'], 'package.zip'), {
      applyAccept: false,
    });
    await userEvent.click(screen.getByRole('button', { name: 'Upload' }));
    expect(await screen.findByText('Choose a file of these types: .tgz, .tar.gz.')).toBeInTheDocument();
    expect(uploadComponent).not.toHaveBeenCalled();
  });

  it('explains repositories that take no uploads from the caller', async () => {
    stub();
    const { router } = page('/upload/locked');
    expect(await screen.findByText('You cannot upload here')).toBeInTheDocument();
    await userEvent.click(screen.getByRole('link', { name: 'Back to Upload' }));
    await waitFor(() => expect(router.state.location.pathname).toBe('/upload'));
  });
});
