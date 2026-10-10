
/*global Ext, AS*/

/**
 * @since 3.17
 */
Ext.define('AS.onboarding.view.ConfigureAnonymousAccessScreen', {
  extend: 'AS.onboarding.view.OnboardingScreen',
  alias: 'widget.as-onboarding-configure-anonymous-access-screen',
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

      buttons: ['back','->', 'next'],

      fields: [{
        xtype: 'form',
        defaults: {
          anchor: '100%'
        },
        items: [
          {
            xtype: 'radiogroup',
            allowBlank: false,
            columns: 1,
            items: [
              {
                xtype: 'radio',
                checked: false,
                boxLabel: AS.I18n.render(me, 'Enable_Label'),
                name: 'configureAnonymous',
                itemId: 'anonymousEnabled'
              },
              {
                xtype: 'radio',
                boxLabel: AS.I18n.render(me, 'Disable_Label'),
                checked: false,
                name: 'configureAnonymous',
                itemId: 'anonymousDisabled'
              }
            ]
          }]
      }]
    });

    me.callParent();
  }

});
