import { describe, expect, it } from 'vitest';
import {
  dockerImage,
  encodePath,
  repositoryUrl,
  supportsUsage,
  type UsageContext,
  usageSnippets,
} from './snippets';

const BASE = 'https://repo.example.org';

function context(overrides: Partial<UsageContext>): UsageContext {
  return {
    format: 'raw',
    repositoryName: 'files',
    repositoryType: 'hosted',
    baseUrl: BASE,
    ...overrides,
  };
}

const ids = (snippets: { id: string }[]) => snippets.map((snippet) => snippet.id);
const code = (snippets: { id: string; code: string }[], id: string) =>
  snippets.find((snippet) => snippet.id === id)?.code ?? '';

describe('repository addresses', () => {
  it('end with a slash and encode what needs it', () => {
    expect(repositoryUrl('http://h:8081/', 'maven releases')).toBe(
      'http://h:8081/repository/maven%20releases/',
    );
    expect(encodePath('/a b/c#.txt')).toBe('a%20b/c%23.txt');
  });
});

describe('maven usage', () => {
  const component = { group: 'org.acme', name: 'lib', version: '1.0' };

  it('gives the dependency, Gradle, settings and deployment for a hosted component', () => {
    const snippets = usageSnippets(context({ format: 'maven2', repositoryName: 'releases', component }));
    expect(ids(snippets)).toEqual(['maven', 'gradle-kotlin', 'gradle-groovy', 'settings', 'deploy']);
    expect(code(snippets, 'maven')).toBe(
      '<dependency>\n  <groupId>org.acme</groupId>\n  <artifactId>lib</artifactId>\n  <version>1.0</version>\n</dependency>',
    );
    expect(code(snippets, 'gradle-kotlin')).toContain(
      'url = uri("https://repo.example.org/repository/releases/")',
    );
    expect(code(snippets, 'gradle-kotlin')).toContain('implementation("org.acme:lib:1.0")');
    expect(code(snippets, 'gradle-groovy')).toContain("implementation 'org.acme:lib:1.0'");
    expect(code(snippets, 'settings')).toContain('<activeProfile>releases</activeProfile>');
    expect(code(snippets, 'settings')).not.toContain('<servers>');
    expect(code(snippets, 'deploy')).toContain('<repository>');
  });

  it('mirrors everything through a group or proxy and deploys nothing there', () => {
    const snippets = usageSnippets(
      context({ format: 'maven2', repositoryName: 'public', repositoryType: 'group' }),
    );
    expect(ids(snippets)).toEqual(['gradle-kotlin', 'gradle-groovy', 'settings']);
    expect(code(snippets, 'settings')).toContain('<mirrorOf>*</mirrorOf>');
    expect(code(snippets, 'settings')).toContain('<url>https://repo.example.org/repository/public/</url>');
    expect(code(snippets, 'gradle-kotlin')).not.toContain('publishing');
  });

  it('publishes with Gradle to a hosted repository and keeps snapshots apart', () => {
    const hosted = usageSnippets(context({ format: 'maven2', repositoryName: 'releases' }));
    expect(code(hosted, 'gradle-kotlin')).toContain('credentials(PasswordCredentials::class)');
    expect(code(hosted, 'gradle-groovy')).toContain('credentials(PasswordCredentials)');
    const snapshot = usageSnippets(
      context({ format: 'maven2', component: { ...component, version: '1.1-SNAPSHOT' } }),
    );
    expect(code(snapshot, 'deploy')).toContain('<snapshotRepository>');
  });

  it('names classifiers and unusual types', () => {
    const snippets = usageSnippets(
      context({ format: 'maven2', component, attributes: { classifier: 'sources', extension: 'zip' } }),
    );
    expect(code(snippets, 'maven')).toContain('<classifier>sources</classifier>\n  <type>zip</type>');
    expect(code(snippets, 'gradle-kotlin')).toContain('implementation("org.acme:lib:1.0:sources@zip")');
    const jar = usageSnippets(context({ format: 'maven2', component, attributes: { extension: 'jar' } }));
    expect(code(jar, 'maven')).not.toContain('<type>');
  });
});

describe('npm usage', () => {
  it('installs a scoped package with a scoped registry line and publishes to hosted', () => {
    const snippets = usageSnippets(
      context({
        format: 'npm',
        repositoryName: 'npm-internal',
        component: { group: 'team', name: 'tool', version: '2.0.0' },
      }),
    );
    expect(code(snippets, 'npm')).toBe(
      'npm install @team/tool@2.0.0 --registry=https://repo.example.org/repository/npm-internal/',
    );
    expect(code(snippets, 'npmrc')).toBe('@team:registry=https://repo.example.org/repository/npm-internal/');
    expect(code(snippets, 'publish')).toContain('npm publish --registry=');
  });

  it('only installs and signs in through a group', () => {
    const snippets = usageSnippets(
      context({ format: 'npm', repositoryName: 'npm-all', repositoryType: 'group' }),
    );
    expect(ids(snippets)).toEqual(['npm', 'yarn', 'pnpm', 'npmrc', 'login']);
    expect(code(snippets, 'npm')).toContain('npm install <package>');
    expect(code(snippets, 'npmrc')).toBe('registry=https://repo.example.org/repository/npm-all/');
  });
});

describe('pypi usage', () => {
  it('uses the simple index and twine for hosted repositories', () => {
    const snippets = usageSnippets(
      context({
        format: 'pypi',
        repositoryName: 'py',
        component: { group: '', name: 'tool', version: '0.3.0' },
      }),
    );
    expect(code(snippets, 'pip')).toBe(
      'pip install --index-url https://repo.example.org/repository/py/simple/ tool==0.3.0',
    );
    expect(ids(snippets)).not.toContain('pypirc');
    expect(code(snippets, 'twine')).toBe(
      'twine upload --repository-url https://repo.example.org/repository/py/ dist/*',
    );
  });

  it('trusts a plain HTTP host and uploads nothing through a proxy', () => {
    const snippets = usageSnippets(
      context({
        format: 'pypi',
        repositoryName: 'pypi-proxy',
        repositoryType: 'proxy',
        baseUrl: 'http://10.0.0.5:8081',
      }),
    );
    expect(ids(snippets)).toEqual(['pip', 'pip-conf', 'requirements']);
    expect(code(snippets, 'pip')).toContain('--trusted-host 10.0.0.5 <package>');
    expect(code(snippets, 'pip-conf')).toContain('trusted-host = 10.0.0.5');
  });
});

