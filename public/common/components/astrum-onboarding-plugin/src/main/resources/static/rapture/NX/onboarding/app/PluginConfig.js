
/*global Ext, NX*/

/**
 * @since 3.17
 */
Ext.define('NX.onboarding.app.PluginConfig', {
  '@aggregate_priority': 100,

  requires: [
    'NX.onboarding.app.PluginStrings'
  ],

  controllers: [
    {
      id: 'NX.onboarding.controller.Onboarding',
      active: function () {
        return NX.app.Application.bundleActive('astrum-onboarding-plugin');
      }
    }
  ]
});
