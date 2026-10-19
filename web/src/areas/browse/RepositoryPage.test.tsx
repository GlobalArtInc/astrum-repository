import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { ADMIN_STATE, fakeServer, jsonResponse, renderWithApp } from '../../test/render';
import { setMedia } from '../../test/setup';
import { COMPACT, PHONE } from '../../ui';
import RepositoryPage from './RepositoryPage';
import { file, folder, listing, repository } from './test-fixtures';
import type { AssetDetails } from './types';

const LIST = '/service/rest/internal/ui/repositories';
const TREE = '/service/rest/internal/ui/browse/files';

function details(path: string, extra: Partial<AssetDetails> = {}): AssetDetails {
  return {
    id: `id-${path}`,
    path,
    repository: 'files',
    browsedRepository: 'files',
    format: 'raw',
    downloadUrl: `http://localhost/repository/files${path}`,
    checksum: { sha1: 'a'.repeat(40), sha256: 'b'.repeat(64), md5: 'c'.repeat(32) },
    contentType: 'text/plain',
    lastModified: '2026-10-01T10:00:00.000+00:00',
    lastDownloaded: null,
    uploader: 'admin',
    uploaderIp: '127.0.0.1',
    fileSize: 5,
    blobCreated: '2026-10-01T10:00:00.000+00:00',
    blobUpdated: '2026-10-01T10:00:00.000+00:00',
    blobStoreName: 'default',
    lastVerified: null,
    downloadCount: 7,
    component: { id: 'c1', repository: 'files', group: '/a/b', name: path, version: '' },
    canDelete: true,
    raw: {},
    ...extra,
  };
}

function server(extra: Record<string, () => Response> = {}) {
  const fake = fakeServer({
    [`GET ${LIST}`]: () => jsonResponse([repository('files', 'raw', 'hosted')]),
    [`GET ${TREE}?path=%2F`]: () => jsonResponse(listing('files', '/', [folder('/a/', 2), file('/c.txt')])),
    [`GET ${TREE}?path=%2Fa%2F`]: () =>
      jsonResponse(listing('files', '/a/', [folder('/a/b/'), file('/a/one.txt')], true)),
    [`GET ${TREE}?path=%2Fa%2Fb%2F`]: () => jsonResponse(listing('files', '/a/b/', [file('/a/b/two.txt')])),
    [`GET ${TREE}/asset?path=%2Fa%2Fb%2Ftwo.txt`]: () => jsonResponse(details('/a/b/two.txt')),
    [`GET ${TREE}/asset?path=%2Fc.txt`]: () => jsonResponse(details('/c.txt', { canDelete: false })),
    ...extra,
  });
  vi.stubGlobal('fetch', fake.fetch);
  return fake;
}

function page(path: string) {
  return renderWithApp(null, {
    path,
    routes: [{ path: 'browse/:repository/*', element: <RepositoryPage /> }],
  });
}

afterEach(() => {
  vi.unstubAllGlobals();
});

