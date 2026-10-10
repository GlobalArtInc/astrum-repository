
/*global Ext, AS*/

/**
 * @since 3.17
 */
Ext.define('AS.onboarding.step.OnboardingStartStep', {
  extend: 'AS.wizard.Step',
  requires: [
    'AS.onboarding.view.OnboardingStartScreen'
  ],

  config: {
    screen: 'AS.onboarding.view.OnboardingStartScreen',
    enabled: true
  },

  /**
   * @override
   */
  init: function () {
    var me = this;

    me.control({
      'button[action=next]': {
        click: me.moveNext
      }
    });
  }
});
