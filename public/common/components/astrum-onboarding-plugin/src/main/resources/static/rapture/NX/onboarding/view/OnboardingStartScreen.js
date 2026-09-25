
/*global Ext, NX*/

/**
 * @since 3.17
 */
Ext.define('NX.onboarding.view.OnboardingStartScreen', {
  extend: 'NX.onboarding.view.OnboardingScreen',
  alias: 'widget.nx-onboarding-start-screen',
  requires: [
    'NX.Conditions',
    'NX.I18n'
  ],

  /**
   * @override
   */
  initComponent: function () {
    var me = this;

    Ext.apply(me, {
      title: NX.I18n.render(me, 'Title'),

      description: NX.I18n.render(me, 'Description'),

      buttons: ['->', 'next']
    });

    me.callParent();
  }

});