describe('docker usage', () => {
  it('pulls and pushes by path on the server host', () => {
    const snippets = usageSnippets(
      context({
        format: 'docker',
        repositoryName: 'docker-hosted',
        baseUrl: 'https://repo.example.org:8443',
        component: { group: '', name: 'team/app', version: '1.2' },
      }),
    );
    const image = 'repo.example.org:8443/docker-hosted/team/app:1.2';
    const blocks = (id: string) =>
      snippets.find((snippet) => snippet.id === id)?.blocks?.map((block) => [block.label, block.code]);
    expect(blocks('docker')).toEqual([
      ['Pull', `docker pull ${image}`],
      ['Login', 'docker login repo.example.org:8443'],
      ['Push', `docker tag team/app:1.2 ${image}\ndocker push ${image}`],
    ]);
    expect(blocks('podman')).toEqual([
      ['Pull', `podman pull ${image}`],
      ['Login', 'podman login repo.example.org:8443'],
      ['Push', `podman tag team/app:1.2 ${image}\npodman push ${image}`],
    ]);
    expect(code(snippets, 'dockerfile')).toBe(`FROM ${image}`);
    expect(ids(snippets)).not.toContain('insecure');
  });

  it('reads manifests by digest from the path and warns about plain HTTP', () => {
    const digest = { path: '/v2/app/manifests/sha256:abc' };
    expect(dockerImage(context(digest))).toEqual({ image: 'app', reference: 'sha256:abc' });
    const snippets = usageSnippets(
      context({
        format: 'docker',
        repositoryName: 'docker-hosted',
        baseUrl: 'http://127.0.0.1:18321',
        ...digest,
      }),
    );
    expect(snippets[0]?.blocks?.map((block) => block.code)).toEqual([
      'docker pull 127.0.0.1:18321/docker-hosted/app@sha256:abc',
      'docker login 127.0.0.1:18321',
    ]);
    expect(ids(snippets)).not.toContain('insecure');
  });

  it('shows placeholders for a whole proxy', () => {
    const snippets = usageSnippets(
      context({ format: 'docker', repositoryName: 'hub', repositoryType: 'proxy' }),
    );
    expect(snippets[1]?.blocks?.map((block) => block.id)).toEqual(['pull', 'login']);
    expect(snippets[1]?.blocks?.[0]?.code).toBe('podman pull repo.example.org/hub/<image>:<tag>');
    expect(ids(snippets)).toEqual(['docker', 'podman', 'dockerfile', 'compose', 'kubernetes']);
  });
});

describe('repository domains', () => {
  const root = {
    host: 'docker-prem.example.org',
    routing: 'ROOT' as const,
    url: 'https://docker-prem.example.org/',
  };
  const shared = {
    host: 'images.example.org',
    routing: 'PATH' as const,
    path: 'prod',
    url: 'https://images.example.org/prod/',
  };

  it('name images by the domain of the repository', () => {
    const component = { group: '', name: 'team/app', version: '1.2' };
    const pulls = (domain: typeof root | typeof shared) =>
      usageSnippets(
        context({ format: 'docker', repositoryName: 'docker-onprem', component, domain }),
      )[0]?.blocks?.map((block) => block.code);
    expect(pulls(root)).toEqual([
      'docker pull docker-prem.example.org/team/app:1.2',
      'docker login docker-prem.example.org',
      'docker tag team/app:1.2 docker-prem.example.org/team/app:1.2\ndocker push docker-prem.example.org/team/app:1.2',
    ]);
    expect(pulls(shared)?.[0]).toBe('docker pull images.example.org/prod/team/app:1.2');
    expect(pulls(shared)?.[1]).toBe('docker login images.example.org');
  });

  it('give other formats the address on the domain', () => {
    const snippets = usageSnippets(context({ path: '/docs/a.txt', domain: shared }));
    expect(code(snippets, 'curl')).toBe('curl -fLO https://images.example.org/prod/docs/a.txt');
  });
});

describe('raw usage', () => {
  it('downloads and uploads a known file', () => {
    const snippets = usageSnippets(context({ path: '/docs/read me.txt' }));
    expect(code(snippets, 'curl')).toBe(
      'curl -fLO https://repo.example.org/repository/files/docs/read%20me.txt',
    );
    expect(code(snippets, 'upload')).toBe(
      'curl -u <user> --upload-file ./read me.txt https://repo.example.org/repository/files/docs/read%20me.txt',
    );
    expect(snippets.find((snippet) => snippet.id === 'upload')?.labelKey).toBe('browse.usage.label.upload');
  });

  it('uses a placeholder path for the repository and downloads only from groups', () => {
    const snippets = usageSnippets(context({ repositoryType: 'group', repositoryName: 'all' }));
    expect(ids(snippets)).toEqual(['curl', 'wget']);
    expect(code(snippets, 'wget')).toBe('wget https://repo.example.org/repository/all/path/to/file.txt');
  });

  it('has nothing for formats it does not know', () => {
    expect(usageSnippets(context({ format: 'no-such-format' }))).toEqual([]);
  });
});

