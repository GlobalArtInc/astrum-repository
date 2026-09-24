
/*global Ext, NX*/

/**
 * @since 3.17
 */
Ext.define('NX.onboarding.step.OnboardingCompleteStep', {
  extend: 'NX.wizard.Step',
  requires: [
    'NX.onboarding.view.OnboardingCompleteScreen'
  ],

  config: {
    screen: 'NX.onboarding.view.OnboardingCompleteScreen',
    enabled: true
  },

  /**
   * @override
   */
  init: function () {
    var me = this;

    me.control({
      'button[action=finish]': {
        click: me.finish
      }
    });
  }
});
