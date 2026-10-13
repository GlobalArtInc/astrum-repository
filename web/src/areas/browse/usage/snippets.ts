import type { RepositoryType } from '../../../api';
import { huggingface } from './huggingface';
import { swift } from './swift';
import { terraform } from './terraform';

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
}

export interface Snippet {
  id: string;
  label: string;
  labelKey?: string;
  code: string;
  hint?: string;
  hintVars?: Record<string, string>;
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
] as const;

export function supportsUsage(format: string): boolean {
  return (USAGE_FORMATS as readonly string[]).includes(format);
}

export function repositoryUrl(baseUrl: string, repository: string): string {
  return `${baseUrl.replace(/\/+$/, '')}/repository/${encodeURIComponent(repository)}/`;
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
  const url = repositoryUrl(context.baseUrl, context.repositoryName);
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
  const url = repositoryUrl(context.baseUrl, context.repositoryName);
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
  const url = repositoryUrl(context.baseUrl, context.repositoryName);
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
  const url = repositoryUrl(context.baseUrl, context.repositoryName);
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
  const host = hostOf(context.baseUrl);
  const registry = `${host}/${context.repositoryName}`;
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
  const snippets: Snippet[] = [
    { id: 'pull', label: 'docker pull', code: `docker pull ${ref}` },
    {
      id: 'login',
      label: 'docker login',
      code: `docker login ${host}`,
      hint: 'browse.usage.hint.dockerLogin',
    },
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
  if (context.repositoryType === 'hosted' && !digest) {
    snippets.push({
      id: 'push',
      label: 'docker push',
      code: `docker tag ${image}:${reference} ${ref}\ndocker push ${ref}`,
      hint: 'browse.usage.hint.dockerPush',
    });
  }
  return snippets;
}

function raw(context: UsageContext): Snippet[] {
  const url = repositoryUrl(context.baseUrl, context.repositoryName);
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
  const url = repositoryUrl(context.baseUrl, context.repositoryName);
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
  huggingface: (context) => huggingface(context, repositoryUrl(context.baseUrl, context.repositoryName)),
  terraform: (context) =>
    terraform(context, repositoryUrl(context.baseUrl, context.repositoryName), hostOf(context.baseUrl)),
};

export function usageSnippets(context: UsageContext): Snippet[] {
  return GENERATORS[context.format]?.(context) ?? [];
}