describe('helm usage', () => {
  const component = { group: '', name: 'demo', version: '0.5.0' };

  it('adds the repository, installs, declares and uploads a hosted chart', () => {
    const snippets = usageSnippets(context({ format: 'helm', repositoryName: 'charts', component }));
    expect(ids(snippets)).toEqual(['repo', 'install', 'dependency', 'upload']);
    expect(code(snippets, 'repo')).toBe(
      'helm repo add charts https://repo.example.org/repository/charts/\nhelm repo update',
    );
    expect(code(snippets, 'install')).toBe('helm install demo charts/demo --version 0.5.0');
    expect(code(snippets, 'dependency')).toBe(
      'dependencies:\n  - name: demo\n    version: 0.5.0\n    repository: https://repo.example.org/repository/charts/',
    );
    expect(code(snippets, 'upload')).toBe(
      'curl -u <user> --upload-file demo-0.5.0.tgz https://repo.example.org/repository/charts/demo-0.5.0.tgz',
    );
    expect(snippets.find((snippet) => snippet.id === 'repo')?.hint).toBe('browse.usage.hint.helmRepo');
    expect(snippets.find((snippet) => snippet.id === 'upload')?.hint).toBe('browse.usage.hint.helmUpload');
  });

  it('only reads from proxies and groups and uses placeholders without a chart', () => {
    const snippets = usageSnippets(
      context({ format: 'helm', repositoryName: 'all-charts', repositoryType: 'group' }),
    );
    expect(ids(snippets)).toEqual(['repo', 'install', 'dependency']);
    expect(code(snippets, 'install')).toBe('helm install <chart> all-charts/<chart>');
    expect(code(snippets, 'dependency')).not.toContain('version:');
    const hosted = usageSnippets(context({ format: 'helm', repositoryName: 'charts' }));
    expect(code(hosted, 'upload')).toContain('--upload-file <chart>-<version>.tgz');
  });
});

describe('nuget usage', () => {
  const component = { group: '', name: 'Astrum.Probe', version: '1.0.0' };

  it('adds the package, registers the source and pushes to a hosted repository', () => {
    const snippets = usageSnippets(context({ format: 'nuget', repositoryName: 'nuget-hosted', component }));
    expect(ids(snippets)).toEqual(['dotnet', 'source', 'nuget', 'config', 'push']);
    const index = 'https://repo.example.org/repository/nuget-hosted/index.json';
    expect(code(snippets, 'dotnet')).toBe(
      `dotnet add package Astrum.Probe --version 1.0.0 --source ${index}`,
    );
    expect(code(snippets, 'source')).toBe(`dotnet nuget add source ${index} --name nuget-hosted`);
    expect(code(snippets, 'nuget')).toBe(`nuget sources add -Name nuget-hosted -Source ${index}`);
    expect(code(snippets, 'config')).toBe(
      `<configuration>\n  <packageSources>\n    <add key="nuget-hosted" value="${index}" />\n  </packageSources>\n</configuration>`,
    );
    expect(code(snippets, 'push')).toBe(
      'dotnet nuget push Astrum.Probe.1.0.0.nupkg --source https://repo.example.org/repository/nuget-hosted/ --api-key <api-key>',
    );
    expect(snippets.find((snippet) => snippet.id === 'push')?.hint).toBe('browse.usage.hint.nugetPush');
  });

  it('only reads from proxies and groups and works without a component', () => {
    const group = usageSnippets(
      context({ format: 'nuget', repositoryName: 'nuget-group', repositoryType: 'group' }),
    );
    expect(ids(group)).toEqual(['source', 'nuget', 'config']);
    const proxy = usageSnippets(
      context({ format: 'nuget', repositoryName: 'nuget.org', repositoryType: 'proxy', component }),
    );
    expect(ids(proxy)).toEqual(['dotnet', 'source', 'nuget', 'config']);
  });
});

describe('yum usage', () => {
  const component = { group: '', name: 'astrum-dep', version: '1:2.0-3' };

  it('writes a .repo file at the repodata directory, installs and uploads a hosted package', () => {
    const snippets = usageSnippets(
      context({
        format: 'yum',
        repositoryName: 'rpms',
        component,
        path: 'el9/x86_64/Packages/astrum-dep-2.0-3.noarch.rpm',
      }),
    );
    expect(ids(snippets)).toEqual(['repo', 'install', 'upload']);
    expect(code(snippets, 'repo')).toBe(
      '[rpms]\nname=rpms\nbaseurl=https://repo.example.org/repository/rpms/el9/x86_64/\nenabled=1\ngpgcheck=0',
    );
    expect(snippets[0]?.label).toBe('rpms.repo');
    expect(snippets[0]?.hintVars).toEqual({ name: 'rpms' });
    expect(code(snippets, 'install')).toBe('sudo dnf install astrum-dep-1:2.0-3');
    expect(code(snippets, 'upload')).toBe(
      'curl -u <user> --upload-file ./astrum-dep-2.0-3.noarch.rpm https://repo.example.org/repository/rpms/el9/x86_64/Packages/astrum-dep-2.0-3.noarch.rpm',
    );
    expect(snippets.find((snippet) => snippet.id === 'upload')?.hint).toBe('browse.usage.hint.yumUpload');
  });

  it('only reads from proxies and groups and uses placeholders without a package', () => {
    const group = usageSnippets(
      context({ format: 'yum', repositoryName: 'all-rpms', repositoryType: 'group' }),
    );
    expect(ids(group)).toEqual(['repo']);
    expect(code(group, 'repo')).toContain('baseurl=https://repo.example.org/repository/all-rpms/\n');
    const hosted = usageSnippets(context({ format: 'yum', repositoryName: 'rpms' }));
    expect(code(hosted, 'upload')).toBe(
      'curl -u <user> --upload-file ./<package>.rpm https://repo.example.org/repository/rpms/<package>.rpm',
    );
  });
});

