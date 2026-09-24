
/*global Ext, NX*/

/**
 * @since 3.17
 */
Ext.define('NX.onboarding.step.OnboardingStartStep', {
  extend: 'NX.wizard.Step',
  requires: [
    'NX.onboarding.view.OnboardingStartScreen'
  ],

  config: {
    screen: 'NX.onboarding.view.OnboardingStartScreen',
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
