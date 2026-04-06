import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { ApiError } from '../../api';
import { ADMIN_STATE, jsonResponse, renderWithApp } from '../../test/render';
import { MAVEN, NPM, RAW } from './specs.fixture';
import { uploadComponent } from './transport';
import UploadListPage from './UploadListPage';
import UploadPage from './UploadPage';

vi.setConfig({ testTimeout: 20_000 });

const FILE = /^File\*?$/;

vi.mock('./transport', () => ({ uploadComponent: vi.fn() }));

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
});

describe('the upload list', () => {
  it('shows only hosted repositories the caller may add to, with the old filters', async () => {
    stub();
    const { router } = page('/upload');
    await screen.findByRole('button', { name: 'Upload to files' });
    expect(screen.getByRole('button', { name: 'Upload to releases' })).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'Upload to cache' })).toBeNull();
    expect(screen.queryByRole('button', { name: 'Upload to locked' })).toBeNull();
    expect(screen.getByText('3 repositories')).toBeInTheDocument();
    const filters = screen.getByRole('complementary', { name: 'Filter panel' });
    expect(within(filters).getByText('Raw')).toBeInTheDocument();
    await userEvent.click(within(filters).getByRole('checkbox', { name: 'Maven' }));
    expect(screen.getByText('1 repository')).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'Upload to files' })).toBeNull();
    await userEvent.click(within(filters).getByRole('button', { name: 'Reset Filters' }));
    await userEvent.type(screen.getByRole('textbox', { name: 'Filter repositories by name' }), 'fil');
    expect(screen.queryByRole('button', { name: 'Upload to releases' })).toBeNull();
    await userEvent.click(screen.getByRole('button', { name: 'Upload to files' }));
    expect(router.state.location.pathname).toBe('/upload/files');
  });

  it('sorts by name or format from the headers', async () => {
    stub();
    page('/upload');
    await screen.findByRole('button', { name: 'Upload to files' });
    const names = () =>
      screen.getAllByRole('button', { name: /^Upload to / }).map((row) => row.getAttribute('aria-label'));
    expect(names()).toEqual(['Upload to files', 'Upload to packages', 'Upload to releases']);
    await userEvent.click(screen.getByRole('button', { name: 'Name: ascending; click to sort descending' }));
    expect(names()).toEqual(['Upload to releases', 'Upload to packages', 'Upload to files']);
    await userEvent.click(screen.getByRole('button', { name: 'Sort by Format' }));
    expect(names()).toEqual(['Upload to releases', 'Upload to packages', 'Upload to files']);
  });
});