describe('rubygems usage', () => {
  const component = { group: '', name: 'hola', version: '1.1.0' };

  it('installs, adds the source and pushes to a hosted repository', () => {
    const snippets = usageSnippets(
      context({
        format: 'rubygems',
        repositoryName: 'gems',
        component,
        path: '/gems/hola-1.1.0-x86_64-linux.gem',
      }),
    );
    expect(ids(snippets)).toEqual(['install', 'gemfile', 'sources', 'push']);
    expect(code(snippets, 'install')).toBe(
      'gem install hola -v 1.1.0 --clear-sources --source https://repo.example.org/repository/gems/',
    );
    expect(code(snippets, 'gemfile')).toBe(
      'source "https://repo.example.org/repository/gems/"\n\ngem "hola", "1.1.0"',
    );
    expect(code(snippets, 'sources')).toBe('gem sources --add https://repo.example.org/repository/gems/');
    expect(code(snippets, 'push')).toBe(
      'export RUBYGEMS_HOST=https://repo.example.org/repository/gems\ngem signin\ngem push hola-1.1.0-x86_64-linux.gem',
    );
    expect(snippets.find((snippet) => snippet.id === 'push')?.hint).toBe('browse.usage.hint.gemPush');
  });

  it('mirrors rubygems.org through a proxy and only reads from groups', () => {
    const proxy = usageSnippets(
      context({ format: 'rubygems', repositoryName: 'gems-proxy', repositoryType: 'proxy' }),
    );
    expect(ids(proxy)).toEqual(['gemfile', 'mirror']);
    expect(code(proxy, 'gemfile')).toBe('source "https://repo.example.org/repository/gems-proxy/"');
    expect(code(proxy, 'mirror')).toBe(
      'bundle config set --global mirror.https://rubygems.org https://repo.example.org/repository/gems-proxy/',
    );
    const group = usageSnippets(
      context({ format: 'rubygems', repositoryName: 'all', repositoryType: 'group', component }),
    );
    expect(ids(group)).toEqual(['install', 'gemfile', 'sources']);
    const hosted = usageSnippets(context({ format: 'rubygems', repositoryName: 'gems' }));
    expect(code(hosted, 'push')).toContain('gem push <name>-<version>.gem');
  });
});

describe('git lfs usage', () => {
  const oid = 'ffc3a223711122b05d9924078a85f112f36e97c1dd987ebf1a2f44a39817380e';

  it('points git at the API and downloads a selected object', () => {
    const snippets = usageSnippets(
      context({
        format: 'gitlfs',
        repositoryName: 'lfs',
        component: { group: '', name: oid, version: '' },
        path: `/objects/ff/c3/${oid}`,
      }),
    );
    expect(ids(snippets)).toEqual(['lfsconfig', 'git-config', 'curl']);
    expect(code(snippets, 'lfsconfig')).toBe(
      'git config -f .lfsconfig lfs.url https://repo.example.org/repository/lfs/info/lfs\ngit add .lfsconfig',
    );
    expect(code(snippets, 'git-config')).toBe(
      'git config lfs.url https://repo.example.org/repository/lfs/info/lfs',
    );
    expect(code(snippets, 'curl')).toBe(
      `curl -u <user> -fL -o ${oid} https://repo.example.org/repository/lfs/info/lfs/objects/${oid}`,
    );
  });

  it('leaves the download out without an object', () => {
    expect(ids(usageSnippets(context({ format: 'gitlfs', repositoryName: 'lfs' })))).toEqual([
      'lfsconfig',
      'git-config',
    ]);
  });
});

describe('cargo usage', () => {
  const component = { group: '', name: 'demo', version: '0.1.0' };

  it('registers the index, adds the crate and publishes to a hosted registry', () => {
    const snippets = usageSnippets(context({ format: 'cargo', repositoryName: 'crates', component }));
    expect(ids(snippets)).toEqual(['config', 'add', 'manifest', 'publish']);
    expect(code(snippets, 'config')).toBe(
      '[registries.crates]\nindex = "sparse+https://repo.example.org/repository/crates/"',
    );
    expect(code(snippets, 'add')).toBe('cargo add demo@0.1.0 --registry crates');
    expect(code(snippets, 'manifest')).toBe(
      '[dependencies]\ndemo = { version = "0.1.0", registry = "crates" }',
    );
    expect(code(snippets, 'publish')).toBe('cargo login --registry crates\ncargo publish --registry crates');
  });

  it('replaces crates.io with a proxy and publishes nowhere but hosted', () => {
    const proxy = usageSnippets(
      context({ format: 'cargo', repositoryName: 'crates-io', repositoryType: 'proxy', component }),
    );
    expect(ids(proxy)).toEqual(['config', 'add', 'manifest']);
    expect(code(proxy, 'config')).toBe(
      '[source.crates-io]\nreplace-with = "crates-io"\n\n[source.crates-io]\nregistry = "sparse+https://repo.example.org/repository/crates-io/"',
    );
    expect(code(proxy, 'add')).toBe('cargo add demo@0.1.0');
    expect(code(proxy, 'manifest')).toBe('[dependencies]\ndemo = "0.1.0"');
    const group = usageSnippets(context({ format: 'cargo', repositoryName: 'all', repositoryType: 'group' }));
    expect(ids(group)).toEqual(['config']);
  });
});

describe('go usage', () => {
  const component = { group: '', name: 'example.com/Astrum/hello', version: 'v1.0.0' };

  it('gets the module, keeps it out of the checksum database and uploads to hosted', () => {
    const snippets = usageSnippets(context({ format: 'go', repositoryName: 'modules', component }));
    expect(ids(snippets)).toEqual(['get', 'env', 'mod', 'upload']);
    expect(code(snippets, 'get')).toBe(
      'GOPROXY=https://repo.example.org/repository/modules go get example.com/Astrum/hello@v1.0.0',
    );
    expect(code(snippets, 'env')).toBe(
      'go env -w GOPROXY=https://repo.example.org/repository/modules,direct\ngo env -w GONOSUMDB=example.com/Astrum/hello',
    );
    expect(code(snippets, 'mod')).toBe('require example.com/Astrum/hello v1.0.0');
    expect(code(snippets, 'upload')).toBe(
      'curl -u <user> --upload-file v1.0.0.zip https://repo.example.org/repository/modules/example.com/!astrum/hello/@v/v1.0.0.zip',
    );
  });

  it('only sets the proxy for proxies and groups', () => {
    const proxy = usageSnippets(context({ format: 'go', repositoryName: 'golang', repositoryType: 'proxy' }));
    expect(ids(proxy)).toEqual(['get', 'env', 'mod']);
    expect(code(proxy, 'env')).toBe('go env -w GOPROXY=https://repo.example.org/repository/golang,direct');
    expect(code(proxy, 'get')).toBe(
      'GOPROXY=https://repo.example.org/repository/golang go get <module>@<version>',
    );
    const group = usageSnippets(
      context({ format: 'go', repositoryName: 'all', repositoryType: 'group', component }),
    );
    expect(snippetsHint(group, 'env')).toBe('browse.usage.hint.goEnv');
  });
});

