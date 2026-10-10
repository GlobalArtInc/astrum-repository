
/*global Ext, AS*/

/**
 * @since 3.17
 */
Ext.define('AS.onboarding.view.ChangeAdminPasswordScreen', {
  extend: 'AS.onboarding.view.OnboardingScreen',
  alias: 'widget.as-onboarding-change-admin-password-screen',
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

      buttons: ['back', '->', 'next'],

      fields: [{
        xtype: 'form',
        defaults: {
          anchor: '100%'
        },
        items: [
          {
            xtype: 'as-password',
            name: 'password',
            itemId: 'password',
            fieldLabel: AS.I18n.get('User_UserChangePassword_Password_FieldLabel'),
            allowBlank: false,
            listeners: {
              change: function(){
                var me = this;
                me.up('form').down('#verifyPassword').validate();
              }
            }
          },
          {
            xtype: 'as-password',
            itemId: 'verifyPassword',
            fieldLabel: AS.I18n.get('User_UserChangePassword_PasswordConfirm_FieldLabel'),
            allowBlank: false,
            submitValue: false,
            validator: function () {
              var me = this;
              return (me.up('form').down('#password').getValue() === me.getValue()) ? true : AS.I18n.get('User_UserChangePassword_NoMatch_Error');
            }
          }
        ]
      }]
    });

    me.callParent();
  }

});
