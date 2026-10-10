
/*global Ext, AS*/

/**
 * @since 3.26
 */
Ext.define('AS.onboarding.view.OnboardingScreen', {
  extend: 'AS.wizard.Screen',
  alias: 'widget.as-onboarding-screen',
  requires: [
    'AS.Conditions',
    'AS.I18n'
  ],

  minHeight: 350
});