function snippetsHint(snippets: ReturnType<typeof usageSnippets>, id: string): string | undefined {
  return snippets.find((snippet) => snippet.id === id)?.hint;
}

describe('apt usage', () => {
  const component = { group: 'amd64', name: 'hello-astrum', version: '1.0.0' };

  it('adds the source line, installs and uploads to a hosted repository', () => {
    const snippets = usageSnippets(
      context({
        format: 'apt',
        repositoryName: 'debs',
        component,
        distribution: 'bookworm',
        path: '/pool/h/hello-astrum/hello-astrum_1.0.0_amd64.deb',
      }),
    );
    expect(ids(snippets)).toEqual(['sources', 'install', 'upload']);
    expect(code(snippets, 'sources')).toBe('deb https://repo.example.org/repository/debs/ bookworm main');
    expect(snippets[0]?.hintVars).toEqual({ name: 'debs' });
    expect(code(snippets, 'install')).toBe('sudo apt-get update\nsudo apt-get install hello-astrum=1.0.0');
    expect(code(snippets, 'upload')).toBe(
      'curl -u <user> -H "Content-Type: multipart/form-data" --data-binary "@hello-astrum_1.0.0_amd64.deb" https://repo.example.org/repository/debs/',
    );
  });

  it('only reads from proxies and uses placeholders without a distribution', () => {
    const proxy = usageSnippets(
      context({ format: 'apt', repositoryName: 'debian', repositoryType: 'proxy' }),
    );
    expect(ids(proxy)).toEqual(['sources']);
    expect(code(proxy, 'sources')).toBe(
      'deb https://repo.example.org/repository/debian/ <distribution> main',
    );
  });
});

describe('composer usage', () => {
  const component = { group: 'acme', name: 'hello', version: '1.0.0' };

  it('adds the repository, requires the package and uploads to hosted', () => {
    const snippets = usageSnippets(context({ format: 'composer', repositoryName: 'php', component }));
    expect(ids(snippets)).toEqual(['config', 'require', 'json', 'upload']);
    expect(code(snippets, 'config')).toBe(
      'composer config repositories.php composer https://repo.example.org/repository/php/',
    );
    expect(code(snippets, 'require')).toBe('composer require acme/hello:1.0.0');
    expect(code(snippets, 'json')).toBe(
      '"repositories": [\n  { "type": "composer", "url": "https://repo.example.org/repository/php/" }\n]',
    );
    expect(code(snippets, 'upload')).toBe(
      'curl -u <user> --upload-file package.zip https://repo.example.org/repository/php/packages/upload/acme/hello/1.0.0',
    );
  });

  it('only reads from proxies and groups', () => {
    const proxy = usageSnippets(
      context({ format: 'composer', repositoryName: 'packagist', repositoryType: 'proxy' }),
    );
    expect(ids(proxy)).toEqual(['config', 'json']);
  });
});

describe('conan usage', () => {
  it('adds the remote, installs and uploads to hosted', () => {
    const snippets = usageSnippets(
      context({
        format: 'conan',
        repositoryName: 'conan',
        component: { group: '', name: 'zlib', version: '1.3.1' },
      }),
    );
    expect(ids(snippets)).toEqual(['remote', 'install', 'conanfile', 'upload']);
    expect(code(snippets, 'remote')).toBe(
      'conan remote add conan https://repo.example.org/repository/conan/',
    );
    expect(code(snippets, 'install')).toBe('conan install --requires=zlib/1.3.1 -r conan');
    expect(code(snippets, 'conanfile')).toBe('[requires]\nzlib/1.3.1');
    expect(code(snippets, 'upload')).toBe('conan upload zlib/1.3.1 -r conan');
  });

  it('names the user and channel and reads only from proxies', () => {
    const proxy = usageSnippets(
      context({
        format: 'conan',
        repositoryName: 'center',
        repositoryType: 'proxy',
        component: { group: 'acme/stable', name: 'tool', version: '2.0' },
      }),
    );
    expect(ids(proxy)).toEqual(['remote', 'install', 'conanfile']);
    expect(code(proxy, 'install')).toBe('conan install --requires=tool/2.0@acme/stable -r center');
  });
});

describe('cocoapods usage', () => {
  const component = { group: '', name: 'AFNetworking', version: '4.0.1' };

  it('gives the Podfile source, the CDN repository and the netrc entry', () => {
    const snippets = usageSnippets(
      context({ format: 'cocoapods', repositoryName: 'trunk', repositoryType: 'proxy', component }),
    );
    expect(ids(snippets)).toEqual(['podfile', 'repo', 'netrc']);
    expect(code(snippets, 'podfile')).toBe(
      "source 'https://repo.example.org/repository/trunk/'\n\ntarget '<App>' do\n  pod 'AFNetworking', '4.0.1'\nend",
    );
    expect(code(snippets, 'repo')).toBe('pod repo add-cdn trunk https://repo.example.org/repository/trunk/');
    expect(code(snippets, 'netrc')).toBe('machine repo.example.org\nlogin <user>\npassword <password>');
  });

  it('names a placeholder pod for the repository itself', () => {
    const snippets = usageSnippets(
      context({ format: 'cocoapods', repositoryName: 'trunk', repositoryType: 'proxy' }),
    );
    expect(code(snippets, 'podfile')).toContain("  pod '<pod>'\n");
  });
});

