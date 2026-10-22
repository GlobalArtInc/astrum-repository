import type { RepositoryDomain, RepositoryType } from '../../../api';
import { ansible } from './ansible';
import { bazel } from './bazel';
import { chef } from './chef';
import { hex } from './hex';
import { huggingface } from './huggingface';
import { nix } from './nix';
import { nx } from './nx';
import { openvsx } from './openvsx';
import { p2 } from './p2';
import { pacman } from './pacman';
import { pub } from './pub';
import { puppet } from './puppet';
import { swift } from './swift';
import { terraform } from './terraform';
import { turbo } from './turbo';
import { vagrant } from './vagrant';

export interface UsageComponent {
  group: string;
  name: string;
  version: string;
}

export interface UsageContext {
  format: string;
  repositoryName: string;
  repositoryType: RepositoryType;
  baseUrl: string;
  component?: UsageComponent;
  path?: string;
  attributes?: Record<string, unknown>;
  distribution?: string;
  alpineKey?: string;
  domain?: RepositoryDomain;
}

export interface SnippetBlock {
  id: string;
  label: string;
  labelKey?: string;
  code: string;
  hint?: string;
}

export interface Snippet {
  id: string;
  label: string;
  labelKey?: string;
  code: string;
  hint?: string;
  hintVars?: Record<string, string>;
  blocks?: SnippetBlock[];
}

export const USAGE_FORMATS = [
  'maven2',
  'npm',
  'nuget',
  'pypi',
  'docker',
  'raw',
  'helm',
  'huggingface',
  'swift',
  'terraform',
  'yum',
  'rubygems',
  'gitlfs',
  'cargo',
  'go',
  'apt',
  'composer',
  'conan',
  'cocoapods',
  'conda',
  'r',
  'p2',
  'alpine',
  'pub',
  'ansible',
  'chef',
  'puppet',
  'vagrant',
  'nix',
  'hex',
  'openvsx',
  'pacman',
  'turbo',
  'nx',
  'bazel',
] as const;

export function supportsUsage(format: string): boolean {
  return (USAGE_FORMATS as readonly string[]).includes(format);
}

export function repositoryUrl(baseUrl: string, repository: string): string {
  return `${baseUrl.replace(/\/+$/, '')}/repository/${encodeURIComponent(repository)}/`;
}

export function repositoryAddress(baseUrl: string, repository: string, domain?: RepositoryDomain): string {
  return domain?.url ?? repositoryUrl(baseUrl, repository);
}

export function dockerRegistry(
  baseUrl: string,
  repository: string,
  domain?: RepositoryDomain,
): { host: string; registry: string } {
  if (!domain) {
    const host = hostOf(baseUrl);
    return { host, registry: `${host}/${repository}` };
  }
  return {
    host: domain.host,
    registry: domain.routing === 'PATH' ? `${domain.host}/${domain.path ?? repository}` : domain.host,
  };
}

export function encodePath(path: string): string {
  return path
    .split('/')
    .filter((segment) => segment !== '')
    .map((segment) => encodeURIComponent(segment))
    .join('/');
}