describe('the repository page', { timeout: 20_000 }, () => {
  it('opens a deep link down to the file and shows its details', async () => {
    server();
    const { router } = page('/browse/files/a/b/two.txt');
    const tree = await screen.findByRole('tree', { name: 'Content of files' });
    const selected = await within(tree).findByRole('treeitem', { name: /two\.txt/ });
    expect(selected).toHaveAttribute('aria-selected', 'true');
    expect(within(tree).getByRole('treeitem', { name: /^a/ })).toHaveAttribute('aria-expanded', 'true');
    expect(await screen.findByRole('heading', { level: 2, name: 'two.txt' })).toBeInTheDocument();
    expect(screen.getByText('7')).toBeInTheDocument();
    expect(screen.getByText('b'.repeat(64))).toBeInTheDocument();
    expect(screen.getByRole('link', { name: /Download/ })).toHaveAttribute(
      'href',
      'http://localhost/repository/files/a/b/two.txt',
    );
    expect(screen.getByRole('button', { name: 'Delete File' })).toBeInTheDocument();
    expect(screen.getByText('MD5')).toBeInTheDocument();
    expect(screen.getByRole('note')).toHaveTextContent('Only the first 2 items of this folder are shown');
    expect(router.state.location.pathname).toBe('/browse/files/a/b/two.txt');
  });

  it('shows the size of an image next to the size of its manifest', async () => {
    server({
      [`GET ${TREE}/asset?path=%2Fc.txt`]: () =>
        jsonResponse(
          details('/c.txt', {
            fileSize: 949,
            image: {
              platforms: [
                { platform: 'linux/amd64', digest: 'sha256:a', size: 30_000_000, layers: 3 },
                { platform: 'linux/arm64', digest: 'sha256:b', size: null, layers: null },
              ],
            },
          }),
        ),
    });
    page('/browse/files/c.txt');
    expect(await screen.findByText('Image size, linux/amd64')).toBeInTheDocument();
    expect(screen.getByText(/3 layers/)).toBeInTheDocument();
    expect(screen.getByText('Image size, linux/arm64')).toBeInTheDocument();
    expect(screen.getByText('Unknown')).toBeInTheDocument();
    expect(screen.getByText('Manifest size')).toBeInTheDocument();
    expect(screen.queryByText('Size')).toBeNull();
  });

  it('moves through the tree with the keyboard and selects with Enter', async () => {
    server();
    const { router } = page('/browse/files');
    const tree = await screen.findByRole('tree');
    const first = await within(tree).findByRole('treeitem', { name: /^a/ });
    first.focus();
    await userEvent.keyboard('{ArrowRight}');
    await within(tree).findByRole('treeitem', { name: /one\.txt/ });
    await userEvent.keyboard('{ArrowDown}');
    expect(within(tree).getByRole('treeitem', { name: /^b/ })).toHaveFocus();
    await userEvent.keyboard('{End}');
    expect(within(tree).getByRole('treeitem', { name: /c\.txt/ })).toHaveFocus();
    await userEvent.keyboard('{Enter}');
    await waitFor(() => expect(router.state.location.pathname).toBe('/browse/files/c.txt'));
    expect(await screen.findByRole('heading', { level: 2, name: 'c.txt' })).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'Delete File' })).toBeNull();
    await userEvent.click(within(tree).getByText('a', { exact: true }));
    await waitFor(() => expect(router.state.location.pathname).toBe('/browse/files/a/'));
    expect(await screen.findByRole('heading', { level: 2, name: 'a' })).toBeInTheDocument();
  });

  it('deletes a folder after confirming and reports the counts', async () => {
    const fake = server({
      [`DELETE ${TREE}?path=%2Fa%2F`]: () =>
        jsonResponse({ deletedAssets: 3, deletedComponents: 3, skippedAssets: 1 }),
    });
    const { router } = page('/browse/files/a/');
    expect(await screen.findByText('Deleting a folder deletes everything inside it.')).toBeInTheDocument();
    await userEvent.click(await screen.findByRole('button', { name: 'Delete Folder' }));
    const dialog = await screen.findByRole('alertdialog', { name: 'Delete Folder' });
    expect(dialog).toHaveTextContent('Delete the folder “a” with everything in it? This cannot be undone.');
    expect(dialog).toHaveTextContent('Repository: files');
    await userEvent.click(within(dialog).getByRole('button', { name: 'Delete' }));
    await waitFor(() => expect(fake.calls.some((call) => call.method === 'DELETE')).toBe(true));
    expect(await screen.findByText('The folder a is deleted')).toBeInTheDocument();
    expect(
      screen.getByText('3 files deleted · 3 components deleted · 1 file kept because you may not delete it'),
    ).toBeInTheDocument();
    await waitFor(() => expect(router.state.location.pathname).toBe('/browse/files'));
  });

  it('says when the selection is not in the repository', async () => {
    server();
    page('/browse/files/a/missing.txt');
    expect(await screen.findByRole('heading', { name: 'Not in this repository' })).toBeInTheDocument();
  });

  it('says when the repository cannot be opened', async () => {
    server();
    page('/browse/other');
    expect(
      await screen.findByRole('heading', { level: 1, name: 'Repository not found' }),
    ).toBeInTheDocument();
  });

  it('searches inside the repository and opens a result in the tree', async () => {
    server({
      'GET /service/rest/v1/search?repository=files&q=two': () =>
        jsonResponse({
          items: [
            {
              id: 'c1',
              repository: 'files',
              format: 'raw',
              group: '/a/b',
              name: '/a/b/two.txt',
              version: '',
              assets: [{ path: '/a/b/two.txt' }],
            },
          ],
          continuationToken: null,
        }),
    });
    const { router } = page('/browse/files');
    await userEvent.type(await screen.findByRole('searchbox', { name: 'Search in files' }), 'two');
    const results = await screen.findByRole('list', { name: 'Search results' }, { timeout: 3000 });
    await userEvent.click(within(results).getByRole('button', { name: /two\.txt/ }));
    await waitFor(() => expect(router.state.location.pathname).toBe('/browse/files/a/b/two.txt'));
  });

  it('stacks the tree and the details on a phone', async () => {
    setMedia(PHONE, true);
    setMedia(COMPACT, true);
    server();
    page('/browse/files/c.txt');
    expect(await screen.findByRole('heading', { level: 2, name: 'c.txt' })).toBeInTheDocument();
    expect(screen.getByRole('tree', { name: 'Content of files' })).toBeInTheDocument();
  });

  it('filters the loaded tree and shows the usage of a file in pill tabs', async () => {
    server();
    renderWithApp(null, {
      path: '/browse/files/c.txt',
      state: { ...ADMIN_STATE, baseUrl: 'https://repo.example.org' },
      routes: [{ path: 'browse/:repository/*', element: <RepositoryPage /> }],
    });
    expect(await screen.findByText('Files')).toBeInTheDocument();
    const tabs = await screen.findByRole('tablist', { name: 'Clients' });
    expect(within(tabs).getByRole('tab', { name: 'curl' })).toHaveAttribute('aria-selected', 'true');
    expect(
      screen.getByText(/curl -fLO https:\/\/repo\.example\.org\/repository\/files\/c\.txt/),
    ).toBeInTheDocument();
    await userEvent.click(within(tabs).getByRole('tab', { name: /Upload/ }));
    expect(screen.getByText(/--upload-file/)).toBeInTheDocument();
    const tree = screen.getByRole('tree');
    await userEvent.type(screen.getByRole('searchbox', { name: 'Filter the loaded tree' }), 'c.t');
    expect(within(tree).getAllByRole('treeitem')).toHaveLength(1);
    await userEvent.type(screen.getByRole('searchbox', { name: 'Filter the loaded tree' }), 'zzz');
    expect(screen.getByText('Nothing loaded matches the filter')).toBeInTheDocument();
  });
});
