import { generate as mavenSnippets } from '../maven2';
import { usageStrings } from './strings';
import { isHosted, withSlash } from './urls';
import type { SnippetAssetModel } from '../types';
import type { UsageGenerator, UsageSnippet } from './types';

function xml(value: string): string {
  return value
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}

function mavenAttributes(attributes: Readonly<Record<string, unknown>> | null | undefined): SnippetAssetModel | undefined {
  const maven2 = attributes?.maven2 as { classifier?: unknown; extension?: unknown } | undefined;
  if (!maven2) {
    return undefined;
  }
  const classifier = typeof maven2.classifier === 'string' && maven2.classifier ? maven2.classifier : undefined;
  const raw = typeof maven2.extension === 'string' && maven2.extension ? maven2.extension : undefined;
  // A jar is what a dependency resolves to anyway, and naming it (`@jar` in Gradle) would turn off the
  // transitive dependencies; the pom of a component is not a dependency of its own. Both describe the
  // component itself.
  const extension = raw === 'jar' || raw === 'pom' ? undefined : raw;
  if (!classifier && !extension) {
    return undefined;
  }
  return { attributes: { maven2: { classifier, extension } } };
}

/**
 * Maven usage: the dependency for pom.xml, Gradle (Kotlin and Groovy) with the repository declared,
 * settings.xml and, for a hosted repository, distributionManagement.
 *
 * A group or proxy repository is offered as a mirror of everything; a hosted repository as an extra
 * repository, since mirroring every dependency to it would hide the rest of the world.
 */
export const generate: UsageGenerator = (context) => {
  const { group, name, version, repositoryName } = context;
  if (!group || !name || !version) {
    return [];
  }
  const url = withSlash(context.repositoryUrl);
  const strings = usageStrings();
  const asset = mavenAttributes(context.attributes);
  const classic = mavenSnippets({ format: 'maven2', group, name, version }, asset);
  const dependency = classic.find((snippet) => snippet.displayName === 'Apache Maven')?.snippetText ?? '';
  const groovy = classic.find((snippet) => snippet.displayName === 'Gradle Groovy DSL')?.snippetText ?? '';
  const kotlin = classic.find((snippet) => snippet.displayName === 'Gradle Kotlin DSL')?.snippetText ?? '';
  const id = xml(repositoryName);
  const hosted = isHosted(context.repositoryType);

  const settings = hosted
    ? '<settings>\n' +
      '  <profiles>\n' +
      '    <profile>\n' +
      `      <id>${id}</id>\n` +
      '      <repositories>\n' +
      '        <repository>\n' +
      `          <id>${id}</id>\n` +
      `          <url>${xml(url)}</url>\n` +
      '        </repository>\n' +
      '      </repositories>\n' +
      '    </profile>\n' +
      '  </profiles>\n' +
      '  <activeProfiles>\n' +
      `    <activeProfile>${id}</activeProfile>\n` +
      '  </activeProfiles>\n' +
      '</settings>'
    : '<settings>\n' +
      '  <mirrors>\n' +
      '    <mirror>\n' +
      `      <id>${id}</id>\n` +
      '      <mirrorOf>*</mirrorOf>\n' +
      `      <url>${xml(url)}</url>\n` +
      '    </mirror>\n' +
      '  </mirrors>\n' +
      '</settings>';

  const snippets: UsageSnippet[] = [
    { id: 'maven', label: 'Maven', snippetText: dependency, description: strings.HINTS.MAVEN_DEPENDENCY },
    {
      id: 'gradle-kotlin',
      label: 'Gradle Kotlin',
      snippetText:
        'repositories {\n' +
        '    maven {\n' +
        `        url = uri("${url}")\n` +
        '    }\n' +
        '}\n\n' +
        'dependencies {\n' +
        `    ${kotlin}\n` +
        '}',
      description: strings.HINTS.GRADLE_KOTLIN,
    },
    {
      id: 'gradle-groovy',
      label: 'Gradle Groovy',
      snippetText:
        'repositories {\n' +
        '    maven {\n' +
        `        url '${url}'\n` +
        '    }\n' +
        '}\n\n' +
        'dependencies {\n' +
        `    ${groovy}\n` +
        '}',
      description: strings.HINTS.GRADLE_GROOVY,
    },
    {
      id: 'settings',
      label: 'settings.xml',
      snippetText: settings,
      description: hosted ? strings.HINTS.MAVEN_SETTINGS_HOSTED : strings.HINTS.MAVEN_SETTINGS_MIRROR,
    },
  ];
  if (hosted) {
    const element = version.endsWith('-SNAPSHOT') ? 'snapshotRepository' : 'repository';
    snippets.push({
      id: 'deploy',
      label: 'mvn deploy',
      snippetText:
        '<distributionManagement>\n' +
        `  <${element}>\n` +
        `    <id>${id}</id>\n` +
        `    <url>${xml(url)}</url>\n` +
        `  </${element}>\n` +
        '</distributionManagement>',
      description: strings.HINTS.MAVEN_DEPLOY,
    });
  }
  return snippets;
};