function hostOf(baseUrl: string): string {
  try {
    return new URL(baseUrl).host;
  } catch {
    return baseUrl.replace(/^[a-z]+:\/\//i, '').replace(/\/.*$/, '');
  }
}

function hostnameOf(baseUrl: string): string {
  try {
    return new URL(baseUrl).hostname;
  } catch {
    return hostOf(baseUrl).replace(/:\d+$/, '');
  }
}

function plainHttp(baseUrl: string): boolean {
  return /^http:/i.test(baseUrl);
}

function xml(value: string): string {
  return value.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
}

function text(value: unknown): string | undefined {
  return typeof value === 'string' && value !== '' ? value : undefined;
}

function mavenExtras(attributes: Record<string, unknown> | undefined) {
  const classifier = text(attributes?.classifier);
  const extension = text(attributes?.extension);
  return {
    classifier,
    extension: extension === 'jar' || extension === 'pom' ? undefined : extension,
  };
}

function mavenSettings(context: UsageContext, url: string): Snippet {
  const id = xml(context.repositoryName);
  const hosted = context.repositoryType === 'hosted';
  const body = hosted
    ? [
        '  <profiles>',
        '    <profile>',
        `      <id>${id}</id>`,
        '      <repositories>',
        '        <repository>',
        `          <id>${id}</id>`,
        `          <url>${xml(url)}</url>`,
        '        </repository>',
        '      </repositories>',
        '    </profile>',
        '  </profiles>',
        '  <activeProfiles>',
        `    <activeProfile>${id}</activeProfile>`,
        '  </activeProfiles>',
      ]
    : [
        '  <mirrors>',
        '    <mirror>',
        `      <id>${id}</id>`,
        '      <mirrorOf>*</mirrorOf>',
        `      <url>${xml(url)}</url>`,
        '    </mirror>',
        '  </mirrors>',
      ];
  return {
    id: 'settings',
    label: 'settings.xml',
    code: ['<settings>', ...body, '</settings>'].join('\n'),
    hint: hosted ? 'browse.usage.hint.mavenSettingsHosted' : 'browse.usage.hint.mavenSettingsMirror',
  };
}

function gradle(context: UsageContext, url: string, kotlin: boolean): Snippet {
  const component = context.component;
  const target = kotlin ? `url = uri("${url}")` : `url '${url}'`;
  const lines = ['repositories {', '    maven {', `        ${target}`, '    }', '}'];
  if (component) {
    const { classifier, extension } = mavenExtras(context.attributes);
    const coordinates = [component.group, component.name, component.version, classifier]
      .filter((part) => part !== undefined)
      .join(':');
    const notation = extension ? `${coordinates}@${extension}` : coordinates;
    lines.push(
      '',
      'dependencies {',
      kotlin ? `    implementation("${notation}")` : `    implementation '${notation}'`,
      '}',
    );
  } else if (context.repositoryType === 'hosted') {
    lines.push(
      '',
      'publishing {',
      '    repositories {',
      '        maven {',
      kotlin ? '            name = "astrum"' : "            name 'astrum'",
      `            ${target}`,
      kotlin
        ? '            credentials(PasswordCredentials::class)'
        : '            credentials(PasswordCredentials)',
      '        }',
      '    }',
      '}',
    );
  }
  const publishing = !component && context.repositoryType === 'hosted';
  return {
    id: kotlin ? 'gradle-kotlin' : 'gradle-groovy',
    label: kotlin ? 'Gradle Kotlin' : 'Gradle Groovy',
    code: lines.join('\n'),
    hint: publishing
      ? kotlin
        ? 'browse.usage.hint.gradleKotlinPublish'
        : 'browse.usage.hint.gradleGroovyPublish'
      : kotlin
        ? 'browse.usage.hint.gradleKotlin'
        : 'browse.usage.hint.gradleGroovy',
  };
}

function maven(context: UsageContext): Snippet[] {
  const url = repositoryAddress(context.baseUrl, context.repositoryName, context.domain);
  const id = xml(context.repositoryName);
  const snippets: Snippet[] = [];
  const component = context.component;
  if (component?.group && component.name && component.version) {
    const { classifier, extension } = mavenExtras(context.attributes);
    snippets.push({
      id: 'maven',
      label: 'Maven',
      code: [
        '<dependency>',
        `  <groupId>${xml(component.group)}</groupId>`,
        `  <artifactId>${xml(component.name)}</artifactId>`,
        `  <version>${xml(component.version)}</version>`,
        ...(classifier ? [`  <classifier>${xml(classifier)}</classifier>`] : []),
        ...(extension ? [`  <type>${xml(extension)}</type>`] : []),
        '</dependency>',
      ].join('\n'),
      hint: 'browse.usage.hint.mavenDependency',
    });
  }
  snippets.push(gradle(context, url, true), gradle(context, url, false), mavenSettings(context, url));
  if (context.repositoryType === 'hosted') {
    const snapshot = component?.version.endsWith('-SNAPSHOT') ?? false;
    const element = snapshot ? 'snapshotRepository' : 'repository';
    snippets.push({
      id: 'deploy',
      label: 'mvn deploy',
      code: [
        '<distributionManagement>',
        `  <${element}>`,
        `    <id>${id}</id>`,
        `    <url>${xml(url)}</url>`,
        `  </${element}>`,
        '</distributionManagement>',
      ].join('\n'),
      hint: 'browse.usage.hint.mavenDeploy',
    });
  }
  return snippets;
}

function npm(context: UsageContext): Snippet[] {
  const url = repositoryAddress(context.baseUrl, context.repositoryName, context.domain);
  const component = context.component;
  const scope = component?.group ? `@${component.group.replace(/^@/, '')}` : '';
  const pkg = component ? (scope ? `${scope}/${component.name}` : component.name) : '<package>';
  const spec = component?.version ? `${pkg}@${component.version}` : pkg;
  const snippets: Snippet[] = [
    { id: 'npm', label: 'npm', code: `npm install ${spec} --registry=${url}` },
    { id: 'yarn', label: 'Yarn', code: `yarn add ${spec} --registry ${url}` },
    { id: 'pnpm', label: 'pnpm', code: `pnpm add ${spec} --registry=${url}` },
    {
      id: 'npmrc',
      label: '.npmrc',
      code: scope ? `${scope}:registry=${url}` : `registry=${url}`,
      hint: scope ? 'browse.usage.hint.npmrcScoped' : 'browse.usage.hint.npmrc',
    },
  ];
  if (context.repositoryType === 'hosted') {
    snippets.push({
      id: 'publish',
      label: 'npm publish',
      code: `npm login --registry=${url}\nnpm publish --registry=${url}`,
      hint: 'browse.usage.hint.npmPublish',
    });
  } else {
    snippets.push({
      id: 'login',
      label: 'npm login',
      code: `npm login --registry=${url}`,
      hint: 'browse.usage.hint.npmLogin',
    });
  }
  return snippets;
}

function pypi(context: UsageContext): Snippet[] {
  const url = repositoryAddress(context.baseUrl, context.repositoryName, context.domain);
  const index = `${url}simple/`;
  const trusted = plainHttp(context.baseUrl) ? hostnameOf(context.baseUrl) : '';
  const component = context.component;
  const requirement = component
    ? component.version
      ? `${component.name}==${component.version}`
      : component.name
    : '<package>';
  const snippets: Snippet[] = [
    {
      id: 'pip',
      label: 'pip',
      code: `pip install --index-url ${index}${trusted ? ` --trusted-host ${trusted}` : ''} ${requirement}`,
    },
    {
      id: 'pip-conf',
      label: 'pip.conf',
      code: `[global]\nindex-url = ${index}${trusted ? `\ntrusted-host = ${trusted}` : ''}`,
      hint: 'browse.usage.hint.pipConf',
    },
    {
      id: 'requirements',
      label: 'requirements.txt',
      code: `--index-url ${index}${trusted ? `\n--trusted-host ${trusted}` : ''}\n${requirement}`,
    },
  ];
  if (context.repositoryType === 'hosted') {
    snippets.push({
      id: 'twine',
      label: 'twine',
      code: `twine upload --repository-url ${url} dist/*`,
      hint: 'browse.usage.hint.twine',
    });
  }
  return snippets;
}

function nuget(context: UsageContext): Snippet[] {
  const url = repositoryAddress(context.baseUrl, context.repositoryName, context.domain);
  const index = `${url}index.json`;
  const name = context.repositoryName;
  const component = context.component;
  const snippets: Snippet[] = [];
  if (component?.name) {
    const version = component.version ? ` --version ${component.version}` : '';
    snippets.push({
      id: 'dotnet',
      label: '.NET CLI',
      code: `dotnet add package ${component.name}${version} --source ${index}`,
    });
  }
  snippets.push(
    {
      id: 'source',
      label: 'dotnet nuget',
      code: `dotnet nuget add source ${index} --name ${name}`,
      hint: 'browse.usage.hint.nugetSource',
    },
    {
      id: 'nuget',
      label: 'nuget.exe',
      code: `nuget sources add -Name ${name} -Source ${index}`,
      hint: 'browse.usage.hint.nugetSource',
    },
    {
      id: 'config',
      label: 'NuGet.Config',
      code: [
        '<configuration>',
        '  <packageSources>',
        `    <add key="${xml(name)}" value="${xml(index)}" />`,
        '  </packageSources>',
        '</configuration>',
      ].join('\n'),
      hint: 'browse.usage.hint.nugetConfig',
    },
  );
  if (context.repositoryType === 'hosted') {
    const file = component?.name
      ? component.version
        ? `${component.name}.${component.version}.nupkg`
        : `${component.name}.nupkg`
      : '<package>.nupkg';
    snippets.push({
      id: 'push',
      label: 'dotnet nuget push',
      code: `dotnet nuget push ${file} --source ${url} --api-key <api-key>`,
      hint: 'browse.usage.hint.nugetPush',
    });
  }
  return snippets;
}

const MANIFEST = /^\/?v2\/(.+)\/manifests\/([^/]+)$/;

export function dockerImage(context: UsageContext): { image: string; reference: string } | null {
  const match = context.path ? MANIFEST.exec(context.path) : null;
  if (match?.[1] && match[2]) {
    return { image: match[1], reference: match[2] };
  }
  if (context.component?.name && context.component.version) {
    return { image: context.component.name, reference: context.component.version };
  }
  return null;
}

function docker(context: UsageContext): Snippet[] {
  const { host, registry } = dockerRegistry(context.baseUrl, context.repositoryName, context.domain);
  const target = dockerImage(context);
  const image = target?.image ?? '<image>';
  const reference = target?.reference ?? '<tag>';
  const digest = /^[a-z0-9]+:/.test(reference);
  const ref = `${registry}/${image}${digest ? '@' : ':'}${reference}`;
  const service =
    image
      .split('/')
      .pop()
      ?.replace(/[^a-z0-9-]/gi, '') || 'app';
  const push = context.repositoryType === 'hosted' && !digest;
  const client = (id: string, label: string): Snippet => {
    const blocks: SnippetBlock[] = [
      { id: 'pull', label: 'Pull', labelKey: 'browse.usage.label.pull', code: `${id} pull ${ref}` },
      {
        id: 'login',
        label: 'Login',
        labelKey: 'browse.usage.label.login',
        code: `${id} login ${host}`,
        hint: 'browse.usage.hint.dockerLogin',
      },
    ];
    if (push) {
      blocks.push({
        id: 'push',
        label: 'Push',
        labelKey: 'browse.usage.label.push',
        code: `${id} tag ${image}:${reference} ${ref}\n${id} push ${ref}`,
        hint: 'browse.usage.hint.dockerPush',
      });
    }
    return { id, label, code: blocks.map((block) => block.code).join('\n'), blocks };
  };
  return [
    client('docker', 'Docker'),
    client('podman', 'Podman'),
    {
      id: 'dockerfile',
      label: 'Dockerfile',
      code: `FROM ${ref}`,
      hint: 'browse.usage.hint.dockerfile',
    },
    {
      id: 'compose',
      label: 'Compose',
      code: `services:\n  ${service}:\n    image: ${ref}`,
    },
    {
      id: 'kubernetes',
      label: 'Kubernetes',
      code: `spec:\n  containers:\n  - name: ${service}\n    image: ${ref}`,
    },
  ];
}

function raw(context: UsageContext): Snippet[] {
  const url = repositoryAddress(context.baseUrl, context.repositoryName, context.domain);
  const path = context.path ?? (context.component ? context.component.name : '');
  const known = path !== '' && !path.endsWith('/');
  const fileUrl = known ? `${url}${encodePath(path)}` : `${url}path/to/file.txt`;
  const file = known ? (path.split('/').pop() ?? 'file.txt') : 'file.txt';
  const snippets: Snippet[] = [
    {
      id: 'curl',
      label: 'curl',
      code: `curl -fLO ${fileUrl}`,
      hint: 'browse.usage.hint.rawSignIn',
    },
    { id: 'wget', label: 'wget', code: `wget ${fileUrl}`, hint: 'browse.usage.hint.rawSignIn' },
  ];
  if (context.repositoryType === 'hosted') {
    snippets.push({
      id: 'upload',
      label: 'curl',
      labelKey: 'browse.usage.label.upload',
      code: `curl -u <user> --upload-file ./${file} ${fileUrl}`,
      hint: 'browse.usage.hint.rawUpload',
    });
  }
  return snippets;
}

function helm(context: UsageContext): Snippet[] {
  const url = repositoryAddress(context.baseUrl, context.repositoryName, context.domain);
  const repository = context.repositoryName;
  const name = context.component?.name || '<chart>';
  const version = context.component?.version ?? '';
  const snippets: Snippet[] = [
    {
      id: 'repo',
      label: 'helm repo add',
      code: `helm repo add ${repository} ${url}\nhelm repo update`,
      hint: 'browse.usage.hint.helmRepo',
    },
    {
      id: 'install',
      label: 'helm install',
      code: `helm install ${name} ${repository}/${name}${version ? ` --version ${version}` : ''}`,
    },
    {
      id: 'dependency',
      label: 'Chart.yaml',
      code: [
        'dependencies:',
        `  - name: ${name}`,
        ...(version ? [`    version: ${version}`] : []),
        `    repository: ${url}`,
      ].join('\n'),
    },
  ];
  if (context.repositoryType === 'hosted') {
    const file = `${name}-${version || '<version>'}.tgz`;
    snippets.push({
      id: 'upload',
      label: 'curl',
      labelKey: 'browse.usage.label.upload',
      code: `curl -u <user> --upload-file ${file} ${url}${file}`,
      hint: 'browse.usage.hint.helmUpload',
    });
  }
  return snippets;
}

function yumBase(context: UsageContext): string[] {
  const directories = (context.path ?? '')
    .split('/')
    .filter((segment) => segment !== '')
    .slice(0, -1);
  const packages = directories.lastIndexOf('Packages');
  return packages >= 0 ? directories.slice(0, packages) : directories;
}

function yum(context: UsageContext): Snippet[] {
  const url = repositoryAddress(context.baseUrl, context.repositoryName, context.domain);
  const name = context.repositoryName;
  const base = yumBase(context);
  const baseUrl = base.length > 0 ? `${url}${encodePath(base.join('/'))}/` : url;
  const snippets: Snippet[] = [
    {
      id: 'repo',
      label: `${name}.repo`,
      code: [`[${name}]`, `name=${name}`, `baseurl=${baseUrl}`, 'enabled=1', 'gpgcheck=0'].join('\n'),
      hint: 'browse.usage.hint.yumRepo',
      hintVars: { name },
    },
  ];
  const component = context.component;
  if (component?.name) {
    snippets.push({
      id: 'install',
      label: 'dnf',
      code: `sudo dnf install ${component.name}${component.version ? `-${component.version}` : ''}`,
    });
  }
  if (context.repositoryType === 'hosted') {
    const path = context.path ?? '';
    const known = path.endsWith('.rpm');
    const file = known ? (path.split('/').pop() ?? '<package>.rpm') : '<package>.rpm';
    const target = known ? encodePath(path) : `${base.length > 0 ? `${base.join('/')}/` : ''}<package>.rpm`;
    snippets.push({
      id: 'upload',
      label: 'curl',
      labelKey: 'browse.usage.label.upload',
      code: `curl -u <user> --upload-file ./${file} ${url}${target}`,
      hint: 'browse.usage.hint.yumUpload',
    });
  }
  return snippets;
}

function rubygems(context: UsageContext): Snippet[] {
  const url = repositoryAddress(context.baseUrl, context.repositoryName, context.domain);
  const name = context.component?.name ?? '';
  const version = context.component?.version ?? '';
  const snippets: Snippet[] = [];
  if (name) {
    snippets.push({
      id: 'install',
      label: 'gem install',
      code: `gem install ${name}${version ? ` -v ${version}` : ''} --clear-sources --source ${url}`,
    });
  }
  snippets.push({
    id: 'gemfile',
    label: 'Gemfile',
    code: `source "${url}"${name ? `\n\ngem "${name}"${version ? `, "${version}"` : ''}` : ''}`,
    hint: 'browse.usage.hint.gemfile',
  });
  if (context.repositoryType === 'proxy') {
    snippets.push({
      id: 'mirror',
      label: 'bundle config',
      code: `bundle config set --global mirror.https://rubygems.org ${url}`,
      hint: 'browse.usage.hint.bundleMirror',
    });
  } else {
    snippets.push({
      id: 'sources',
      label: 'gem sources',
      code: `gem sources --add ${url}`,
      hint: 'browse.usage.hint.gemSources',
    });
  }
  if (context.repositoryType === 'hosted') {
    const stored = (context.path ?? '').split('/').pop() ?? '';
    const file = stored.endsWith('.gem')
      ? stored
      : name && version
        ? `${name}-${version}.gem`
        : '<name>-<version>.gem';
    snippets.push({
      id: 'push',
      label: 'gem push',
      code: `export RUBYGEMS_HOST=${url.replace(/\/+$/, '')}\ngem signin\ngem push ${file}`,
      hint: 'browse.usage.hint.gemPush',
    });
  }
  return snippets;
}

function cargo(context: UsageContext): Snippet[] {
  const url = repositoryAddress(context.baseUrl, context.repositoryName, context.domain);
  const repository = context.repositoryName;
  const name = context.component?.name ?? '';
  const version = context.component?.version ?? '';
  const proxy = context.repositoryType === 'proxy';
  const snippets: Snippet[] = [
    {
      id: 'config',
      label: 'config.toml',
      code: proxy
        ? `[source.crates-io]\nreplace-with = "${repository}"\n\n[source.${repository}]\nregistry = "sparse+${url}"`
        : `[registries.${repository}]\nindex = "sparse+${url}"`,
      hint: 'browse.usage.hint.cargoConfig',
    },
  ];
  if (name) {
    const registry = proxy ? '' : ` --registry ${repository}`;
    snippets.push(
      {
        id: 'add',
        label: 'cargo add',
        code: `cargo add ${name}${version ? `@${version}` : ''}${registry}`,
      },
      {
        id: 'manifest',
        label: 'Cargo.toml',
        code: proxy
          ? `[dependencies]\n${name} = "${version || '*'}"`
          : `[dependencies]\n${name} = { version = "${version || '*'}", registry = "${repository}" }`,
      },
    );
  }
  if (context.repositoryType === 'hosted') {
    snippets.push({
      id: 'publish',
      label: 'cargo publish',
      code: `cargo login --registry ${repository}\ncargo publish --registry ${repository}`,
      hint: 'browse.usage.hint.cargoPublish',
    });
  }
  return snippets;
}

function escapeGoPath(text: string): string {
  return text.replace(/[A-Z]/g, (letter) => `!${letter.toLowerCase()}`);
}

function go(context: UsageContext): Snippet[] {
  const url = repositoryAddress(context.baseUrl, context.repositoryName, context.domain).replace(/\/+$/, '');
  const module = context.component?.name || '<module>';
  const version = context.component?.version || '<version>';
  const snippets: Snippet[] = [
    {
      id: 'get',
      label: 'go get',
      code: `GOPROXY=${url} go get ${module}@${version}`,
    },
  ];
  if (context.repositoryType !== 'hosted') {
    snippets.push({
      id: 'env',
      label: 'go env',
      code: `go env -w GOPROXY=${url},direct`,
      hint: 'browse.usage.hint.goEnv',
    });
  } else {
    snippets.push({
      id: 'env',
      label: 'go env',
      code: `go env -w GOPROXY=${url},direct\ngo env -w GONOSUMDB=${module}`,
      hint: 'browse.usage.hint.goEnvPrivate',
    });
  }
  snippets.push({ id: 'mod', label: 'go.mod', code: `require ${module} ${version}` });
  if (context.repositoryType === 'hosted') {
    const target = `${escapeGoPath(module)}/@v/${escapeGoPath(version)}.zip`;
    snippets.push({
      id: 'upload',
      label: 'curl',
      labelKey: 'browse.usage.label.upload',
      code: `curl -u <user> --upload-file ${version}.zip ${url}/${target}`,
      hint: 'browse.usage.hint.goUpload',
    });
  }
  return snippets;
}

function apt(context: UsageContext): Snippet[] {
  const url = repositoryAddress(context.baseUrl, context.repositoryName, context.domain);
  const name = context.component?.name ?? '';
  const version = context.component?.version ?? '';
  const distribution = context.distribution || '<distribution>';
  const snippets: Snippet[] = [
    {
      id: 'sources',
      label: 'sources.list',
      code: `deb ${url} ${distribution} main`,
      hint: 'browse.usage.hint.aptSources',
      hintVars: { name: context.repositoryName },
    },
  ];
  if (name) {
    snippets.push({
      id: 'install',
      label: 'apt-get',
      code: `sudo apt-get update\nsudo apt-get install ${name}${version ? `=${version}` : ''}`,
    });
  }
  if (context.repositoryType === 'hosted') {
    const stored = (context.path ?? '').split('/').pop() ?? '';
    const file = stored.endsWith('.deb') ? stored : '<package>.deb';
    snippets.push({
      id: 'upload',
      label: 'curl',
      labelKey: 'browse.usage.label.upload',
      code: `curl -u <user> -H "Content-Type: multipart/form-data" --data-binary "@${file}" ${url}`,
      hint: 'browse.usage.hint.aptUpload',
    });
  }
  return snippets;
}

function composer(context: UsageContext): Snippet[] {
  const url = repositoryAddress(context.baseUrl, context.repositoryName, context.domain);
  const vendor = context.component?.group ?? '';
  const project = context.component?.name ?? '';
  const version = context.component?.version ?? '';
  const snippets: Snippet[] = [
    {
      id: 'config',
      label: 'composer config',
      code: `composer config repositories.${context.repositoryName} composer ${url}`,
      hint: 'browse.usage.hint.composerConfig',
    },
  ];
  if (project) {
    snippets.push({
      id: 'require',
      label: 'composer require',
      code: `composer require ${vendor ? `${vendor}/` : ''}${project}${version ? `:${version}` : ''}`,
    });
  }
  snippets.push({
    id: 'json',
    label: 'composer.json',
    code: `"repositories": [\n  { "type": "composer", "url": "${url}" }\n]`,
  });
  if (context.repositoryType === 'hosted') {
    snippets.push({
      id: 'upload',
      label: 'curl',
      labelKey: 'browse.usage.label.upload',
      code: `curl -u <user> --upload-file package.zip ${url}packages/upload/${vendor || '<vendor>'}/${project || '<project>'}/${version || '<version>'}`,
      hint: 'browse.usage.hint.composerUpload',
    });
  }
  return snippets;
}

function conan(context: UsageContext): Snippet[] {
  const url = repositoryAddress(context.baseUrl, context.repositoryName, context.domain);
  const repository = context.repositoryName;
  const name = context.component?.name ?? '';
  const version = context.component?.version ?? '';
  const channel = context.component?.group ?? '';
  const reference = name
    ? `${name}/${version || '<version>'}${channel ? `@${channel}` : ''}`
    : '<name>/<version>';
  const snippets: Snippet[] = [
    {
      id: 'remote',
      label: 'conan remote',
      code: `conan remote add ${repository} ${url}`,
      hint: 'browse.usage.hint.conanRemote',
    },
    { id: 'install', label: 'conan install', code: `conan install --requires=${reference} -r ${repository}` },
    { id: 'conanfile', label: 'conanfile.txt', code: `[requires]\n${reference}` },
  ];
  if (context.repositoryType === 'hosted') {
    snippets.push({
      id: 'upload',
      label: 'conan upload',
      code: `conan upload ${reference} -r ${repository}`,
      hint: 'browse.usage.hint.conanUpload',
    });
  }
  return snippets;
}

function cocoapods(context: UsageContext): Snippet[] {
  const url = repositoryAddress(context.baseUrl, context.repositoryName, context.domain);
  const name = context.component?.name ?? '';
  const version = context.component?.version ?? '';
  const pod = name ? `pod '${name}'${version ? `, '${version}'` : ''}` : "pod '<pod>'";
  return [
    {
      id: 'podfile',
      label: 'Podfile',
      code: `source '${url}'\n\ntarget '<App>' do\n  ${pod}\nend`,
      hint: 'browse.usage.hint.cocoapodsPodfile',
    },
    {
      id: 'repo',
      label: 'pod repo add-cdn',
      code: `pod repo add-cdn ${context.repositoryName} ${url}`,
      hint: 'browse.usage.hint.cocoapodsRepo',
    },
    {
      id: 'netrc',
      label: '.netrc',
      code: `machine ${hostnameOf(context.baseUrl)}\nlogin <user>\npassword <password>`,
      hint: 'browse.usage.hint.cocoapodsNetrc',
    },
  ];
}

const CONDA_PACKAGE = /\.(conda|tar\.bz2)$/;
const CONDA_INDEX = /^(current_)?repodata\.json(\.bz2|\.zst)?$|^channeldata\.json$/;
const CONDA_SUBDIR = /^(noarch|[a-z0-9_]+-[a-z0-9_]+)$/;

function condaPlace(context: UsageContext): { channel: string; subdir?: string; file?: string } {
  const segments = (context.path ?? '').split('/').filter((segment) => segment !== '');
  const last = segments[segments.length - 1] ?? '';
  if (CONDA_PACKAGE.test(last) && segments.length >= 2) {
    return { channel: segments.slice(0, -2).join('/'), subdir: segments[segments.length - 2], file: last };
  }
  if (last === 'channeldata.json') {
    return { channel: segments.slice(0, -1).join('/') };
  }
  if (CONDA_INDEX.test(last) && segments.length >= 2) {
    return { channel: segments.slice(0, -2).join('/'), subdir: segments[segments.length - 2] };
  }
  if (context.component) {
    return { channel: context.component.group };
  }
  if (CONDA_SUBDIR.test(last)) {
    return { channel: segments.slice(0, -1).join('/'), subdir: last };
  }
  return { channel: segments.join('/') };
}

function conda(context: UsageContext): Snippet[] {
  const url = repositoryAddress(context.baseUrl, context.repositoryName, context.domain);
  const place = condaPlace(context);
  const channel = `${url}${encodePath(place.channel)}`.replace(/\/+$/, '');
  const name = context.component?.name ?? '';
  const version = context.component?.version ?? '';
  const spec = name ? `${name}${version ? `=${version}` : ''}` : '<package>';
  const snippets: Snippet[] = [
    {
      id: 'install',
      label: 'conda install',
      code: `conda install -c ${channel} --override-channels ${spec}`,
    },
    {
      id: 'condarc',
      label: '.condarc',
      code: `channels:\n  - ${channel}`,
      hint: 'browse.usage.hint.condarc',
    },
    {
      id: 'mamba',
      label: 'micromamba',
      code: `micromamba install -c ${channel} --override-channels ${spec}`,
    },
  ];
  if (context.repositoryType === 'hosted') {
    const file = place.file ?? '<package>.conda';
    const subdir = place.subdir && place.subdir !== '' ? place.subdir : '<subdir>';
    snippets.push({
      id: 'upload',
      label: 'curl',
      labelKey: 'browse.usage.label.upload',
      code: `curl -u <user> --upload-file ${file} ${channel}/${subdir}/${file}`,
      hint: 'browse.usage.hint.condaUpload',
    });
  }
  return snippets;
}

const R_PACKAGE = /^[A-Za-z][A-Za-z0-9.]*_[0-9]+(?:[.-][0-9]+)+\.(?:tar\.gz|tar\.bz2|tar\.xz|tgz|zip)$/;

function rString(value: string): string {
  return `"${value.replace(/\\/g, '\\\\').replace(/"/g, '\\"')}"`;
}

function r(context: UsageContext): Snippet[] {
  const url = repositoryAddress(context.baseUrl, context.repositoryName, context.domain).replace(/\/+$/, '');
  const name = context.component?.name ?? '';
  const version = context.component?.version ?? '';
  const repos = rString(url);
  const hosted = context.repositoryType === 'hosted';
  const snippets: Snippet[] = [
    {
      id: 'install',
      label: 'install.packages',
      code: `install.packages(${rString(name || '<package>')}, repos = ${hosted ? `c(${repos}, getOption("repos"))` : repos})`,
    },
    {
      id: 'profile',
      label: '.Rprofile',
      code: hosted
        ? `options(repos = c(${rString(context.repositoryName)} = ${repos}, getOption("repos")))`
        : `options(repos = c(CRAN = ${repos}))`,
      hint: hosted ? 'browse.usage.hint.rProfile' : 'browse.usage.hint.rProfileMirror',
    },
  ];
  if (name) {
    snippets.push({
      id: 'description',
      label: 'DESCRIPTION',
      code: `Imports: ${name}${version ? ` (>= ${version})` : ''}`,
      hint: 'browse.usage.hint.rDescription',
    });
  }
  if (hosted) {
    const stored = context.path ?? '';
    const file = stored.split('/').pop() ?? '';
    const target = R_PACKAGE.test(file)
      ? stored.replace(/^\/+/, '')
      : `src/contrib/${name || '<package>'}_${version || '<version>'}.tar.gz`;
    snippets.push({
      id: 'upload',
      label: 'curl',
      labelKey: 'browse.usage.label.upload',
      code: `curl -u <user> --upload-file ${target.split('/').pop()} ${url}/${target}`,
      hint: 'browse.usage.hint.rUpload',
    });
  }
  return snippets;
}

function alpine(context: UsageContext): Snippet[] {
  const url = repositoryAddress(context.baseUrl, context.repositoryName, context.domain);
  const segments = (context.path ?? '').split('/').filter((segment) => segment !== '');
  const stored = segments[segments.length - 1] ?? '';
  const placed = stored.endsWith('.apk') && segments.length >= 2;
  const name = context.component?.name ?? '';
  const version = context.component?.version ?? '';
  const arch = placed ? (segments[segments.length - 2] ?? '') : context.component?.group || '<arch>';
  const branch = placed ? segments.slice(0, -2).join('/') : '<branch>';
  const key = context.alpineKey;
  const trust = key ? '' : ' --allow-untrusted';
  const lines = [
    ...(key ? [`wget -O /etc/apk/keys/${key} ${url}${key}`] : []),
    `echo "${branch === '' ? url.replace(/\/+$/, '') : `${url}${branch}`}" >> /etc/apk/repositories`,
    `apk update${trust}`,
  ];
  const snippets: Snippet[] = [
    {
      id: 'repositories',
      label: '/etc/apk/repositories',
      code: lines.join('\n'),
      hint: key ? 'browse.usage.hint.apkRepositories' : 'browse.usage.hint.apkUntrusted',
    },
  ];
  if (name) {
    snippets.push({
      id: 'add',
      label: 'apk add',
      code: `apk add${trust} ${name}${version ? `=${version}` : ''}`,
    });
  }
  if (context.repositoryType === 'hosted') {
    const file = placed ? stored : name && version ? `${name}-${version}.apk` : '<package>.apk';
    const target = placed ? encodePath(context.path ?? '') : `${branch}/${arch}/${file}`;
    snippets.push({
      id: 'upload',
      label: 'curl',
      labelKey: 'browse.usage.label.upload',
      code: `curl -u <user> --upload-file ${file} ${url}${target}`,
      hint: 'browse.usage.hint.apkUpload',
    });
  }
  return snippets;
}

const LFS_OID = /^[0-9a-f]{64}$/;

function gitlfs(context: UsageContext): Snippet[] {
  const api = `${repositoryAddress(context.baseUrl, context.repositoryName, context.domain)}info/lfs`;
  const file = (context.path ?? '').split('/').pop() ?? '';
  const oid = [context.component?.name, file].find((candidate) => candidate && LFS_OID.test(candidate));
  const snippets: Snippet[] = [
    {
      id: 'lfsconfig',
      label: '.lfsconfig',
      code: `git config -f .lfsconfig lfs.url ${api}\ngit add .lfsconfig`,
      hint: 'browse.usage.hint.lfsConfig',
    },
    {
      id: 'git-config',
      label: 'git config',
      code: `git config lfs.url ${api}`,
      hint: 'browse.usage.hint.lfsLocalConfig',
    },
  ];
  if (oid) {
    snippets.push({
      id: 'curl',
      label: 'curl',
      code: `curl -u <user> -fL -o ${oid} ${api}/objects/${oid}`,
      hint: 'browse.usage.hint.lfsObject',
    });
  }
  return snippets;
}

const GENERATORS: Record<string, (context: UsageContext) => Snippet[]> = {
  maven2: maven,
  npm,
  nuget,
  pypi,
  docker,
  raw,
  helm,
  swift: (context) =>
    swift(context, repositoryUrl(context.baseUrl, context.repositoryName), hostnameOf(context.baseUrl)),
  huggingface: (context) =>
    huggingface(context, repositoryAddress(context.baseUrl, context.repositoryName, context.domain)),
  terraform: (context) =>
    terraform(context, repositoryUrl(context.baseUrl, context.repositoryName), hostOf(context.baseUrl)),
  yum,
  rubygems,
  gitlfs,
  cargo,
  go,
  apt,
  composer,
  conan,
  cocoapods,
  conda,
  r,
  p2: (context) => p2(context, repositoryAddress(context.baseUrl, context.repositoryName, context.domain)),
  alpine,
  pub: (context) => pub(context, repositoryAddress(context.baseUrl, context.repositoryName, context.domain)),
  ansible: (context) =>
    ansible(context, repositoryAddress(context.baseUrl, context.repositoryName, context.domain)),
  chef: (context) =>
    chef(context, repositoryAddress(context.baseUrl, context.repositoryName, context.domain)),
  puppet: (context) =>
    puppet(context, repositoryAddress(context.baseUrl, context.repositoryName, context.domain)),
  vagrant: (context) =>
    vagrant(context, repositoryAddress(context.baseUrl, context.repositoryName, context.domain)),
  nix: (context) => nix(context, repositoryAddress(context.baseUrl, context.repositoryName, context.domain)),
  turbo: (context) => turbo(repositoryAddress(context.baseUrl, context.repositoryName, context.domain)),
  nx: (context) => nx(repositoryAddress(context.baseUrl, context.repositoryName, context.domain)),
  bazel: (context) =>
    bazel(context, repositoryAddress(context.baseUrl, context.repositoryName, context.domain)),
  hex: (context) => hex(context, repositoryAddress(context.baseUrl, context.repositoryName, context.domain)),
  openvsx: (context) =>
    openvsx(context, repositoryAddress(context.baseUrl, context.repositoryName, context.domain)),
  pacman: (context) =>
    pacman(context, repositoryAddress(context.baseUrl, context.repositoryName, context.domain)),
};

export function usageSnippets(context: UsageContext): Snippet[] {
  return GENERATORS[context.format]?.(context) ?? [];
}