describe('conda usage', () => {
  it('installs from the channel of a package and uploads to its subdir', () => {
    const snippets = usageSnippets(
      context({
        format: 'conda',
        repositoryName: 'conda-hosted',
        component: { group: 'main', name: 'hello', version: '1.0' },
        path: '/main/noarch/hello-1.0-py_0.tar.bz2',
      }),
    );
    expect(ids(snippets)).toEqual(['install', 'condarc', 'mamba', 'upload']);
    const channel = 'https://repo.example.org/repository/conda-hosted/main';
    expect(code(snippets, 'install')).toBe(`conda install -c ${channel} --override-channels hello=1.0`);
    expect(code(snippets, 'condarc')).toBe(`channels:\n  - ${channel}`);
    expect(code(snippets, 'mamba')).toBe(`micromamba install -c ${channel} --override-channels hello=1.0`);
    expect(code(snippets, 'upload')).toBe(
      `curl -u <user> --upload-file hello-1.0-py_0.tar.bz2 ${channel}/noarch/hello-1.0-py_0.tar.bz2`,
    );
  });

  it('uses the repository itself as the channel of a proxy and only reads from it', () => {
    const proxy = usageSnippets(
      context({ format: 'conda', repositoryName: 'conda-forge', repositoryType: 'proxy' }),
    );
    expect(ids(proxy)).toEqual(['install', 'condarc', 'mamba']);
    expect(code(proxy, 'install')).toBe(
      'conda install -c https://repo.example.org/repository/conda-forge --override-channels <package>',
    );
    const index = usageSnippets(
      context({ format: 'conda', repositoryName: 'conda-hosted', path: '/team/main/linux-64/repodata.json' }),
    );
    expect(code(index, 'upload')).toBe(
      'curl -u <user> --upload-file <package>.conda https://repo.example.org/repository/conda-hosted/team/main/linux-64/<package>.conda',
    );
  });
});

describe('r usage', () => {
  const component = { group: '', name: 'astrumhello', version: '1.1.0' };

  it('installs from a hosted repository next to the configured ones and uploads to its path', () => {
    const snippets = usageSnippets(
      context({
        format: 'r',
        repositoryName: 'r-hosted',
        component,
        path: '/src/contrib/astrumhello_1.1.0.tar.gz',
      }),
    );
    expect(ids(snippets)).toEqual(['install', 'profile', 'description', 'upload']);
    expect(code(snippets, 'install')).toBe(
      'install.packages("astrumhello", repos = c("https://repo.example.org/repository/r-hosted", getOption("repos")))',
    );
    expect(code(snippets, 'profile')).toBe(
      'options(repos = c("r-hosted" = "https://repo.example.org/repository/r-hosted", getOption("repos")))',
    );
    expect(code(snippets, 'description')).toBe('Imports: astrumhello (>= 1.1.0)');
    expect(code(snippets, 'upload')).toBe(
      'curl -u <user> --upload-file astrumhello_1.1.0.tar.gz https://repo.example.org/repository/r-hosted/src/contrib/astrumhello_1.1.0.tar.gz',
    );
  });

  it('keeps the path of binary packages and names a source package without one', () => {
    const binary = usageSnippets(
      context({
        format: 'r',
        repositoryName: 'r-hosted',
        component,
        path: '/bin/windows/contrib/4.4/astrumhello_1.1.0.zip',
      }),
    );
    expect(code(binary, 'upload')).toBe(
      'curl -u <user> --upload-file astrumhello_1.1.0.zip https://repo.example.org/repository/r-hosted/bin/windows/contrib/4.4/astrumhello_1.1.0.zip',
    );
    const index = usageSnippets(
      context({ format: 'r', repositoryName: 'r-hosted', path: '/src/contrib/PACKAGES' }),
    );
    expect(ids(index)).toEqual(['install', 'profile', 'upload']);
    expect(code(index, 'upload')).toBe(
      'curl -u <user> --upload-file <package>_<version>.tar.gz https://repo.example.org/repository/r-hosted/src/contrib/<package>_<version>.tar.gz',
    );
  });

  it('replaces CRAN with proxies and groups and does not upload to them', () => {
    const proxy = usageSnippets(context({ format: 'r', repositoryName: 'cran', repositoryType: 'proxy' }));
    expect(ids(proxy)).toEqual(['install', 'profile']);
    expect(code(proxy, 'install')).toBe(
      'install.packages("<package>", repos = "https://repo.example.org/repository/cran")',
    );
    expect(code(proxy, 'profile')).toBe(
      'options(repos = c(CRAN = "https://repo.example.org/repository/cran"))',
    );
    expect(supportsUsage('r')).toBe(true);
  });
});

describe('alpine usage', () => {
  const component = { group: 'x86_64', name: 'hello', version: '1.0-r0' };

  it('saves the key, adds the branch, installs and uploads to a signed hosted repository', () => {
    const snippets = usageSnippets(
      context({
        format: 'alpine',
        repositoryName: 'apks',
        component,
        alpineKey: 'apks.rsa.pub',
        path: '/v3.20/main/x86_64/hello-1.0-r0.apk',
      }),
    );
    expect(ids(snippets)).toEqual(['repositories', 'add', 'upload']);
    expect(code(snippets, 'repositories')).toBe(
      'wget -O /etc/apk/keys/apks.rsa.pub https://repo.example.org/repository/apks/apks.rsa.pub\necho "https://repo.example.org/repository/apks/v3.20/main" >> /etc/apk/repositories\napk update',
    );
    expect(snippetsHint(snippets, 'repositories')).toBe('browse.usage.hint.apkRepositories');
    expect(code(snippets, 'add')).toBe('apk add hello=1.0-r0');
    expect(code(snippets, 'upload')).toBe(
      'curl -u <user> --upload-file hello-1.0-r0.apk https://repo.example.org/repository/apks/v3.20/main/x86_64/hello-1.0-r0.apk',
    );
  });

  it('asks for --allow-untrusted without a key and uses placeholders without a path', () => {
    const proxy = usageSnippets(
      context({ format: 'alpine', repositoryName: 'alpine', repositoryType: 'proxy' }),
    );
    expect(ids(proxy)).toEqual(['repositories']);
    expect(code(proxy, 'repositories')).toBe(
      'echo "https://repo.example.org/repository/alpine/<branch>" >> /etc/apk/repositories\napk update --allow-untrusted',
    );
    expect(snippetsHint(proxy, 'repositories')).toBe('browse.usage.hint.apkUntrusted');
    const root = usageSnippets(
      context({ format: 'alpine', repositoryName: 'apks', component, path: '/x86_64/hello-1.0-r0.apk' }),
    );
    expect(code(root, 'repositories')).toContain('echo "https://repo.example.org/repository/apks" >>');
    expect(code(root, 'add')).toBe('apk add --allow-untrusted hello=1.0-r0');
    const bare = usageSnippets(context({ format: 'alpine', repositoryName: 'apks', component }));
    expect(code(bare, 'upload')).toBe(
      'curl -u <user> --upload-file hello-1.0-r0.apk https://repo.example.org/repository/apks/<branch>/x86_64/hello-1.0-r0.apk',
    );
  });
});

