
/*global Ext, NX*/

/**
 * Core plugin configuration.
 *
 * @since 3.0
 */
Ext.define('NX.app.PluginConfig', {
  '@aggregate_priority': 100,

  requires: [
    'NX.app.PluginStrings'
  ],

  controllers: [
    'Dashboard',
    'Help',
    'Main',
    'Menu',
    'MenuGroup',
    'Refresh',
    'SettingsForm',
    'UiSessionTimeout',
    'User',

    {
      id: 'dev.Developer',
      active: function () {
        return NX.app.Application.debugMode;
      }
    },
  ]
});
