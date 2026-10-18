import { describe, expect, it } from 'vitest';
import { componentGuideUrl, componentSnippets } from './snippets';

const base = { baseUrl: 'https://repo.example.org', repository: 'r' };

describe('component snippets of the M4c formats', () => {
  it('give cargo add and composer require with the project command hint', () => {
    const cargo = componentSnippets({ ...base, format: 'cargo', group: '', name: 'serde', version: '1.0.0' });
    expect(cargo).toEqual([
      {
        id: 'cargo',
        label: 'cargo',
        hint: 'browse.component.snippet.projectCommand',
        code: 'cargo add serde@1.0.0',
      },
    ]);
    const composer = componentSnippets({
      ...base,
      format: 'composer',
      group: 'psr',
      name: 'log',
      version: '3.0.2',
    });
    expect(composer[0]?.code).toBe('composer require psr/log:3.0.2');
    expect(
      componentSnippets({ ...base, format: 'cocoapods', group: '', name: 'AFNetworking', version: '4.0.1' }),
    ).toEqual([
      {
        id: 'podfile',
        label: 'Podfile',
        hint: 'browse.component.snippet.podfile',
        code: "pod 'AFNetworking', '4.0.1'",
      },
    ]);
    expect(
      componentSnippets({ ...base, format: 'go', group: '', name: 'example.com/m', version: 'v1.0.0' }),
    ).toEqual([]);
  });

  it('give install.packages and the DESCRIPTION line for R packages', () => {
    expect(
      componentSnippets({
        ...base,
        repository: 'r-group',
        format: 'r',
        group: '',
        name: 'astrumhello',
        version: '1.1.0',
      }),
    ).toEqual([
      {
        id: 'install',
        label: 'install.packages',
        code: 'install.packages("astrumhello", repos = "https://repo.example.org/repository/r-group")',
      },
      {
        id: 'description',
        label: 'DESCRIPTION',
        hint: 'browse.component.snippet.rDescription',
        code: 'Imports: astrumhello (>= 1.1.0)',
      },
    ]);
  });

  it('give dart pub add and the pubspec.yaml dependency for pub packages', () => {
    expect(
      componentSnippets({
        ...base,
        repository: 'pub-group',
        format: 'pub',
        group: '',
        name: 'astrum_hello',
        version: '1.0.0-dev.2',
      }),
    ).toEqual([
      {
        id: 'add',
        label: 'dart pub add',
        hint: 'browse.component.snippet.projectCommand',
        code: "dart pub add 'astrum_hello:{hosted: https://repo.example.org/repository/pub-group, version: ^1.0.0-dev.2}'",
      },
      {
        id: 'pubspec',
        label: 'pubspec.yaml',
        hint: 'browse.component.snippet.pubspec',
        code: 'dependencies:\n  astrum_hello:\n    hosted: https://repo.example.org/repository/pub-group\n    version: ^1.0.0-dev.2',
      },
    ]);
  });

  it('give ansible-galaxy and requirements.yml for Ansible collections', () => {
    const snippets = componentSnippets({
      ...base,
      format: 'ansible',
      group: 'community',
      name: 'general',
      version: '8.0.0',
    });
    expect(snippets.map((snippet) => snippet.id)).toEqual(['install', 'requirements']);
    expect(snippets[0]?.code).toBe(
      'ansible-galaxy collection install community.general:8.0.0 -s https://repo.example.org/repository/r/',
    );
    expect(snippets[1]?.code).toBe(
      'collections:\n  - name: community.general\n    version: "8.0.0"\n    source: https://repo.example.org/repository/r/',
    );
    expect(snippets[1]?.hint).toBe('browse.component.snippet.ansibleRequirements');
  });

  it('link cargo components to the guide', () => {
    expect(componentGuideUrl('cargo', '', 'serde', '1.0.0')).toBe(
      'https://guide.globalart.dev/component/cargo/serde/1.0.0?referrer=search-component-detail',
    );
    expect(componentGuideUrl('go', '', 'example.com/m', 'v1.0.0')).toBeNull();
  });

  it('give scoped npm packages and maven artifacts their registry names in the guide', () => {
    expect(componentGuideUrl('npm', 'babel', 'core', '7.24.0')).toBe(
      'https://guide.globalart.dev/component/npm/%40babel%2Fcore/7.24.0?referrer=search-component-detail',
    );
    expect(componentGuideUrl('maven2', 'org.acme', 'lib', '2.0')).toBe(
      'https://guide.globalart.dev/component/maven/org.acme%3Alib/2.0?referrer=search-component-detail',
    );
  });
});

describe('component snippets of Alpine', () => {
  it('install the exact version with apk add', () => {
    expect(
      componentSnippets({ ...base, format: 'alpine', group: 'x86_64', name: 'hello', version: '1.0-r0' }),
    ).toEqual([{ id: 'apk', label: 'apk', code: 'apk add hello=1.0-r0' }]);
  });
});

describe('component snippets on a repository domain', () => {
  it('pull images from the domain', () => {
    const image = { ...base, format: 'docker', group: '', name: 'team/app', version: '1.2' };
    expect(componentSnippets(image)[0]?.code).toBe('docker pull repo.example.org/r/team/app:1.2');
    const root = {
      host: 'docker-prem.example.org',
      routing: 'ROOT' as const,
      url: 'https://docker-prem.example.org/',
    };
    expect(componentSnippets({ ...image, domain: root })[0]?.code).toBe(
      'docker pull docker-prem.example.org/team/app:1.2',
    );
    const shared = {
      host: 'images.example.org',
      routing: 'PATH' as const,
      url: 'https://images.example.org/r/',
    };
    expect(componentSnippets({ ...image, domain: shared })[1]?.code).toBe(
      'podman pull images.example.org/r/team/app:1.2',
    );
  });
});
