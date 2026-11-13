import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import type { Handler } from '../../../test/http';
import { ADMIN_STATE, fakeServer, jsonResponse, renderWithApp } from '../../../test/render';
import { repository } from '../test-fixtures';
import type { SearchAsset, SearchComponent } from '../types';
import ComponentPage, { keyFromPath, locate, mainAsset } from './ComponentPage';
import { componentGuideUrl, componentSnippets } from './snippets';
import { csv } from './tables';

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

function serve(extra: Record<string, Handler> = {}) {
  const server = fakeServer({
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
    ...extra,
  });
  vi.stubGlobal('fetch', server.fetch);
  return server;
}

function waitingAt(version: string, continuationToken: string | null = null) {
  return jsonResponse({
    items: [
      {
        repository: 'mirror',
        format: 'maven2',
        namespace: 'org.acme',
        name: 'lib',
        version,
        publishedAt: '2026-11-10T08:00:00Z',
        availableAt: '2026-11-17T08:00:00Z',
        blocked: true,
        refusals: 1,
        firstRefusedAt: null,
      },
    ],
    continuationToken,
  });
}

const CAPPED = 'GET /service/rest/v1/security/cooldown?repository=mirror';

function cooldownCalls(server: { calls: Array<{ path: string }> }) {
  return server.calls.filter((call) => call.path.includes('/security/cooldown')).length;
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
  it('reads the component and the tab from the address', () => {
    expect(keyFromPath('/search/component/maven2%3Aorg.acme%3Alib')).toBe('maven2:org.acme:lib');
    expect(keyFromPath('/search/component/docker%3A%3Ateam%2Fapp/')).toBe('docker::team/app');
    expect(locate('/search/component/maven2%3Aorg.acme%3Alib/files')).toEqual({
      key: 'maven2:org.acme:lib',
      tab: 'files',
    });
    expect(keyFromPath('/elsewhere')).toBe('');
  });

  it('builds the dependency snippets and the CSV', () => {
    const context = {
      group: 'org.acme',
      name: 'lib',
      version: '2.0',
      baseUrl: 'https://r.example',
      repository: 'x',
    };
    expect(componentSnippets({ ...context, format: 'maven2' }).map((snippet) => snippet.label)).toEqual([
      'Apache Maven',
      'Gradle Groovy DSL',
      'Gradle Kotlin DSL',
      'Scala SBT',
      'Apache Ivy',
      'Groovy Grape',
      'Leiningen',
      'Apache Buildr',
      'Maven Central Badge',
      'PURL',
    ]);
    expect(componentSnippets({ ...context, format: 'docker', group: '', name: 'team/app' })[0]?.code).toBe(
      'docker pull r.example/x/team/app:2.0',
    );
    expect(componentSnippets({ ...context, format: 'docker', group: '', name: 'team/app' })[1]).toEqual({
      id: 'podman',
      label: 'Podman',
      code: 'podman pull r.example/x/team/app:2.0',
    });
    expect(componentSnippets({ ...context, format: 'helm' })).toEqual([]);
    expect(componentSnippets({ ...context, format: 'conda', group: 'main', name: 'hello' })[0]?.code).toBe(
      'conda install -c https://r.example/repository/x/main hello=2.0',
    );
    expect(componentSnippets({ ...context, format: 'conda', group: '', name: 'hello' })[0]?.code).toBe(
      'conda install -c https://r.example/repository/x hello=2.0',
    );
    expect(componentSnippets({ ...context, format: 'rubygems', group: '', name: 'hola' })).toEqual([
      { id: 'install', label: 'Install', code: 'gem install hola -v 2.0' },
    ]);
    expect(componentGuideUrl('rubygems', '', 'hola', '2.0')).toBe(
      'https://guide.globalart.dev/component/gem/hola/2.0?referrer=search-component-detail',
    );
    expect(
      csv([
        ['a,b', 'c"d'],
        ['e', 'f'],
      ]),
    ).toBe('"a,b","c""d"\ne,f\n');
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

  it('shows the cooldown line of a version that waits at a proxy', async () => {
    serve({ 'GET /service/rest/v1/security/cooldown?repository=mirror': () => waitingAt('2.0') });
    page(URL);
    expect(await screen.findByText(/Cooldown: published/)).toBeInTheDocument();
  });

  it('shows no cooldown line when the waiting list has no row for the version', async () => {
    const server = serve({
      'GET /service/rest/v1/security/cooldown?repository=mirror': () => waitingAt('3.0'),
    });
    page(URL);
    expect(await screen.findByRole('heading', { level: 1, name: 'lib v2.0' })).toBeInTheDocument();
    await waitFor(() => expect(server.calls.some((call) => call.path.includes('/cooldown'))).toBe(true));
    expect(screen.queryByText(/Cooldown: published/)).not.toBeInTheDocument();
  });

  it('gives up the cooldown line after four pages of the waiting list', async () => {
    const server = serve({
      [CAPPED]: () => waitingAt('3.0', 'page-2'),
      [`${CAPPED}&continuationToken=page-2`]: () => waitingAt('3.0', 'page-3'),
      [`${CAPPED}&continuationToken=page-3`]: () => waitingAt('3.0', 'page-4'),
      [`${CAPPED}&continuationToken=page-4`]: () => waitingAt('3.0', 'page-5'),
      [`${CAPPED}&continuationToken=page-5`]: () => waitingAt('2.0'),
    });
    page(URL);
    expect(await screen.findByRole('heading', { level: 1, name: 'lib v2.0' })).toBeInTheDocument();
    await waitFor(() => expect(cooldownCalls(server)).toBe(4));
    await new Promise((resolve) => setTimeout(resolve, 50));
    expect(cooldownCalls(server)).toBe(4);
    expect(screen.queryByText(/Cooldown: published/)).not.toBeInTheDocument();
  });

  it('offers the guide only when a version came from a proxy', async () => {
    serve();
    page(`${URL}?version=1.0`);
    expect(await screen.findByRole('heading', { level: 1, name: 'lib v1.0' })).toBeInTheDocument();
    expect(screen.queryByRole('link', { name: 'Research in Guide' })).not.toBeInTheDocument();
  });

  it('shows the newest version with its dependency snippet and details', async () => {
    serve();
    page(URL);
    expect(await screen.findByRole('heading', { level: 1, name: 'lib v2.0' })).toBeInTheDocument();
    expect(screen.getByRole('link', { name: 'Registry' })).toHaveAttribute(
      'href',
      'https://central.globalart.dev/artifact/org.acme/lib',
    );
    expect(screen.getByRole('link', { name: 'Research in Guide' })).toHaveAttribute(
      'href',
      'https://guide.globalart.dev/component/maven/org.acme%3Alib/2.0?referrer=search-component-detail',
    );
    expect(screen.getByRole('heading', { name: 'Dependencies' })).toBeInTheDocument();
    expect(screen.getByText(/<version>2\.0<\/version>/)).toBeInTheDocument();
    expect(screen.getByText('Paste this snippet into pom.xml')).toBeInTheDocument();
    expect(screen.getByText('releases, mirror')).toBeInTheDocument();
    const tabs = screen.getByRole('navigation', { name: 'Component sections' });
    expect(within(tabs).getByRole('link', { name: /Versions\s*2/ })).toHaveAttribute(
      'href',
      `${URL}/versions?version=2.0`,
    );
  });

  it('lists the files of the chosen version with their type and menu', async () => {
    serve();
    page(`${URL}/files?version=1.0`);
    const table = await screen.findByRole('table', { name: 'Files' });
    await waitFor(() => expect(within(table).getAllByRole('rowheader')).toHaveLength(2));
    expect(within(table).getByText('lib-1.0.jar')).toBeInTheDocument();
    expect(within(table).getAllByText('releases')).toHaveLength(2);
    const filters = screen.getByRole('complementary', { name: 'Filters' });
    await userEvent.click(within(filters).getByRole('checkbox', { name: 'pom' }));
    expect(within(table).getAllByRole('rowheader')).toHaveLength(1);
    await userEvent.click(within(table).getByRole('button', { name: 'Actions for lib-1.0.pom' }));
    expect(await screen.findByRole('menuitem', { name: 'Open in Repository' })).toBeInTheDocument();
  });

  it('lists versions and repositories and moves between versions', async () => {
    serve();
    const { router } = page(`${URL}/versions`);
    const table = await screen.findByRole('table', { name: 'Versions' });
    expect(
      within(table)
        .getAllByRole('rowheader')
        .map((cell) => cell.textContent),
    ).toEqual(['2.0', '1.0']);
    expect(within(table).getByText('2 repositories')).toBeInTheDocument();
    await userEvent.click(within(table).getByRole('link', { name: '1.0' }));
    await waitFor(() => expect(router.state.location.search).toBe('?version=1.0'));
    expect(await screen.findByRole('heading', { level: 1, name: 'lib v1.0' })).toBeInTheDocument();
    await userEvent.click(screen.getByRole('link', { name: /Repositories\s*2/ }));
    const repositories = await screen.findByRole('table', { name: 'Repositories' });
    expect(within(repositories).getByText('2 versions')).toBeInTheDocument();
    expect(within(repositories).getByText('Proxy')).toBeInTheDocument();
    expect(screen.queryByRole('link', { name: /^Security/ })).not.toBeInTheDocument();
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
    expect(await screen.findByRole('heading', { level: 1, name: 'Component not found' })).toBeInTheDocument();
  });

  it('names an address that is not a component', async () => {
    serve();
    page('/search/component/nonsense');
    expect(await screen.findByRole('heading', { level: 1, name: 'Unknown component' })).toBeInTheDocument();
  });
});
