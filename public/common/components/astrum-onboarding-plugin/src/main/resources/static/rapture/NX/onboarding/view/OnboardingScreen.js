
/*global Ext, NX*/

/**
 * @since 3.26
 */
Ext.define('NX.onboarding.view.OnboardingScreen', {
  extend: 'NX.wizard.Screen',
  alias: 'widget.nx-onboarding-screen',
  requires: [
    'NX.Conditions',
    'NX.I18n'
  ],

  minHeight: 350
});
