
/*global Ext, AS*/

/**
 * Maven plugin configuration
 *
 * @since 3.15
 */
Ext.define('AS.maven.app.PluginConfig', {
  '@aggregate_priority': 100,

  controllers: [
    {
      id: 'AS.maven.controller.MavenDependencySnippetController',
      active: true
    }
  ]
});
