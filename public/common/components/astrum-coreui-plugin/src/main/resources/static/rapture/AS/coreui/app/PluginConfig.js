
/*global Ext, AS*/

/**
 * CoreUi plugin configuration.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.app.PluginConfig', {
  '@aggregate_priority': 100,

  namespaces: [
    'AS.coreui'
  ],

  requires: [
    'AS.coreui.app.PluginStrings'
  ]
});
