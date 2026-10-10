
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
        return AS.app.Application.debugMode;
      }
    },
  ]
});
