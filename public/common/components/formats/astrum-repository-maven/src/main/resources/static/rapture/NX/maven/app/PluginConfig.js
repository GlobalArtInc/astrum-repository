
/*global Ext, NX*/

/**
 * Maven plugin configuration
 *
 * @since 3.15
 */
Ext.define('NX.maven.app.PluginConfig', {
  '@aggregate_priority': 100,

  controllers: [
    {
      id: 'NX.maven.controller.MavenDependencySnippetController',
      active: true
    }
  ]
});
