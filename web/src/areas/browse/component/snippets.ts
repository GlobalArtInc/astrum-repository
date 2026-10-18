import { installCommand, requirementsFile } from '../usage/ansible';
import { directorCommand, installableUnit } from '../usage/p2';
import { pubAdd, pubDependency } from '../usage/pub';
import { repositoryUrl } from '../usage/snippets';

export interface ComponentSnippet {
  id: string;
  label: string;
  code: string;
  hint?: string;
}

export interface ComponentSnippetContext {
  format: string;
  group: string;
  name: string;
  version: string;
  baseUrl: string;
  repository: string;
}

function host(baseUrl: string): string {
  try {
    return new URL(baseUrl).host;
  } catch {
    return baseUrl.replace(/^[a-z]+:\/\//i, '').replace(/\/.*$/, '');
  }
}

function maven({ group, name, version }: ComponentSnippetContext): ComponentSnippet[] {
  const query = `g:%22${group}%22%20AND%20a:%22${name}%22`;
  return [
    {
      id: 'maven',
      label: 'Apache Maven',
      hint: 'browse.component.snippet.pom',
      code: `<dependency>\n  <groupId>${group}</groupId>\n  <artifactId>${name}</artifactId>\n  <version>${version}</version>\n</dependency>`,
    },
    { id: 'gradle-groovy', label: 'Gradle Groovy DSL', code: `implementation '${group}:${name}:${version}'` },
    {
      id: 'gradle-kotlin',
      label: 'Gradle Kotlin DSL',
      code: `implementation("${group}:${name}:${version}")`,
    },
    { id: 'sbt', label: 'Scala SBT', code: `libraryDependencies += "${group}" % "${name}" % "${version}"` },
    {
      id: 'ivy',
      label: 'Apache Ivy',
      code: `<dependency org="${group}" name="${name}" rev="${version}"></dependency>`,
    },
    {
      id: 'grape',
      label: 'Groovy Grape',
      code: `@Grapes(\n  @Grab(group='${group}', module='${name}', version='${version}')\n)`,
    },
    { id: 'leiningen', label: 'Leiningen', code: `[${group}/${name} "${version}"]` },
    { id: 'buildr', label: 'Apache Buildr', code: `'${group}:${name}:jar:${version}'` },
    {
      id: 'badge',
      label: 'Maven Central Badge',
      code: `[![Maven Central](https://img.shields.io/maven-central/v/${group}/${name}.svg?label=Maven%20Central)](https://search.maven.org/search?q=${query})`,
    },
    { id: 'purl', label: 'PURL', code: `pkg:maven/${group}/${name}@${version}` },
  ];
}

function npm({ group, name, version }: ComponentSnippetContext): ComponentSnippet[] {
  const full = group ? `@${group.replace(/^@/, '')}/${name}` : name;
  return [
    {
      id: 'npm',
      label: 'npm',
      hint: 'browse.component.snippet.npmInstall',
      code: `npm install ${full}@${version}`,
    },
    {
      id: 'yarn',
      label: 'Yarn',
      hint: 'browse.component.snippet.npmInstall',
      code: `yarn add ${full}@${version}`,
    },
    {
      id: 'package-json',
      label: 'package.json',
      hint: 'browse.component.snippet.packageJson',
      code: `"${full}": "${version}"`,
    },
  ];
}

function pypi({ name, version }: ComponentSnippetContext): ComponentSnippet[] {
  return [
    { id: 'pip', label: 'pip', code: `pip install ${name}==${version}` },
    { id: 'easy-install', label: 'easy_install', code: `easy_install ${name}==${version}` },
    { id: 'pipenv', label: 'pipenv', code: `pipenv install ${name}==${version}` },
    { id: 'requirements', label: 'requirements.txt', code: `${name} == ${version}` },
  ];
}

function docker(context: ComponentSnippetContext): ComponentSnippet[] {
  const image = `${host(context.baseUrl)}/${context.repository}/${context.name}:${context.version}`;
  const service = context.name.split('/').pop() ?? context.name;
  return [
    { id: 'docker', label: 'Docker', code: `docker pull ${image}` },
    { id: 'podman', label: 'Podman', code: `podman pull ${image}` },
    { id: 'dockerfile', label: 'Dockerfile', code: `FROM ${image}` },
    { id: 'compose', label: 'Compose', code: `services:\n  ${service}:\n    image: ${image}` },
    {
      id: 'kubernetes',
      label: 'Kubernetes',
      code: `spec:\n  containers:\n  - name: ${service}\n    image: ${image}`,
    },
  ];
}

function nuget(context: ComponentSnippetContext): ComponentSnippet[] {
  const { name, version } = context;
  const source = `${context.baseUrl.replace(/\/+$/, '')}/repository/${context.repository}/`;
  return [
    { id: 'package-manager', label: 'Package Manager', code: `Install-Package ${name} -Version ${version}` },
    { id: 'dotnet', label: '.NET CLI', code: `dotnet add package ${name} --version ${version}` },
    { id: 'paket', label: 'Paket CLI', code: `paket add ${name} --version ${version}` },
    {
      id: 'choco',
      label: 'Chocolatey CLI',
      code: `choco install ${name} --version ${version} --source="${source}"`,
    },
  ];
}

function cargo({ name, version }: ComponentSnippetContext): ComponentSnippet[] {
  return [
    {
      id: 'cargo',
      label: 'cargo',
      hint: 'browse.component.snippet.projectCommand',
      code: `cargo add ${name}@${version}`,
    },
  ];
}

function composer({ group, name, version }: ComponentSnippetContext): ComponentSnippet[] {
  return [
    {
      id: 'composer',
      label: 'composer',
      hint: 'browse.component.snippet.projectCommand',
      code: `composer require ${group ? `${group}/` : ''}${name}:${version}`,
    },
  ];
}

function cocoapods({ name, version }: ComponentSnippetContext): ComponentSnippet[] {
  return [
    {
      id: 'podfile',
      label: 'Podfile',
      hint: 'browse.component.snippet.podfile',
      code: `pod '${name}', '${version}'`,
    },
  ];
}

function conda({ baseUrl, repository, group, name, version }: ComponentSnippetContext): ComponentSnippet[] {
  const channel = [
    baseUrl.replace(/\/+$/, ''),
    'repository',
    encodeURIComponent(repository),
    ...group.split('/'),
  ]
    .filter((part) => part !== '')
    .join('/');
  return [
    {
      id: 'conda',
      label: 'conda',
      code: `conda install -c ${channel} ${name}=${version}`,
    },
  ];
}

function r({ name, version, baseUrl, repository }: ComponentSnippetContext): ComponentSnippet[] {
  const url = `${baseUrl.replace(/\/+$/, '')}/repository/${repository}`;
  return [
    {
      id: 'install',
      label: 'install.packages',
      code: `install.packages("${name}", repos = "${url}")`,
    },
    {
      id: 'description',
      label: 'DESCRIPTION',
      hint: 'browse.component.snippet.rDescription',
      code: `Imports: ${name} (>= ${version})`,
    },
  ];
}

function p2(context: ComponentSnippetContext): ComponentSnippet[] {
  const url = repositoryUrl(context.baseUrl, context.repository);
  const unit = installableUnit(context);
  if (!unit) {
    return [];
  }
  return [
    {
      id: 'director',
      label: 'p2 director',
      hint: 'browse.component.snippet.p2Director',
      code: directorCommand(url, `${unit}/${context.version}`),
    },
    {
      id: 'target',
      label: '.target',
      hint: 'browse.component.snippet.p2Target',
      code: `<unit id="${unit}" version="${context.version}"/>`,
    },
  ];
}

function alpine({ name, version }: ComponentSnippetContext): ComponentSnippet[] {
  return [{ id: 'apk', label: 'apk', code: `apk add ${name}=${version}` }];
}

function pub(context: ComponentSnippetContext): ComponentSnippet[] {
  const url = repositoryUrl(context.baseUrl, context.repository).replace(/\/+$/, '');
  return [
    {
      id: 'add',
      label: 'dart pub add',
      hint: 'browse.component.snippet.projectCommand',
      code: pubAdd(context.name, url, context.version),
    },
    {
      id: 'pubspec',
      label: 'pubspec.yaml',
      hint: 'browse.component.snippet.pubspec',
      code: pubDependency(context.name, url, context.version),
    },
  ];
}

function ansible(context: ComponentSnippetContext): ComponentSnippet[] {
  const url = repositoryUrl(context.baseUrl, context.repository);
  const collection = context.group ? `${context.group}.${context.name}` : context.name;
  return [
    { id: 'install', label: 'ansible-galaxy', code: installCommand(url, collection, context.version) },
    {
      id: 'requirements',
      label: 'requirements.yml',
      hint: 'browse.component.snippet.ansibleRequirements',
      code: requirementsFile(url, collection, context.version),
    },
  ];
}

function rubygems({ name, version }: ComponentSnippetContext): ComponentSnippet[] {
  return [{ id: 'install', label: 'Install', code: `gem install ${name} -v ${version}` }];
}

export function componentSnippets(context: ComponentSnippetContext): ComponentSnippet[] {
  switch (context.format) {
    case 'maven2':
      return maven(context);
    case 'npm':
      return npm(context);
    case 'pypi':
      return pypi(context);
    case 'docker':
      return docker(context);
    case 'nuget':
      return nuget(context);
    case 'rubygems':
      return rubygems(context);
    case 'cargo':
      return cargo(context);
    case 'composer':
      return composer(context);
    case 'cocoapods':
      return cocoapods(context);
    case 'conda':
      return conda(context);
    case 'r':
      return r(context);
    case 'p2':
      return p2(context);
    case 'alpine':
      return alpine(context);
    case 'pub':
      return pub(context);
    case 'ansible':
      return ansible(context);
    default:
      return [];
  }
}

export function componentPath(format: string, group: string, name: string, version: string): string | null {
  if (format === 'maven2') {
    return `${group.replace(/\./g, '/')}/${name}/${version}`;
  }
  if (format === 'npm') {
    return `${group ? `${group}:` : ''}${name}/${version}`;
  }
  return null;
}

export function registryUrl(format: string, group: string, name: string): string | null {
  return format === 'maven2' ? `https://central.globalart.dev/artifact/${group}/${name}` : null;
}

const GUIDE_FORMATS: Record<string, string> = {
  maven2: 'maven',
  npm: 'npm',
  pypi: 'pypi',
  nuget: 'nuget',
  rubygems: 'gem',
  cargo: 'cargo',
};

export function componentGuideUrl(
  format: string,
  group: string,
  name: string,
  version: string,
): string | null {
  const id = GUIDE_FORMATS[format];
  if (!id || version === '') {
    return null;
  }
  const shown = group ? `${group}:${name}` : name;
  return `https://guide.globalart.dev/component/${id}/${encodeURIComponent(shown)}/${encodeURIComponent(version)}?referrer=search-component-detail`;
}
