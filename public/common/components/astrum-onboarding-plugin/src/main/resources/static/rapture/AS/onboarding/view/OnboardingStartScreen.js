
/*global Ext, AS*/

/**
 * @since 3.17
 */
Ext.define('AS.onboarding.view.OnboardingStartScreen', {
  extend: 'AS.onboarding.view.OnboardingScreen',
  alias: 'widget.as-onboarding-start-screen',
  requires: [
    'AS.Conditions',
    'AS.I18n'
  ],

  /**
   * @override
   */
  initComponent: function () {
    var me = this;

    Ext.apply(me, {
      title: AS.I18n.render(me, 'Title'),

      description: AS.I18n.render(me, 'Description'),

      buttons: ['->', 'next']
    });

    me.callParent();
  }

});
