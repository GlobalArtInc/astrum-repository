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

  it('give Berksfile, Policyfile and knife lines for Chef cookbooks', () => {
    const snippets = componentSnippets({
      ...base,
      format: 'chef',
      group: '',
      name: 'nginx',
      version: '12.3.4',
    });
    expect(snippets.map((snippet) => snippet.id)).toEqual(['berksfile', 'policyfile', 'knife']);
    const url = 'https://repo.example.org/repository/r';
    expect(snippets[0]?.code).toBe(`source "${url}"\n\ncookbook "nginx", "= 12.3.4"`);
    expect(snippets[0]?.hint).toBe('browse.component.snippet.chefBerksfile');
    expect(snippets[1]?.code).toContain(`default_source :supermarket, "${url}"`);
    expect(snippets[1]?.hint).toBe('browse.component.snippet.chefPolicyfile');
    expect(snippets[2]?.code).toBe(`knife supermarket download nginx 12.3.4 -m ${url}`);
  });

  it('give puppet module install and the Puppetfile line for Puppet modules', () => {
    expect(
      componentSnippets({ ...base, format: 'puppet', group: 'puppetlabs', name: 'stdlib', version: '9.6.0' }),
    ).toEqual([
      {
        id: 'install',
        label: 'puppet module install',
        code: 'puppet module install puppetlabs-stdlib --version 9.6.0 --module_repository https://repo.example.org/repository/r',
      },
      {
        id: 'puppetfile',
        label: 'Puppetfile',
        hint: 'browse.component.snippet.puppetfile',
        code: "mod 'puppetlabs-stdlib', '9.6.0'",
      },
    ]);
  });

  it('give vagrant box add and a Vagrantfile for Vagrant boxes', () => {
    const snippets = componentSnippets({
      ...base,
      format: 'vagrant',
      group: 'acme',
      name: 'base',
      version: '1.2.0',
    });
    expect(snippets.map((snippet) => snippet.id)).toEqual(['add', 'vagrantfile']);
    expect(snippets[0]?.code).toBe(
      'vagrant box add https://repo.example.org/repository/r/acme/base --box-version 1.2.0',
    );
    expect(snippets[1]?.code).toBe(
      'Vagrant.configure("2") do |config|\n  config.vm.box = "acme/base"\n  config.vm.box_url = "https://repo.example.org/repository/r/acme/base"\n  config.vm.box_version = "1.2.0"\nend',
    );
    expect(snippets[1]?.hint).toBe('browse.component.snippet.vagrantfile');
  });

  it('give the dependency and the registry line for Bazel modules', () => {
    const snippets = componentSnippets({
      ...base,
      format: 'bazel',
      group: '',
      name: 'rules_demo',
      version: '1.2.0',
    });
    expect(snippets).toEqual([
      {
        id: 'dependency',
        label: 'MODULE.bazel',
        hint: 'browse.component.snippet.bazelDependency',
        code: 'bazel_dep(name = "rules_demo", version = "1.2.0")',
      },
      {
        id: 'registry',
        label: '.bazelrc',
        hint: 'browse.component.snippet.bazelRegistry',
        code: 'common --registry=https://repo.example.org/repository/r',
      },
    ]);
  });

  it('give nix copy for Nix store paths', () => {
    const snippets = componentSnippets({
      ...base,
      format: 'nix',
      group: 'mw9ls4zxx884y7yq0a04ildfdjfy1vn2',
      name: 'nix',
      version: '2.34.9',
    });
    expect(snippets).toEqual([
      {
        id: 'copy',
        label: 'nix copy',
        hint: 'browse.component.snippet.nixCopy',
        code: 'nix copy --from https://repo.example.org/repository/r /nix/store/mw9ls4zxx884y7yq0a04ildfdjfy1vn2-nix-2.34.9',
      },
    ]);
    expect(componentSnippets({ ...base, format: 'nix', group: '', name: 'x', version: '' })).toEqual([]);
  });

  it('give mix.exs and rebar.config dependencies for Hex packages', () => {
    const hosted = componentSnippets({
      ...base,
      repository: 'hex-local',
      repositoryType: 'hosted',
      format: 'hex',
      group: '',
      name: 'hello',
      version: '1.4.2',
    });
    expect(hosted).toEqual([
      {
        id: 'mix',
        label: 'mix.exs',
        hint: 'browse.component.snippet.mixDeps',
        code: '{:hello, "~> 1.4", repo: "hex-local"}',
      },
      {
        id: 'rebar',
        label: 'rebar.config',
        hint: 'browse.component.snippet.rebarDeps',
        code: '{hello, "1.4.2"}',
      },
    ]);
    const proxied = componentSnippets({
      ...base,
      repository: 'hexpm',
      repositoryType: 'proxy',
      format: 'hex',
      group: '',
      name: 'jason',
      version: '1.4.4',
    });
    expect(proxied[0]?.code).toBe('{:jason, "~> 1.4"}');
  });

  it('give the editor command and ovsx get for Open VSX extensions', () => {
    const snippets = componentSnippets({
      ...base,
      format: 'openvsx',
      group: 'acme',
      name: 'hello',
      version: '1.2.0',
    });
    expect(snippets.map((snippet) => snippet.id)).toEqual(['install', 'get']);
    expect(snippets[0]?.code).toBe('codium --install-extension acme.hello@1.2.0');
    expect(snippets[0]?.hint).toBe('browse.component.snippet.openvsxInstall');
    expect(snippets[1]?.code).toBe(
      'npx ovsx get acme.hello -v 1.2.0 -r https://repo.example.org/repository/r',
    );
  });

  it('download JetBrains plugins by id and version', () => {
    const snippets = componentSnippets({
      ...base,
      format: 'jetbrains',
      group: '',
      name: 'Acme Tool',
      version: '1.2.0+b1',
    });
    expect(snippets.map((snippet) => snippet.id)).toEqual(['download']);
    expect(snippets[0]?.code).toBe(
      "curl -fLOJ 'https://repo.example.org/repository/r/plugin/download?pluginId=Acme%20Tool&version=1.2.0%2Bb1'",
    );
    expect(snippets[0]?.hint).toBe('browse.component.snippet.jetbrainsDownload');
  });

  it('install Chromium extensions by policy and download their packages', () => {
    const id = 'abcdefghijklmnopabcdefghijklmnop';
    const snippets = componentSnippets({ ...base, format: 'chrome', group: '', name: id, version: '2.0.1' });
    expect(snippets.map((snippet) => snippet.id)).toEqual(['policy', 'download']);
    expect(JSON.parse(snippets[0]?.code ?? '')).toEqual({
      ExtensionSettings: {
        [id]: {
          installation_mode: 'force_installed',
          update_url: 'https://repo.example.org/repository/r/updates.xml',
          override_update_url: true,
        },
      },
    });
    expect(snippets[0]?.hint).toBe('browse.component.snippet.chromePolicy');
    expect(snippets[1]?.code).toBe(
      `curl -fLO 'https://repo.example.org/repository/r/${id}/2.0.1/${id}-2.0.1.crx'`,
    );
  });

  it('install Firefox add-ons by policy and download one version', () => {
    const snippets = componentSnippets({
      ...base,
      format: 'firefox',
      group: '',
      name: 'name@example.org',
      version: '1.0+b1',
    });
    expect(snippets.map((snippet) => snippet.id)).toEqual(['policy', 'download']);
    expect(JSON.parse(snippets[0]?.code ?? '')).toEqual({
      policies: {
        ExtensionSettings: {
          'name@example.org': {
            installation_mode: 'force_installed',
            install_url: 'https://repo.example.org/repository/r/name@example.org/latest.xpi',
          },
        },
      },
    });
    expect(snippets[0]?.hint).toBe('browse.component.snippet.firefoxPolicy');
    expect(snippets[1]?.code).toBe(
      "curl -fLo 'name-1.0+b1.xpi' 'https://repo.example.org/repository/r/name@example.org/1.0%2Bb1.xpi'",
    );
    expect(snippets[1]?.hint).toBe('browse.component.snippet.firefoxDownload');
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
    expect(
      componentSnippets({ ...base, format: 'pacman', group: 'x86_64', name: 'hello', version: '1:1.0-2' }),
    ).toEqual([{ id: 'pacman', label: 'pacman', code: 'sudo pacman -S hello' }]);
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
