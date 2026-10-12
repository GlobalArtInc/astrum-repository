import { getUsageSnippets, USAGE_GENERATORS } from '..';
import type { UsageContext } from '../types';

const BASE = 'https://astrum.example.com/repository';

function context(overrides: Partial<UsageContext> & Pick<UsageContext, 'format' | 'repositoryName'>): UsageContext {
  return {
    repositoryUrl: `${BASE}/${overrides.repositoryName}/`,
    repositoryType: 'hosted',
    ...overrides,
  };
}

function texts(ctx: UsageContext): Record<string, string> {
  const map: Record<string, string> = {};
  for (const snippet of getUsageSnippets(ctx)) {
    map[snippet.id] = snippet.snippetText;
  }
  return map;
}

function ids(ctx: UsageContext): string[] {
  return getUsageSnippets(ctx).map((snippet) => snippet.id);
}

describe('usage snippets', () => {
  it('has a repository-aware generator for every format the server supports', () => {
    expect(Object.keys(USAGE_GENERATORS).sort()).toEqual([
      'alpine', 'apt', 'cargo', 'cocoapods', 'composer', 'conan', 'conda', 'docker', 'gitlfs', 'go', 'helm',
      'huggingface', 'maven2', 'npm', 'nuget', 'p2', 'pypi', 'r', 'raw', 'swift', 'yum',
    ]);
  });

  it('every snippet carries an id, a label and text', () => {
    const contexts: UsageContext[] = Object.keys(USAGE_GENERATORS).map((format) =>
      context({ format, repositoryName: `${format}-hosted`, group: 'acme', name: 'tool', version: '1.0.0', path: 'a/b/tool-1.0.0.bin' }),
    );
    for (const ctx of contexts) {
      const snippets = getUsageSnippets(ctx);
      expect(snippets.length).toBeGreaterThan(0);
      for (const snippet of snippets) {
        expect(snippet.id).toBeTruthy();
        expect(snippet.label).toBeTruthy();
        expect(snippet.snippetText.trim()).not.toBe('');
      }
      expect(new Set(snippets.map((snippet) => snippet.id)).size).toBe(snippets.length);
    }
  });

  describe('docker', () => {
    const tag = context({
      format: 'docker',
      repositoryName: 'docker-hosted',
      name: 'herald-api',
      version: '0.1.0',
      path: '/v2/herald-api/manifests/0.1.0',
      registryUrl: 'astrum.example.com/docker-hosted',
    });

    it('offers pull, login, Dockerfile, Compose, Kubernetes and push for a hosted repository', () => {
      expect(ids(tag)).toEqual(['pull', 'login', 'dockerfile', 'compose', 'kubernetes', 'push']);
    });

    it('addresses the image through the registry of the repository', () => {
      const s = texts(tag);
      expect(s.pull).toBe('docker pull astrum.example.com/docker-hosted/herald-api:0.1.0');
      expect(s.login).toBe('docker login astrum.example.com');
      expect(s.dockerfile).toBe('FROM astrum.example.com/docker-hosted/herald-api:0.1.0');
      expect(s.compose).toBe('services:\n  herald-api:\n    image: astrum.example.com/docker-hosted/herald-api:0.1.0');
      expect(s.kubernetes).toBe(
        'spec:\n  containers:\n  - name: herald-api\n    image: astrum.example.com/docker-hosted/herald-api:0.1.0',
      );
      expect(s.push).toBe(
        'docker tag herald-api:0.1.0 astrum.example.com/docker-hosted/herald-api:0.1.0\n' +
          'docker push astrum.example.com/docker-hosted/herald-api:0.1.0',
      );
    });

    it('signs in to the connector host and port when the registry is a connector', () => {
      const s = texts({ ...tag, registryUrl: 'astrum.example.com:8443' });
      expect(s.login).toBe('docker login astrum.example.com:8443');
      expect(s.pull).toBe('docker pull astrum.example.com:8443/herald-api:0.1.0');
    });

    it('falls back to the repository path on the server host without a registry', () => {
      expect(texts({ ...tag, registryUrl: null }).pull).toBe(
        'docker pull astrum.example.com/docker-hosted/herald-api:0.1.0',
      );
    });

    it('pulls a manifest stored under a digest by digest and reads the image from the path', () => {
      const digest = 'sha256:2afc9a932abb547de79b8723d4a6bf1c38e87767aaa4ee08205c692d50221ba1';
      const s = texts({ ...tag, name: null, version: null, path: `v2/library/nginx/manifests/${encodeURIComponent(digest)}` });
      expect(s.pull).toBe(`docker pull astrum.example.com/docker-hosted/library/nginx@${digest}`);
      expect(s.push).toBeUndefined();
    });

    it('does not offer push for a proxy', () => {
      expect(ids({ ...tag, repositoryType: 'proxy' })).not.toContain('push');
    });

    it('ignores a registry that is not a plain host and path', () => {
      expect(texts({ ...tag, registryUrl: 'evil.com"; rm -rf /' }).pull).toBe(
        'docker pull astrum.example.com/docker-hosted/herald-api:0.1.0',
      );
    });
  });

  describe('maven2', () => {
    const jar = context({
      format: 'maven2',
      repositoryName: 'maven-releases',
      group: 'dev.globalart.herald',
      name: 'herald-core',
      version: '1.3.0',
      attributes: { maven2: { extension: 'jar' } },
    });

    it('offers Maven, Gradle Kotlin and Groovy, settings.xml and deploy for a hosted repository', () => {
      expect(ids(jar)).toEqual(['maven', 'gradle-kotlin', 'gradle-groovy', 'settings', 'deploy']);
    });

    it('writes the dependency and Gradle builds with the repository URL', () => {
      const s = texts(jar);
      expect(s.maven).toBe(
        '<dependency>\n  <groupId>dev.globalart.herald</groupId>\n  <artifactId>herald-core</artifactId>\n' +
          '  <version>1.3.0</version>\n</dependency>',
      );
      expect(s['gradle-kotlin']).toBe(
        'repositories {\n    maven {\n        url = uri("https://astrum.example.com/repository/maven-releases/")\n    }\n}\n\n' +
          'dependencies {\n    implementation("dev.globalart.herald:herald-core:1.3.0")\n}',
      );
      expect(s['gradle-groovy']).toContain("url 'https://astrum.example.com/repository/maven-releases/'");
      expect(s['gradle-groovy']).toContain("implementation 'dev.globalart.herald:herald-core:1.3.0'");
    });

    it('adds a hosted repository as a profile and deploys releases to it', () => {
      const s = texts(jar);
      expect(s.settings).toContain('<activeProfile>maven-releases</activeProfile>');
      expect(s.settings).toContain('<url>https://astrum.example.com/repository/maven-releases/</url>');
      expect(s.settings).not.toContain('<mirrorOf>');
      expect(s.deploy).toContain('<repository>\n    <id>maven-releases</id>');
    });

    it('deploys a snapshot to the snapshot repository element', () => {
      expect(texts({ ...jar, version: '1.4.0-SNAPSHOT' }).deploy).toContain('<snapshotRepository>');
    });

    it('offers a group or proxy as a mirror of everything and no deploy', () => {
      const group = { ...jar, repositoryName: 'maven-public', repositoryUrl: `${BASE}/maven-public/`, repositoryType: 'group' };
      const s = texts(group);
      expect(s.settings).toContain('<mirrorOf>*</mirrorOf>');
      expect(s.settings).toContain('<url>https://astrum.example.com/repository/maven-public/</url>');
      expect(ids(group)).not.toContain('deploy');
    });

    it('describes the component, not the pom file, for a pom asset', () => {
      expect(texts({ ...jar, attributes: { maven2: { extension: 'pom' } } })['gradle-kotlin']).toContain(
        'implementation("dev.globalart.herald:herald-core:1.3.0")',
      );
    });

    it('keeps a classifier', () => {
      const s = texts({ ...jar, attributes: { maven2: { extension: 'jar', classifier: 'sources' } } });
      expect(s.maven).toContain('<classifier>sources</classifier>');
      expect(s['gradle-kotlin']).toContain('implementation("dev.globalart.herald:herald-core:1.3.0:sources")');
    });

    it('names an extension other than jar, which Gradle would not resolve otherwise', () => {
      const s = texts({ ...jar, attributes: { maven2: { extension: 'zip' } } });
      expect(s.maven).toContain('<type>zip</type>');
      expect(s['gradle-kotlin']).toContain('implementation("dev.globalart.herald:herald-core:1.3.0@zip")');
    });

    it('needs full coordinates', () => {
      expect(getUsageSnippets({ ...jar, group: null })).toEqual([]);
    });
  });

  describe('npm', () => {
    const pkg = context({ format: 'npm', repositoryName: 'npm-hosted', name: 'herald-cli', version: '0.4.2' });

    it('installs with npm, Yarn and pnpm from the repository', () => {
      const s = texts(pkg);
      expect(s.npm).toBe('npm install herald-cli@0.4.2 --registry=https://astrum.example.com/repository/npm-hosted/');
      expect(s.yarn).toBe('yarn add herald-cli@0.4.2 --registry https://astrum.example.com/repository/npm-hosted/');
      expect(s.pnpm).toBe('pnpm add herald-cli@0.4.2 --registry=https://astrum.example.com/repository/npm-hosted/');
      expect(s.npmrc).toBe('registry=https://astrum.example.com/repository/npm-hosted/');
      expect(s.publish).toContain('npm publish --registry=https://astrum.example.com/repository/npm-hosted/');
    });

    it('sets a scoped registry for a scoped package', () => {
      const s = texts({ ...pkg, group: 'herald', name: 'ui-kit', version: '2.2.0' });
      expect(s.npm).toBe('npm install @herald/ui-kit@2.2.0 --registry=https://astrum.example.com/repository/npm-hosted/');
      expect(s.npmrc).toBe('@herald:registry=https://astrum.example.com/repository/npm-hosted/');
    });

    it('does not publish to a group', () => {
      expect(ids({ ...pkg, repositoryType: 'group' })).toEqual(['npm', 'yarn', 'pnpm', 'npmrc']);
    });
  });

  describe('pypi', () => {
    const pkg = context({ format: 'pypi', repositoryName: 'pypi-hosted', name: 'herald-client', version: '1.4.0' });

    it('installs from the simple index of the repository', () => {
      const s = texts(pkg);
      expect(s.pip).toBe('pip install --index-url https://astrum.example.com/repository/pypi-hosted/simple/ herald-client==1.4.0');
      expect(s['pip-conf']).toBe('[global]\nindex-url = https://astrum.example.com/repository/pypi-hosted/simple/');
      expect(s.requirements).toBe('--index-url https://astrum.example.com/repository/pypi-hosted/simple/\nherald-client==1.4.0');
      expect(s.twine).toBe('twine upload --repository-url https://astrum.example.com/repository/pypi-hosted/ dist/*');
    });

    it('trusts a host served over plain HTTP', () => {
      const s = texts({ ...pkg, repositoryUrl: 'http://10.0.0.5:8081/repository/pypi-hosted' });
      expect(s.pip).toBe(
        'pip install --index-url http://10.0.0.5:8081/repository/pypi-hosted/simple/ --trusted-host 10.0.0.5 herald-client==1.4.0',
      );
      expect(s['pip-conf']).toContain('trusted-host = 10.0.0.5');
    });
  });

  describe('nuget', () => {
    const pkg = context({ format: 'nuget', repositoryName: 'nuget-hosted', name: 'Herald.Client', version: '1.3.0' });

    it('uses the v3 service index of the repository', () => {
      const s = texts(pkg);
      expect(s.dotnet).toBe(
        'dotnet add package Herald.Client --version 1.3.0 --source https://astrum.example.com/repository/nuget-hosted/index.json',
      );
      expect(s.source).toBe('dotnet nuget add source https://astrum.example.com/repository/nuget-hosted/index.json --name nuget-hosted');
      expect(s.nuget).toBe('nuget sources add -Name nuget-hosted -Source https://astrum.example.com/repository/nuget-hosted/index.json');
      expect(s.config).toContain('<add key="nuget-hosted" value="https://astrum.example.com/repository/nuget-hosted/index.json" />');
      expect(s.push).toBe(
        'dotnet nuget push Herald.Client.1.3.0.nupkg --source https://astrum.example.com/repository/nuget-hosted/ --api-key <api-key>',
      );
    });
  });

  describe('helm', () => {
    const chart = context({ format: 'helm', repositoryName: 'helm-hosted', name: 'herald', version: '0.3.0' });

    it('adds the repository, installs the chart and uploads a package', () => {
      const s = texts(chart);
      expect(s.repo).toBe('helm repo add helm-hosted https://astrum.example.com/repository/helm-hosted/\nhelm repo update');
      expect(s.install).toBe('helm install herald helm-hosted/herald --version 0.3.0');
      expect(s.dependency).toBe(
        'dependencies:\n  - name: herald\n    version: 0.3.0\n    repository: https://astrum.example.com/repository/helm-hosted/',
      );
      expect(s.upload).toBe(
        'curl -u <user> --upload-file herald-0.3.0.tgz https://astrum.example.com/repository/helm-hosted/herald-0.3.0.tgz',
      );
    });
  });

  describe('gitlfs', () => {
    const oid = '4d7a214614ab2935c943f9e0ff69d22eadbb8f32b1258daaa5e2ca24d17e2393';
    const object = context({ format: 'gitlfs', repositoryName: 'lfs', name: oid, path: `objects/4d/7a/${oid}` });

    it('points git at the LFS API of the repository and downloads the object by its id', () => {
      const s = texts(object);
      expect(s.lfsconfig).toBe(
        'git config -f .lfsconfig lfs.url https://astrum.example.com/repository/lfs/info/lfs\ngit add .lfsconfig',
      );
      expect(s['git-config']).toBe('git config lfs.url https://astrum.example.com/repository/lfs/info/lfs');
      expect(s.curl).toBe(
        `curl -u <user> -fL -o ${oid} https://astrum.example.com/repository/lfs/info/lfs/objects/${oid}`,
      );
    });

    it('takes the id from the asset path and leaves out the download without one', () => {
      expect(texts(context({ format: 'gitlfs', repositoryName: 'lfs', path: `objects/4d/7a/${oid}` })).curl)
        .toContain(`/info/lfs/objects/${oid}`);
      expect(ids(context({ format: 'gitlfs', repositoryName: 'lfs', name: 'model.bin' }))).toEqual(['lfsconfig', 'git-config']);
    });
  });

  describe('raw', () => {
    const file = context({ format: 'raw', repositoryName: 'raw-hosted', path: 'releases/0.2.1/herald agent.tar.gz' });

    it('downloads and uploads the file at its path', () => {
      const s = texts(file);
      expect(s.curl).toBe('curl -fLO https://astrum.example.com/repository/raw-hosted/releases/0.2.1/herald%20agent.tar.gz');
      expect(s.wget).toBe('wget https://astrum.example.com/repository/raw-hosted/releases/0.2.1/herald%20agent.tar.gz');
      expect(s.upload).toBe(
        'curl -u <user> --upload-file ./herald agent.tar.gz https://astrum.example.com/repository/raw-hosted/releases/0.2.1/herald%20agent.tar.gz',
      );
    });

    it('does not encode an already encoded path twice', () => {
      expect(texts({ ...file, path: '/docs/a%20b.txt' }).wget).toBe('wget https://astrum.example.com/repository/raw-hosted/docs/a%20b.txt');
    });

    it('needs a path and only uploads to a hosted repository', () => {
      expect(getUsageSnippets({ ...file, path: null })).toEqual([]);
      expect(ids({ ...file, repositoryType: 'proxy' })).toEqual(['curl', 'wget']);
    });
  });

  describe('go', () => {
    const mod = context({ format: 'go', repositoryName: 'go-hosted', name: 'github.com/globalart/herald', version: 'v0.2.1' });

    it('uses the repository as GOPROXY', () => {
      const s = texts(mod);
      expect(s.get).toBe('GOPROXY=https://astrum.example.com/repository/go-hosted go get github.com/globalart/herald@v0.2.1');
      expect(s.env).toBe(
        'go env -w GOPROXY=https://astrum.example.com/repository/go-hosted,direct\ngo env -w GONOSUMDB=github.com/globalart/herald',
      );
      expect(s.mod).toBe('require github.com/globalart/herald v0.2.1');
      expect(s.upload).toBe(
        'curl -u <user> --upload-file v0.2.1.zip https://astrum.example.com/repository/go-hosted/github.com/globalart/herald/@v/v0.2.1.zip',
      );
    });

    it('keeps the checksum database for a proxy', () => {
      expect(texts({ ...mod, repositoryType: 'proxy' }).env).toBe(
        'go env -w GOPROXY=https://astrum.example.com/repository/go-hosted,direct',
      );
    });
  });

  describe('apt', () => {
    const deb = context({
      format: 'apt',
      repositoryName: 'apt-hosted',
      name: 'herald-agent',
      version: '0.2.1',
      path: 'pool/h/herald-agent/herald-agent_0.2.1_amd64.deb',
      distribution: 'bookworm',
    });

    it('writes the sources line with the distribution and installs the version', () => {
      const s = texts(deb);
      expect(s.sources).toBe('deb https://astrum.example.com/repository/apt-hosted/ bookworm main');
      expect(s.install).toBe('sudo apt-get update\nsudo apt-get install herald-agent=0.2.1');
      expect(s.upload).toBe(
        'curl -u <user> -H "Content-Type: multipart/form-data" --data-binary "@herald-agent_0.2.1_amd64.deb" https://astrum.example.com/repository/apt-hosted/',
      );
    });

    it('reads the distribution from a dists path, else leaves a placeholder', () => {
      expect(texts({ ...deb, distribution: null, path: 'dists/jammy/Release' }).sources).toContain(' jammy main');
      expect(texts({ ...deb, distribution: null, path: null }).sources).toContain(' <distribution> main');
    });
  });

  describe('cargo', () => {
    const crate = context({ format: 'cargo', repositoryName: 'cargo-hosted', name: 'herald-proto', version: '0.3.1' });

    it('registers the sparse index and adds the crate from it', () => {
      const s = texts(crate);
      expect(s.config).toBe('[registries.cargo-hosted]\nindex = "sparse+https://astrum.example.com/repository/cargo-hosted/"');
      expect(s.add).toBe('cargo add herald-proto@0.3.1 --registry cargo-hosted');
      expect(s.manifest).toBe('[dependencies]\nherald-proto = { version = "0.3.1", registry = "cargo-hosted" }');
      expect(s.publish).toContain('cargo publish --registry cargo-hosted');
    });

    it('replaces crates.io with a proxy', () => {
      const s = texts({ ...crate, repositoryName: 'crates-proxy', repositoryUrl: `${BASE}/crates-proxy/`, repositoryType: 'proxy' });
      expect(s.config).toBe(
        '[source.crates-io]\nreplace-with = "crates-proxy"\n\n[source.crates-proxy]\nregistry = "sparse+https://astrum.example.com/repository/crates-proxy/"',
      );
      expect(s.add).toBe('cargo add herald-proto@0.3.1');
    });
  });

  describe('conan', () => {
    it('adds a remote and installs from it', () => {
      const s = texts(context({ format: 'conan', repositoryName: 'conan-hosted', name: 'zlib', version: '1.3.1' }));
      expect(s.remote).toBe('conan remote add conan-hosted https://astrum.example.com/repository/conan-hosted/');
      expect(s.install).toBe('conan install --requires=zlib/1.3.1 -r conan-hosted');
      expect(s.conanfile).toBe('[requires]\nzlib/1.3.1');
      expect(s.upload).toBe('conan upload zlib/1.3.1 -r conan-hosted');
    });
  });

  describe('conda', () => {
    it('installs from the channel of the package', () => {
      const s = texts(context({
        format: 'conda',
        repositoryName: 'conda-proxy',
        repositoryType: 'proxy',
        name: 'numpy',
        version: '1.26.4',
        path: 'main/linux-64/numpy-1.26.4-py312.tar.bz2',
      }));
      expect(s.install).toBe('conda install --override-channels -c https://astrum.example.com/repository/conda-proxy/main numpy=1.26.4');
      expect(s.condarc).toBe('channels:\n  - https://astrum.example.com/repository/conda-proxy/main');
      expect(s.upload).toBeUndefined();
    });
  });

  describe('r', () => {
    it('installs from the repository', () => {
      const s = texts(context({ format: 'r', repositoryName: 'r-hosted', name: 'heraldr', version: '0.1.2' }));
      expect(s.install).toBe('install.packages("heraldr", repos = "https://astrum.example.com/repository/r-hosted/")');
      expect(s.remotes).toBe(
        'remotes::install_version("heraldr", version = "0.1.2", repos = "https://astrum.example.com/repository/r-hosted/")',
      );
      expect(s.rprofile).toBe('options(repos = c(r_hosted = "https://astrum.example.com/repository/r-hosted/"))');
      expect(s.upload).toBe(
        'curl -u <user> --upload-file heraldr_0.1.2.tar.gz https://astrum.example.com/repository/r-hosted/src/contrib/heraldr_0.1.2.tar.gz',
      );
    });
  });

  describe('cocoapods', () => {
    it('points the Podfile at the repository', () => {
      const s = texts(context({ format: 'cocoapods', repositoryName: 'cocoapods-proxy', repositoryType: 'proxy', name: 'Alamofire', version: '5.9.1' }));
      expect(s.podfile).toBe("source 'https://astrum.example.com/repository/cocoapods-proxy/'\n\npod 'Alamofire', '5.9.1'");
      expect(s.repo).toBe('pod repo add-cdn cocoapods-proxy https://astrum.example.com/repository/cocoapods-proxy/');
    });
  });

  describe('p2', () => {
    it('offers the repository as an update site', () => {
      const s = texts(context({ format: 'p2', repositoryName: 'eclipse-p2-proxy', repositoryType: 'proxy' }));
      expect(s.site).toBe('https://astrum.example.com/repository/eclipse-p2-proxy/');
      expect(s.tycho).toContain('<layout>p2</layout>');
    });
  });

  describe('composer', () => {
    it('registers the repository and requires the package', () => {
      const s = texts(context({ format: 'composer', repositoryName: 'composer-hosted', group: 'globalart', name: 'herald-php', version: '1.0.0' }));
      expect(s.config).toBe('composer config repositories.composer-hosted composer https://astrum.example.com/repository/composer-hosted/');
      expect(s.require).toBe('composer require globalart/herald-php:1.0.0');
      expect(s.upload).toBe(
        'curl -u <user> --upload-file package.zip https://astrum.example.com/repository/composer-hosted/packages/upload/globalart/herald-php/1.0.0',
      );
    });
  });

  describe('alpine', () => {
    it('adds the branch directory of the package to the apk repositories', () => {
      const s = texts(context({
        format: 'alpine',
        repositoryName: 'alpine-proxy',
        repositoryType: 'proxy',
        name: 'curl',
        version: '8.9.1-r0',
        path: 'v3.20/main/x86_64/curl-8.9.1-r0.apk',
      }));
      expect(s.repositories).toBe('echo "https://astrum.example.com/repository/alpine-proxy/v3.20/main" >> /etc/apk/repositories\napk update');
      expect(s.add).toBe('apk add curl=8.9.1-r0');
    });
  });

  describe('yum', () => {
    it('points the .repo file at the repodata of a hosted repository and uploads to the package path', () => {
      const s = texts(context({
        format: 'yum',
        repositoryName: 'yum-hosted',
        name: 'hello',
        version: '2.12.1-3.el9',
        path: 'el9/x86_64/Packages/hello-2.12.1-3.el9.x86_64.rpm',
        repodataDepth: 2,
        repodataSigned: true,
      }));
      expect(s.repo).toBe([
        '[yum-hosted]',
        'name=yum-hosted',
        'baseurl=https://astrum.example.com/repository/yum-hosted/el9/x86_64/',
        'enabled=1',
        'gpgcheck=0',
        'repo_gpgcheck=1',
        'gpgkey=https://astrum.example.com/repository/yum-hosted/el9/x86_64/repodata/repomd.xml.key',
      ].join('\n'));
      expect(s.install).toBe('sudo dnf install hello-2.12.1-3.el9');
      expect(s.upload).toBe('curl -u <user> --upload-file ./hello-2.12.1-3.el9.x86_64.rpm '
        + 'https://astrum.example.com/repository/yum-hosted/el9/x86_64/Packages/hello-2.12.1-3.el9.x86_64.rpm');
    });

    it('finds the repodata of a proxy above its Packages directory', () => {
      const s = texts(context({
        format: 'yum',
        repositoryName: 'rocky-proxy',
        repositoryType: 'proxy',
        name: 'bash',
        path: 'BaseOS/x86_64/os/Packages/b/bash-5.1.8-9.el9.x86_64.rpm',
      }));
      expect(s.repo).toContain('baseurl=https://astrum.example.com/repository/rocky-proxy/BaseOS/x86_64/os/\n');
      expect(s.repo).not.toContain('repo_gpgcheck');
      expect(s.install).toBe('sudo dnf install bash');
      expect(ids(context({ format: 'yum', repositoryName: 'rocky-proxy', repositoryType: 'proxy' })))
        .toEqual(['repo']);
    });
  });

  describe('swift', () => {
    const release = context({ format: 'swift', repositoryName: 'swift-hosted', group: 'mona', name: 'LinkedList', version: '1.1.1' });

    it('sets the registry, adds the dependency, signs in and publishes over HTTPS', () => {
      const s = texts(release);
      expect(ids(release)).toEqual(['registry', 'manifest', 'login', 'publish']);
      expect(s.registry).toBe('swift package-registry set https://astrum.example.com/repository/swift-hosted/');
      expect(s.manifest).toBe('.package(id: "mona.LinkedList", from: "1.1.1")');
      expect(s.login).toBe('swift package-registry login https://astrum.example.com/repository/swift-hosted/ --username <user>');
      expect(s.publish).toBe(
        'swift package-registry publish mona.LinkedList 1.1.1 --url https://astrum.example.com/repository/swift-hosted/',
      );
    });

    it('allows plain HTTP and signs in through .netrc', () => {
      const plain = { ...release, repositoryUrl: 'http://10.0.0.5:8081/repository/swift-hosted' };
      const s = texts(plain);
      expect(ids(plain)).toEqual(['registry', 'manifest', 'netrc', 'publish']);
      expect(s.registry).toBe('swift package-registry set --allow-insecure-http http://10.0.0.5:8081/repository/swift-hosted/');
      expect(s.netrc).toBe('machine 10.0.0.5\n  login <user>\n  password <password>');
      expect(s.publish).toContain('--url http://10.0.0.5:8081/repository/swift-hosted/ --allow-insecure-http');
    });

    it('does not publish to a proxy or a group', () => {
      expect(ids({ ...release, repositoryName: 'swift-group', repositoryType: 'group' })).toEqual(['registry', 'manifest', 'login']);
      expect(ids(context({ format: 'swift', repositoryName: 'swift-proxy', repositoryType: 'proxy' }))).toEqual(['registry', 'login']);
  describe('huggingface', () => {
    const commit = '8f4dff0307bd726d0932f44183f1a88e297bcfe4';

    it('points the client at the repository and downloads the file at its commit', () => {
      const file = context({
        format: 'huggingface',
        repositoryName: 'hf-proxy',
        repositoryType: 'proxy',
        group: 'models',
        name: 'acme/tiny',
        version: commit,
        path: `/models/acme/tiny/${commit}/onnx/model.onnx`,
      });
      expect(ids(file)).toEqual(['endpoint', 'cli', 'python', 'transformers']);
      const snippets = texts(file);
      expect(snippets.endpoint).toBe(`export HF_ENDPOINT=${BASE}/hf-proxy\nexport HF_HUB_DOWNLOAD_TIMEOUT=600`);
      expect(snippets.cli).toBe(`hf download acme/tiny onnx/model.onnx --revision ${commit}`);
      expect(snippets.python).toBe(
        'from huggingface_hub import hf_hub_download\n\npath = hf_hub_download(\n' +
          `    repo_id="acme/tiny",\n    filename="onnx/model.onnx",\n    revision="${commit}",\n)`,
      );
      expect(snippets.transformers).toContain(`AutoModel.from_pretrained("acme/tiny", revision="${commit}")`);
    });

    it('downloads a whole dataset without transformers', () => {
      const dataset = context({
        format: 'huggingface',
        repositoryName: 'hf-proxy',
        repositoryType: 'proxy',
        group: 'datasets',
        name: 'acme/data',
        version: commit,
      });
      expect(ids(dataset)).toEqual(['endpoint', 'cli', 'python']);
      expect(texts(dataset).cli).toBe(`hf download acme/data --repo-type dataset --revision ${commit}`);
      expect(texts(dataset).python).toContain('snapshot_download(\n    repo_id="acme/data",\n    repo_type="dataset",');
    });

    it('prefers the file the server recorded', () => {
      const asset = context({
        format: 'huggingface',
        repositoryName: 'hf-proxy',
        group: 'models',
        name: 'acme/tiny',
        version: commit,
        path: 'unexpected/layout',
        attributes: { huggingface: { file: 'config.json' } },
      });
      expect(texts(asset).cli).toBe(`hf download acme/tiny config.json --revision ${commit}`);
    });

    it('only sets the endpoint for the repository itself and for folders above a commit', () => {
      expect(ids(context({ format: 'huggingface', repositoryName: 'hf-proxy' }))).toEqual(['endpoint']);
      expect(ids(context({ format: 'huggingface', repositoryName: 'hf-proxy', name: 'tiny', path: 'models/acme/tiny' })))
        .toEqual(['endpoint']);
    });
  });

  describe('formats without a repository-aware generator', () => {
    it('fall back to the component dependency snippets', () => {
      const snippets = getUsageSnippets(context({ format: 'rubygems', repositoryName: 'gems', name: 'rails', version: '7.1.0' }));
      expect(snippets.map((snippet) => snippet.label)).toEqual(['Install', 'Gemfile']);
      expect(snippets[0].snippetText).toBe('gem install rails -v 7.1.0');
    });

    it('get nothing when there is no generator at all', () => {
      expect(getUsageSnippets(context({ format: 'bower', repositoryName: 'bower', name: 'x', version: '1' }))).toEqual([]);
    });
  });
});
