
/*global Ext, AS*/

/**
 * @since 3.17
 */
Ext.define('AS.onboarding.view.OnboardingWizard', {
  extend: 'AS.wizard.Panel',
  alias: 'widget.as-onboarding-wizard',
  requires: [
    'AS.Conditions',
    'AS.I18n'
  ],

  minHeight: 350
});
