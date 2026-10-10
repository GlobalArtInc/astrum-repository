
/*global Ext, AS*/

/**
 * @since 3.17
 */
Ext.define('AS.onboarding.app.PluginConfig', {
  '@aggregate_priority': 100,

  requires: [
    'AS.onboarding.app.PluginStrings'
  ],

  controllers: [
    {
      id: 'AS.onboarding.controller.Onboarding',
      active: function () {
        return AS.app.Application.bundleActive('astrum-onboarding-plugin');
      }
    }
  ]
});