describe('the upload form', () => {
  it('uploads raw files with names from the chosen files and returns to the list', async () => {
    stub();
    vi.mocked(uploadComponent).mockImplementation(async (_repository, _data, progress) => {
      progress({ loaded: 5, total: 10 });
    });
    const { router } = page('/upload/files');
    expect(await screen.findByText('No files selected')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Upload Component' })).toBeDisabled();
    expect(screen.getByRole('combobox', { name: /Target Repository/ })).toHaveValue('files');
    await userEvent.type(screen.getByLabelText(/Directory/), '/tools/linux');
    await userEvent.upload(
      screen.getByLabelText(FILE, { selector: 'input' }),
      new File(['hello'], 'hello.txt'),
    );
    expect(screen.getByLabelText(/File Name/)).toHaveValue('hello.txt');
    await userEvent.click(screen.getByRole('button', { name: 'Add Another File' }));
    const second = screen.getByRole('group', { name: 'File 2' });
    await userEvent.upload(
      within(second).getByLabelText(FILE, { selector: 'input' }),
      new File(['bye'], 'bye.txt'),
    );
    await userEvent.clear(within(second).getByLabelText(/File Name/));
    await userEvent.type(within(second).getByLabelText(/File Name/), 'docs/bye.txt');
    expect(screen.getByText('2 files ready')).toBeInTheDocument();
    await userEvent.click(screen.getByRole('button', { name: 'Upload Component' }));
    await waitFor(() => expect(router.state.location.pathname).toBe('/upload'));
    expect(await screen.findByText('Component uploaded to files')).toBeInTheDocument();
    expect(screen.getByText('Component: tools linux')).toBeInTheDocument();
    expect(vi.mocked(uploadComponent).mock.calls[0]?.[0]).toBe('files');
    expect(sent()).toEqual([
      ['raw.directory', '/tools/linux'],
      ['raw.asset1.filename', 'hello.txt'],
      ['raw.asset1', 'file:hello.txt'],
      ['raw.asset2.filename', 'docs/bye.txt'],
      ['raw.asset2', 'file:bye.txt'],
    ]);
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
    expect(await screen.findByRole('combobox', { name: 'Packaging' })).toBeDisabled();
    await userEvent.upload(
      screen.getByLabelText(FILE, { selector: 'input' }),
      new File(['jar'], 'lib-1.0-tests.jar'),
    );
    expect(screen.getByLabelText(/Extension/)).toHaveValue('jar');
    await userEvent.click(screen.getByRole('button', { name: 'Upload Component' }));
    expect(screen.getByText('Fix the errors in the form')).toBeInTheDocument();
    expect(screen.getAllByText('This field is required')).toHaveLength(3);
    expect(uploadComponent).not.toHaveBeenCalled();
    await userEvent.type(screen.getByLabelText(/Group ID/), 'org.example');
    await userEvent.type(screen.getByLabelText(/Artifact ID/), 'lib');
    await userEvent.type(screen.getByLabelText(/^Version/), '1.0');
    await userEvent.click(screen.getByText('Generate POM'));
    expect(screen.getByRole('combobox', { name: 'Packaging' })).toBeEnabled();
    expect(screen.getByRole('combobox', { name: 'Packaging' })).toHaveTextContent('jar');
    await userEvent.click(screen.getByRole('button', { name: 'Upload Component' }));
    expect(await screen.findByText('maven2.asset1.extension is not allowed here')).toBeInTheDocument();
    expect(screen.getByLabelText(/Extension/)).toHaveAttribute('aria-invalid', 'true');
    expect(screen.getByText('Version policy mismatch')).toBeInTheDocument();
    expect(sent()).toContainEqual(['maven2.generate-pom', 'true']);
    expect(sent()).toContainEqual(['maven2.packaging', 'jar']);
  });

  it('lets a POM stand in for the coordinates', async () => {
    stub();
    vi.mocked(uploadComponent).mockResolvedValue();
    const { router } = page('/upload/releases');
    await userEvent.upload(
      await screen.findByLabelText(FILE, { selector: 'input' }),
      new File(['<project/>'], 'lib-2.0.pom'),
    );
    expect(screen.getByLabelText(/Extension/)).toHaveValue('pom');
    expect(
      screen.getByText('The component coordinates are taken from the provided POM file.'),
    ).toBeInTheDocument();
    expect(screen.getByRole('checkbox', { name: /Generate POM/ })).toBeDisabled();
    await userEvent.click(screen.getByRole('button', { name: 'Upload Component' }));
    await waitFor(() => expect(router.state.location.pathname).toBe('/upload'));
    expect(sent().map(([key]) => key)).toEqual(['maven2.asset1.extension', 'maven2.asset1']);
  });

  it('refuses files of the wrong kind for npm', async () => {
    stub();
    page('/upload/packages');
    await userEvent.upload(
      await screen.findByLabelText(FILE, { selector: 'input' }),
      new File(['x'], 'package.zip'),
      {
        applyAccept: false,
      },
    );
    await userEvent.click(screen.getByRole('button', { name: 'Upload Component' }));
    expect(await screen.findByText('Supported: .tgz, .tar.gz')).toBeInTheDocument();
    expect(uploadComponent).not.toHaveBeenCalled();
  });

  it('switches the target repository from the combobox', async () => {
    stub();
    const { router } = page('/upload/files');
    const combo = await screen.findByRole('combobox', { name: /Target Repository/ });
    await userEvent.clear(combo);
    await userEvent.type(combo, 'rel');
    const options = screen.getAllByRole('option');
    expect(options.map((option) => option.textContent)).toEqual(['releasesmaven2']);
    await userEvent.keyboard('{Enter}');
    await waitFor(() => expect(router.state.location.pathname).toBe('/upload/releases'));
  });

  it('explains repositories that take no uploads from the caller', async () => {
    stub();
    const { router } = page('/upload/locked');
    expect(await screen.findByRole('alert')).toHaveTextContent('locked');
    await userEvent.click(screen.getByRole('link', { name: 'Back to Upload' }));
    await waitFor(() => expect(router.state.location.pathname).toBe('/upload'));
  });
});