describe('pub usage', () => {
  const component = { group: '', name: 'astrum_hello', version: '1.1.0' };

  it('adds a token, the hosted dependency and publish_to for a hosted package', () => {
    const snippets = usageSnippets(
      context({
        format: 'pub',
        repositoryName: 'pub-hosted',
        component,
        path: '/packages/astrum_hello/versions/1.1.0.tar.gz',
      }),
    );
    expect(ids(snippets)).toEqual(['token', 'dependency', 'add', 'publish']);
    expect(code(snippets, 'token')).toBe('dart pub token add https://repo.example.org/repository/pub-hosted');
    expect(code(snippets, 'dependency')).toBe(
      'dependencies:\n  astrum_hello:\n    hosted: https://repo.example.org/repository/pub-hosted\n    version: ^1.1.0',
    );
    expect(code(snippets, 'add')).toBe(
      "dart pub add 'astrum_hello:{hosted: https://repo.example.org/repository/pub-hosted, version: ^1.1.0}'",
    );
    expect(code(snippets, 'publish')).toBe('publish_to: https://repo.example.org/repository/pub-hosted');
  });

  it('replaces pub.dev with proxies and groups and names no package without one', () => {
    const group = usageSnippets(
      context({ format: 'pub', repositoryName: 'pub-group', repositoryType: 'group' }),
    );
    expect(ids(group)).toEqual(['token', 'dependency', 'mirror']);
    expect(code(group, 'dependency')).toBe(
      'dependencies:\n  <package>:\n    hosted: https://repo.example.org/repository/pub-group\n    version: any',
    );
    expect(code(group, 'mirror')).toBe(
      'export PUB_HOSTED_URL=https://repo.example.org/repository/pub-group\ndart pub get',
    );
    expect(supportsUsage('pub')).toBe(true);
  });
});

describe('ansible usage', () => {
  it('configures the server, installs a collection and publishes to a hosted repository', () => {
    const snippets = usageSnippets(
      context({
        format: 'ansible',
        repositoryName: 'galaxy-hosted',
        component: { group: 'acme', name: 'tools', version: '1.2.0' },
        path: '/acme/tools/1.2.0/acme-tools-1.2.0.tar.gz',
      }),
    );
    expect(ids(snippets)).toEqual(['config', 'install', 'requirements', 'publish']);
    const url = 'https://repo.example.org/repository/galaxy-hosted/';
    expect(code(snippets, 'config')).toBe(
      `[galaxy]\nserver_list = galaxy_hosted\n\n[galaxy_server.galaxy_hosted]\nurl = ${url}\ntoken = <access token>`,
    );
    expect(code(snippets, 'install')).toBe(`ansible-galaxy collection install acme.tools:1.2.0 -s ${url}`);
    expect(code(snippets, 'requirements')).toBe(
      `collections:\n  - name: acme.tools\n    version: "1.2.0"\n    source: ${url}`,
    );
    expect(code(snippets, 'publish')).toBe(
      `ansible-galaxy collection publish acme-tools-1.2.0.tar.gz -s ${url} --token <access token>`,
    );
  });

  it('reads from proxies and groups without a component and offers no publishing', () => {
    const snippets = usageSnippets(
      context({ format: 'ansible', repositoryName: 'galaxy', repositoryType: 'proxy', path: '/api/' }),
    );
    expect(ids(snippets)).toEqual(['config', 'install', 'requirements']);
    expect(code(snippets, 'install')).toBe(
      'ansible-galaxy collection install <namespace>.<name> -s https://repo.example.org/repository/galaxy/',
    );
    expect(code(snippets, 'requirements')).toContain('version: "*"');
    const hosted = usageSnippets(context({ format: 'ansible', repositoryName: 'own' }));
    expect(code(hosted, 'publish')).toContain('publish <namespace>-<name>-<version>.tar.gz');
  });
});

describe('chef usage', () => {
  it('points Berkshelf, Policyfiles and knife at the repository and publishes with curl', () => {
    const snippets = usageSnippets(
      context({
        format: 'chef',
        repositoryName: 'chef-hosted',
        component: { group: '', name: 'acme', version: '1.2.0' },
        path: '/cookbooks/acme/1.2.0/acme-1.2.0.tar.gz',
      }),
    );
    expect(ids(snippets)).toEqual(['berksfile', 'policyfile', 'knife', 'publish']);
    const url = 'https://repo.example.org/repository/chef-hosted';
    expect(code(snippets, 'berksfile')).toBe(`source "${url}"\n\ncookbook "acme", "= 1.2.0"`);
    expect(code(snippets, 'policyfile')).toBe(
      `default_source :supermarket, "${url}"\n\nrun_list "acme::default"\ncookbook "acme", "= 1.2.0"`,
    );
    expect(code(snippets, 'knife')).toBe(`knife supermarket download acme 1.2.0 -m ${url}`);
    expect(code(snippets, 'publish')).toBe(
      [
        'knife cookbook metadata from file acme/metadata.rb',
        'tar -czf acme.tgz acme',
        `curl -u <user> -F 'cookbook={"category":"Other"}' -F tarball=@acme.tgz ${url}/api/v1/cookbooks`,
      ].join('\n'),
    );
    expect(supportsUsage('chef')).toBe(true);
  });

  it('names no cookbook without one and offers no publishing outside hosted repositories', () => {
    const snippets = usageSnippets(
      context({ format: 'chef', repositoryName: 'supermarket', repositoryType: 'proxy', path: '/universe' }),
    );
    expect(ids(snippets)).toEqual(['berksfile', 'policyfile', 'knife']);
    expect(code(snippets, 'berksfile')).toBe(
      'source "https://repo.example.org/repository/supermarket"\n\ncookbook "<cookbook>"',
    );
    expect(code(snippets, 'knife')).toBe(
      'knife supermarket download <cookbook> -m https://repo.example.org/repository/supermarket',
    );
    const hosted = usageSnippets(context({ format: 'chef', repositoryName: 'own' }));
    expect(code(hosted, 'publish')).toContain('tarball=@<name>.tgz');
  });
});

