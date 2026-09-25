
/*global Ext, NX*/

/**
 * @since 3.17
 */
Ext.define('NX.onboarding.view.ConfigureAnonymousAccessScreen', {
  extend: 'NX.onboarding.view.OnboardingScreen',
  alias: 'widget.nx-onboarding-configure-anonymous-access-screen',
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
                boxLabel: NX.I18n.render(me, 'Enable_Label'),
                name: 'configureAnonymous',
                itemId: 'anonymousEnabled'
              },
              {
                xtype: 'radio',
                boxLabel: NX.I18n.render(me, 'Disable_Label'),
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
