
/*global Ext, AS*/

/**
 * @since 3.15
 */
Ext.define('AS.maven.controller.MavenDependencySnippetController', {
  extend: 'AS.app.Controller',

  /**
   * @override
   */
  init: function() {
    AS.getApplication().getDependencySnippetController()
        .addDependencySnippetGenerator('maven2', this.snippetGenerator);
  },

  snippetGenerator: function(componentModel, assetModel) {
    var group = componentModel.get('group'),
        name = componentModel.get('name'),
        version = componentModel.get('version'),
        attributes = assetModel && assetModel.get('attributes'),
        maven2 = attributes && attributes.maven2,
        classifier = maven2 && maven2.classifier,
        extension = maven2 && maven2.extension,
        gradleCoordinates;

    gradleCoordinates = group + ':' + name + ':' + version +
        (classifier ? ':' + classifier : '') +
        (extension ? '@' + extension : '');

    return [
      {
        displayName: 'Apache Maven',
        description: AS.I18n.format('DependencySnippet_File_Description', 'pom.xml'),
        snippetText:
            '<dependency>\n' +
              '  <groupId>' + group + '</groupId>\n' +
              '  <artifactId>' + name + '</artifactId>\n' +
              '  <version>' + version + '</version>\n' +
              (classifier ? '  <classifier>' + classifier + '</classifier>\n' : '') +
              ((extension && 'jar' !== extension) ? '  <type>' + extension + '</type>\n' : '') +
            '</dependency>'
      }, {
        displayName: 'Gradle Groovy DSL',
        snippetText:
            'implementation \'' + gradleCoordinates + '\''
      }, {
        displayName: 'Gradle Kotlin DSL',
        snippetText:
            'implementation("' + gradleCoordinates + '")'
      }, {
        displayName: 'Scala SBT',
        snippetText:
            'libraryDependencies += "' + group + '" % "' + name + '" % "' + version + '"' +
            (classifier ? ' classifier "' + classifier + '"' : '')
      }, {
        displayName: 'Apache Ivy',
        snippetText:
            '<dependency org="' + group + '" name="' + name + '" rev="' + version +'">' +
            (
                (classifier || extension) ?
                    '\n  <artifact name="' + name + '"' +
                    (extension ? ' ext="' + extension + '"' : '') +
                    (classifier ? ' m:classifier="' + classifier + '"' : '') +
                    ' />\n'
                    :
                    ''
            ) +
            '</dependency>'
      }, {
        displayName: 'Groovy Grape',
        snippetText:
            '@Grapes(\n' +
            '  @Grab(group=\'' + group + '\', module=\'' + name + '\', version=\'' + version + '\'' +
            (classifier ? ', classifier=\'' + classifier + '\'' : '' ) + ')\n' +
            ')'
      }, {
        displayName: 'Leiningen',
        snippetText:
            '[' + group + '/' + name + ' "' + version + '"' +
            (classifier ? ' :classifier "' + classifier + '"' : '') +
            (extension ? ' :extension "' + extension + '"' : '') +
            ']'
      }, {
        displayName: 'Apache Buildr',
        snippetText:
            '\'' + group + ':' + name +
            (extension ? ':' + extension : ':jar') +
            (classifier ? ':' + classifier : '') +
            ':' + version + '\''
      }, {
        displayName: 'Maven Central Badge',
        snippetText:
            '[![Maven Central](https://img.shields.io/maven-central/v/' + group + '/' + name + '.svg?label=Maven%20Central)]' +
            '(https://search.maven.org/search?q=g:%22' + group + '%22%20AND%20a:%22' + name + '%22' +
              (classifier ? '%20AND%20l:%22' + classifier + '%22' : '') +
            ')'
      }, {
        displayName: 'PURL',
        snippetText:
            'pkg:maven/' + group + '/' + name + '@' + version +
            ( (classifier || extension) ?
                '?' + (classifier ? 'classifier=' + classifier + (extension ? '&' :'') : '') +
                (extension ? 'extension=' + extension : '')
                :
                ''
            )
      }
    ];
  }
});