describe('puppet usage', () => {
  it('configures the clients, installs a module and publishes to a hosted repository', () => {
    const snippets = usageSnippets(
      context({
        format: 'puppet',
        repositoryName: 'forge-hosted',
        component: { group: 'acme', name: 'tools', version: '1.2.0' },
        path: '/acme/tools/1.2.0/acme-tools-1.2.0.tar.gz',
      }),
    );
    expect(ids(snippets)).toEqual(['config', 'install', 'r10k', 'bolt', 'puppetfile', 'publish', 'curl']);
    const url = 'https://repo.example.org/repository/forge-hosted';
    expect(code(snippets, 'config')).toBe(
      `[main]\nmodule_repository = ${url}\nforge_authorization = <access token>`,
    );
    expect(code(snippets, 'install')).toBe(
      `puppet module install acme-tools --version 1.2.0 --module_repository ${url}`,
    );
    expect(code(snippets, 'r10k')).toBe(
      `forge:\n  baseurl: '${url}'\n  authorization_token: 'Bearer <access token>'`,
    );
    expect(code(snippets, 'bolt')).toBe(
      `module-install:\n  forge:\n    baseurl: ${url}\n    authorization_token: Bearer <access token>`,
    );
    expect(code(snippets, 'puppetfile')).toBe(`forge '${url}'\n\nmod 'acme-tools', '1.2.0'`);
    expect(code(snippets, 'publish')).toBe(
      `pdk build\npdk release publish --forge-upload-url ${url}/v3/releases --forge-token <access token>`,
    );
    expect(code(snippets, 'curl')).toBe(
      `curl -H "Authorization: Bearer <access token>" -F file=@pkg/acme-tools-1.2.0.tar.gz ${url}/v3/releases`,
    );
    expect(supportsUsage('puppet')).toBe(true);
  });

  it('reads from proxies and groups without a component and offers no publishing', () => {
    const snippets = usageSnippets(
      context({ format: 'puppet', repositoryName: 'forge', repositoryType: 'group', path: '/v3/' }),
    );
    expect(ids(snippets)).toEqual(['config', 'install', 'r10k', 'bolt', 'puppetfile']);
    expect(code(snippets, 'install')).toBe(
      'puppet module install <owner>-<name> --module_repository https://repo.example.org/repository/forge',
    );
    expect(code(snippets, 'puppetfile')).toContain("mod '<owner>-<name>'");
    const hosted = usageSnippets(context({ format: 'puppet', repositoryName: 'own' }));
    expect(code(hosted, 'curl')).toContain('file=@pkg/<owner>-<name>-<version>.tar.gz');
  });
});

describe('vagrant usage', () => {
  it('points Vagrant at the server, names the box in a Vagrantfile and uploads to a hosted repository', () => {
    const snippets = usageSnippets(
      context({
        format: 'vagrant',
        repositoryName: 'boxes',
        component: { group: 'acme', name: 'base', version: '1.2.0' },
        path: '/acme/base/1.2.0/virtualbox/amd64.box',
      }),
    );
    expect(ids(snippets)).toEqual(['server', 'vagrantfile', 'upload']);
    const url = 'https://repo.example.org/repository/boxes';
    expect(code(snippets, 'server')).toBe(
      `export VAGRANT_SERVER_URL=${url}\nexport VAGRANT_CLOUD_TOKEN=<access token>\nvagrant box add acme/base --box-version 1.2.0`,
    );
    expect(code(snippets, 'vagrantfile')).toBe(
      [
        'Vagrant.configure("2") do |config|',
        '  config.vm.box = "acme/base"',
        `  config.vm.box_url = "${url}/acme/base"`,
        '  config.vm.box_version = "1.2.0"',
        'end',
      ].join('\n'),
    );
    expect(code(snippets, 'upload')).toBe(
      `curl -u <user> --upload-file package.box ${url}/acme/base/1.2.0/virtualbox/amd64.box`,
    );
    expect(supportsUsage('vagrant')).toBe(true);
  });

  it('reads from proxies and groups without a component and offers no upload', () => {
    const snippets = usageSnippets(
      context({ format: 'vagrant', repositoryName: 'vagrant-all', repositoryType: 'group' }),
    );
    expect(ids(snippets)).toEqual(['server', 'vagrantfile']);
    expect(code(snippets, 'server').endsWith('vagrant box add <org>/<name>')).toBe(true);
    expect(code(snippets, 'vagrantfile')).not.toContain('box_version');
    const fromPath = usageSnippets(
      context({
        format: 'vagrant',
        repositoryName: 'cache',
        repositoryType: 'proxy',
        path: '/hashicorp/bionic64/1.0.282/virtualbox/unknown.box',
      }),
    );
    expect(code(fromPath, 'vagrantfile')).toContain('config.vm.box = "hashicorp/bionic64"');
    const hosted = usageSnippets(context({ format: 'vagrant', repositoryName: 'own' }));
    expect(code(hosted, 'upload')).toBe(
      'curl -u <user> --upload-file package.box https://repo.example.org/repository/own/<org>/<name>/<version>/<provider>/<architecture>.box',
    );
  });
});
