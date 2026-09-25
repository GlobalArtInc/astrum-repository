
/*global Ext, NX*/

/**
 * @since 3.17
 */
Ext.define('NX.onboarding.view.OnboardingCompleteScreen', {
  extend: 'NX.onboarding.view.OnboardingScreen',
  alias: 'widget.nx-onboarding-complete-screen',
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

      buttons: ['->', {
        text: NX.I18n.render(me, 'Finish_Button'),
        action: 'finish',
        ui: 'nx-primary',
        cls: 'onboarding-finish-button'
      }]
    });

    me.callParent();
  }

});
