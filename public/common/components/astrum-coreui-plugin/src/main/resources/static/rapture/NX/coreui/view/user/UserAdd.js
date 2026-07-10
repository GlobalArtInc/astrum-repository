
/*global Ext, NX*/

/**
 * Add user window.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.user.UserAdd', {
  extend: 'NX.view.AddPanel',
  alias: 'widget.nx-coreui-user-add',
  requires: [
    'NX.Conditions',
    'NX.I18n'
  ],

  defaultFocus: 'userId',

  /**
   * @override
   */
  initComponent: function() {
    var me = this;

    me.settingsForm = {
      xtype: 'nx-coreui-user-settings-form',
      api: {
        submit: 'NX.direct.coreui_User.create'
      },
      settingsFormSuccessMessage: function(data) {
        return NX.I18n.get('User_UserAdd_Create_Success') + data['userId'];
      },
      editableCondition: NX.Conditions.isPermitted('astrum:users:create'),
      editableMarker: NX.I18n.get('User_UserAdd_Create_Error'),

      buttons: [
        { text: NX.I18n.get('User_UserList_New_Button'), action: 'add', formBind: true, ui: 'nx-primary' },
        { text: NX.I18n.get('Add_Cancel_Button'), action: 'back' }
      ]
    };

    me.callParent();

    me.down('#userId').setReadOnly(false);

    me.down('form').insert(5, [
      {
        xtype: 'nx-password',
        name: 'password',
        itemId: 'password',
        fieldLabel: NX.I18n.get('User_UserAdd_Password_FieldLabel')
      },
      {
        xtype: 'nx-password',
        allowBlank: true,
        fieldLabel: NX.I18n.get('User_UserAdd_PasswordConfirm_FieldLabel'),
        submitValue: false,
        validator: function() {
          var me = this;
          return (me.up('form').down('#password').getValue() === me.getValue()) ? true : NX.I18n.get('User_UserChangePassword_NoMatch_Error');
        }
      }
    ]);
  }

});
