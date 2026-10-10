
/*global Ext, AS*/

/**
 * Add user window.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.user.UserAdd', {
  extend: 'AS.view.AddPanel',
  alias: 'widget.as-coreui-user-add',
  requires: [
    'AS.Conditions',
    'AS.I18n'
  ],

  defaultFocus: 'userId',

  /**
   * @override
   */
  initComponent: function() {
    var me = this;

    me.settingsForm = {
      xtype: 'as-coreui-user-settings-form',
      api: {
        submit: 'AS.direct.coreui_User.create'
      },
      settingsFormSuccessMessage: function(data) {
        return AS.I18n.get('User_UserAdd_Create_Success') + data['userId'];
      },
      editableCondition: AS.Conditions.isPermitted('astrum:users:create'),
      editableMarker: AS.I18n.get('User_UserAdd_Create_Error'),

      buttons: [
        { text: AS.I18n.get('User_UserList_New_Button'), action: 'add', formBind: true, ui: 'as-primary' },
        { text: AS.I18n.get('Add_Cancel_Button'), action: 'back' }
      ]
    };

    me.callParent();

    me.down('#userId').setReadOnly(false);

    me.down('form').insert(5, [
      {
        xtype: 'as-password',
        name: 'password',
        itemId: 'password',
        fieldLabel: AS.I18n.get('User_UserAdd_Password_FieldLabel')
      },
      {
        xtype: 'as-password',
        allowBlank: true,
        fieldLabel: AS.I18n.get('User_UserAdd_PasswordConfirm_FieldLabel'),
        submitValue: false,
        validator: function() {
          var me = this;
          return (me.up('form').down('#password').getValue() === me.getValue()) ? true : AS.I18n.get('User_UserChangePassword_NoMatch_Error');
        }
      }
    ]);
  }

});
