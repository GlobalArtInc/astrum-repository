
/*global Ext, NX*/

/**
 * @since 3.17
 */
Ext.define('NX.onboarding.view.OnboardingWizard', {
  extend: 'NX.wizard.Panel',
  alias: 'widget.nx-onboarding-wizard',
  requires: [
    'NX.Conditions',
    'NX.I18n'
  ],

  minHeight: 350
});
