

import type { SnippetGenerator } from './types';
import S from '../../../../constants/pages/search/ComponentDetailStrings';

/**
 * Encode special characters for safe interpolation into XML content.
 * Handles the five predefined XML entities: & < > " '
 */
function xmlEncode(str: string): string {
  return str
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&apos;');
}

/**
 * Maven (maven2) dependency snippet generator.
 *
 * Ported verbatim from Classic AS.maven.controller.MavenDependencySnippetController.
 * Classifier and extension are read from the asset's maven2 attributes; the Overview tab
 * invokes this at component level (asset undefined), so those are absent there.
 */
export const generate: SnippetGenerator = (component, asset) => {
  const { group, name, version } = component;
  const maven2 = asset?.attributes?.maven2;
  const classifier = maven2?.classifier;
  const extension = maven2?.extension;

  // Encode values for safe XML interpolation
  const encGroup = xmlEncode(group);
  const encName = xmlEncode(name);
  const encVersion = xmlEncode(version);
  const encClassifier = classifier ? xmlEncode(classifier) : '';
  const encExtension = extension ? xmlEncode(extension) : '';

  // Percent-encoded values for the shields.io/search badge URL (URL context, not XML).
  const urlGroup = encodeURIComponent(group);
  const urlName = encodeURIComponent(name);

  const gradleCoordinates =
    `${group}:${name}:${version}` +
    (classifier ? `:${classifier}` : '') +
    (extension ? `@${extension}` : '');

  return [
    {
      displayName: 'Apache Maven',
      description: S.SNIPPETS.INSERT_INTO_POM,
      snippetText:
        '<dependency>\n' +
        `  <groupId>${encGroup}</groupId>\n` +
        `  <artifactId>${encName}</artifactId>\n` +
        `  <version>${encVersion}</version>\n` +
        (classifier ? `  <classifier>${encClassifier}</classifier>\n` : '') +
        (extension && extension !== 'jar' ? `  <type>${encExtension}</type>\n` : '') +
        '</dependency>',
    },
    {
      displayName: 'Gradle Groovy DSL',
      snippetText: `implementation '${gradleCoordinates}'`,
    },
    {
      displayName: 'Gradle Kotlin DSL',
      snippetText: `implementation("${gradleCoordinates}")`,
    },
    {
      // A build.sbt is Scala source, not XML, so the coordinates stay raw (matching Classic)
      // rather than carrying the XML entities used by the pom.xml/Ivy snippets above.
      displayName: 'Scala SBT',
      snippetText:
        `libraryDependencies += "${group}" % "${name}" % "${version}"` +
        (classifier ? ` classifier "${classifier}"` : ''),
    },
    {
      displayName: 'Apache Ivy',
      snippetText:
        `<dependency org="${encGroup}" name="${encName}" rev="${encVersion}">` +
        (classifier || extension
          ? `\n  <artifact name="${encName}"` +
            (extension ? ` ext="${encExtension}"` : '') +
            (classifier ? ` m:classifier="${encClassifier}"` : '') +
            ' />\n'
          : '') +
        '</dependency>',
    },
    {
      // Groovy source, not XML — coordinates stay raw (see Scala SBT above).
      displayName: 'Groovy Grape',
      snippetText:
        '@Grapes(\n' +
        `  @Grab(group='${group}', module='${name}', version='${version}'` +
        (classifier ? `, classifier='${classifier}'` : '') +
        ')\n' +
        ')',
    },
    {
      // Clojure/EDN, not XML — coordinates stay raw.
      displayName: 'Leiningen',
      snippetText:
        `[${group}/${name} "${version}"` +
        (classifier ? ` :classifier "${classifier}"` : '') +
        (extension ? ` :extension "${extension}"` : '') +
        ']',
    },
    {
      // Ruby string literal, not XML — coordinates stay raw.
      displayName: 'Apache Buildr',
      snippetText:
        `'${group}:${name}` +
        (extension ? `:${extension}` : ':jar') +
        (classifier ? `:${classifier}` : '') +
        `:${version}'`,
    },
    {
      // A shields.io/search URL is a URL context, so coordinates need percent-encoding
      // (encodeURIComponent) rather than the XML entities used by the pom.xml/Ivy snippets.
      displayName: 'Maven Central Badge',
      snippetText:
        `[![Maven Central](https://img.shields.io/maven-central/v/${urlGroup}/${urlName}.svg?label=Maven%20Central)]` +
        `(https://search.maven.org/search?q=g:%22${urlGroup}%22%20AND%20a:%22${urlName}%22` +
        (classifier ? `%20AND%20l:%22${encodeURIComponent(classifier)}%22` : '') +
        ')',
    },
    {
      // A PURL is not XML content, so it uses the raw coordinates (matching Classic) rather
      // than the XML-entity-encoded values used by the pom.xml/Ivy/Gradle snippets above.
      displayName: 'PURL',
      snippetText:
        `pkg:maven/${group}/${name}@${version}` +
        (classifier || extension
          ? '?' +
            (classifier ? `classifier=${classifier}${extension ? '&' : ''}` : '') +
            (extension ? `extension=${extension}` : '')
          : ''),
    },
  ];
};
