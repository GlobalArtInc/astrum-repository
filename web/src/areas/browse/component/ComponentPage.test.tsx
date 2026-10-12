import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { ADMIN_STATE, fakeServer, jsonResponse, renderWithApp } from '../../../test/render';
import { repository } from '../test-fixtures';
import type { SearchAsset, SearchComponent } from '../types';
import ComponentPage, { keyFromPath, mainAsset } from './ComponentPage';

const VERSIONS = '/service/rest/internal/ui/search/versions?format=maven2&group=org.acme&name=lib';

function component(version: string, repositoryName: string): SearchComponent {
  const base = `/org/acme/lib/${version}/lib-${version}`;
  const asset = (extension: string, size: number) => ({
    id: `${repositoryName}-${version}-${extension}`,
    path: `${base}.${extension}`,
    repository: repositoryName,
    format: 'maven2',
    downloadUrl: `http://localhost/repository/${repositoryName}${base}.${extension}`,
    checksum: { sha1: 'a'.repeat(40) },
    contentType: extension === 'jar' ? 'application/java-archive' : 'application/xml',
    lastModified: '2026-10-01T10:00:00.000+00:00',
    lastDownloaded: null,
    uploader: 'admin',
    fileSize: size,
    downloadCount: 0,
    maven2: { groupId: 'org.acme', artifactId: 'lib', version, extension },
  });
  return {
    id: `${repositoryName}-${version}`,
    repository: repositoryName,
    format: 'maven2',
    group: 'org.acme',
    name: 'lib',
    version,
    assets: [asset('pom', 300), asset('jar', 2048)],
  };
}

function serve() {
  vi.stubGlobal(
    'fetch',
    fakeServer({
      [`GET ${VERSIONS}`]: () =>
        jsonResponse({
          format: 'maven2',
          group: 'org.acme',
          name: 'lib',
          versions: [
            {
              version: '2.0',
              repositories: ['releases', 'mirror'],
              componentIds: ['releases-2.0', 'mirror-2.0'],
              lastModified: '2026-10-02T10:00:00.000+00:00',
            },
            {
              version: '1.0',
              repositories: ['releases'],
              componentIds: ['releases-1.0'],
              lastModified: '2026-10-01T10:00:00.000+00:00',
            },
          ],
        }),
      'GET /service/rest/v1/components/releases-2.0': () => jsonResponse(component('2.0', 'releases')),
      'GET /service/rest/v1/components/mirror-2.0': () => jsonResponse(component('2.0', 'mirror')),
      'GET /service/rest/v1/components/releases-1.0': () => jsonResponse(component('1.0', 'releases')),
      'GET /service/rest/internal/ui/repositories': () =>
        jsonResponse([repository('releases', 'maven2', 'hosted'), repository('mirror', 'maven2', 'proxy')]),
    }).fetch,
  );
}

function page(path: string) {
  return renderWithApp(null, {
    path,
    state: { ...ADMIN_STATE, baseUrl: 'https://repo.example.org' },
    routes: [{ path: 'search/component/*', element: <ComponentPage /> }],
  });
}

const URL = `/search/component/${encodeURIComponent('maven2:org.acme:lib')}`;

afterEach(() => {
  vi.unstubAllGlobals();
});

describe('the component page', { timeout: 20_000 }, () => {
  it('reads the component from the address', () => {
    expect(keyFromPath('/search/component/maven2%3Aorg.acme%3Alib')).toBe('maven2:org.acme:lib');
    expect(keyFromPath('/search/component/docker%3A%3Ateam%2Fapp/')).toBe('docker::team/app');
    expect(keyFromPath('/elsewhere')).toBe('');
  });

  it('prefers the jar of a Maven component and the manifest of an image', () => {
    expect(mainAsset('maven2', component('1.0', 'r').assets)?.path).toBe('/org/acme/lib/1.0/lib-1.0.jar');
    const blob: SearchAsset = {
      ...(component('1.0', 'r').assets[0] as SearchAsset),
      path: '/v2/app/blobs/sha256:1',
      format: 'docker',
    };
    const manifest = { ...blob, path: '/v2/app/manifests/1.0' };
    expect(mainAsset('docker', [blob, manifest])?.path).toBe('/v2/app/manifests/1.0');
  });

  it('shows the newest version with its repositories and attributes', async () => {
    serve();
    page(URL);
    expect(await screen.findByRole('heading', { level: 1, name: 'org.acme:lib' })).toBeInTheDocument();
    expect(await screen.findByRole('combobox', { name: 'Version' })).toHaveTextContent('2.0');
    expect(screen.getByRole('link', { name: 'releases' })).toHaveAttribute('href', '/browse/releases');
    expect(screen.getByRole('link', { name: 'mirror' })).toBeInTheDocument();
    expect(await screen.findByText('artifactId')).toBeInTheDocument();
    expect(screen.getByText('jar')).toBeInTheDocument();
  });

  it('lists the files of the chosen version', async () => {
    serve();
    page(`${URL}?version=1.0&tab=files`);
    const table = await screen.findByRole('table', { name: 'Files of this version' });
    await waitFor(() => expect(within(table).getAllByRole('rowheader')).toHaveLength(2));
    expect(within(table).getByRole('link', { name: '/org/acme/lib/1.0/lib-1.0.jar' })).toHaveAttribute(
      'href',
      '/browse/releases/org/acme/lib/1.0/lib-1.0.jar',
    );
    expect(
      within(table).getByRole('link', { name: 'Download /org/acme/lib/1.0/lib-1.0.pom' }),
    ).toHaveAttribute('href', 'http://localhost/repository/releases/org/acme/lib/1.0/lib-1.0.pom');
  });

  it('gives usage for the exact version through the chosen repository', async () => {
    serve();
    const { router } = page(`${URL}?tab=usage`);
    expect(await screen.findByText(/<version>2\.0<\/version>/)).toBeInTheDocument();
    await userEvent.click(screen.getByRole('tab', { name: /settings\.xml/ }));
    expect(screen.getByText(/<activeProfile>releases<\/activeProfile>/)).toBeInTheDocument();
    await userEvent.click(screen.getByRole('combobox', { name: 'Repository' }));
    await userEvent.click(await screen.findByRole('option', { name: 'mirror' }));
    await userEvent.click(screen.getByRole('tab', { name: /settings\.xml/ }));
    expect(screen.getByText(/<mirrorOf>\*<\/mirrorOf>/)).toBeInTheDocument();
    await userEvent.click(screen.getByRole('tab', { name: /Versions/ }));
    await userEvent.click(await screen.findByRole('link', { name: '1.0' }));
    await waitFor(() => expect(router.state.location.search).toBe('?version=1.0'));
  });

  it('says when nobody may see the component', async () => {
    vi.stubGlobal(
      'fetch',
      fakeServer({
        [`GET ${VERSIONS}`]: () =>
          jsonResponse({ format: 'maven2', group: 'org.acme', name: 'lib', versions: [] }),
        'GET /service/rest/internal/ui/repositories': () => jsonResponse([]),
      }).fetch,
    );
    page(URL);
    expect(await screen.findByRole('heading', { name: 'Component not found' })).toBeInTheDocument();
  });

  it('names an address that is not a component', async () => {
    serve();
    page('/search/component/nonsense');
    expect(await screen.findByRole('heading', { level: 1, name: 'Unknown component' })).toBeInTheDocument();
  });
});
