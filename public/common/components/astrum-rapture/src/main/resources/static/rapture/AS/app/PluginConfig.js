
/*global Ext, AS*/

/**
 * Core plugin configuration.
 *
 * @since 3.0
 */
Ext.define('AS.app.PluginConfig', {
  '@aggregate_priority': 100,

  requires: [
    'AS.app.PluginStrings'
  ],

  controllers: [
    'UiSessionTimeout',
    'User'
  ]
});
