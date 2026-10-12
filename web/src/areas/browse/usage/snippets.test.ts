import { describe, expect, it } from 'vitest';
import { dockerImage, encodePath, repositoryUrl, type UsageContext, usageSnippets } from './snippets';

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
    expect(code(snippets, 'settings')).toContain(`<username>\${env.ASTRUM_USERNAME}</username>`);
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
    expect(code(snippets, 'pypirc')).toContain('repository = https://repo.example.org/repository/py/');
    expect(code(snippets, 'twine')).toBe('twine upload --repository py dist/*');
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
    expect(code(snippets, 'pull')).toBe('docker pull repo.example.org:8443/docker-hosted/team/app:1.2');
    expect(code(snippets, 'login')).toBe('docker login repo.example.org:8443');
    expect(code(snippets, 'dockerfile')).toBe('FROM repo.example.org:8443/docker-hosted/team/app:1.2');
    expect(code(snippets, 'push')).toBe(
      'docker tag team/app:1.2 repo.example.org:8443/docker-hosted/team/app:1.2\ndocker push repo.example.org:8443/docker-hosted/team/app:1.2',
    );
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
    expect(code(snippets, 'pull')).toBe('docker pull 127.0.0.1:18321/docker-hosted/app@sha256:abc');
    expect(ids(snippets)).not.toContain('push');
    expect(code(snippets, 'insecure')).toContain('"insecure-registries": ["127.0.0.1:18321"]');
  });

  it('shows placeholders for a whole proxy', () => {
    const snippets = usageSnippets(
      context({ format: 'docker', repositoryName: 'hub', repositoryType: 'proxy' }),
    );
    expect(code(snippets, 'pull')).toBe('docker pull repo.example.org/hub/<image>:<tag>');
    expect(ids(snippets)).toEqual(['pull', 'login', 'dockerfile']);
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
