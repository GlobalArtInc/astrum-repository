
/*global Ext, AS*/

/**
 * @since 3.17
 */
Ext.define('AS.onboarding.step.OnboardingCompleteStep', {
  extend: 'AS.wizard.Step',
  requires: [
    'AS.onboarding.view.OnboardingCompleteScreen'
  ],

  config: {
    screen: 'AS.onboarding.view.OnboardingCompleteScreen',
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
