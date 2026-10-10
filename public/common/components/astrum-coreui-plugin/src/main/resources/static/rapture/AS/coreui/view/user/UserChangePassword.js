
/*global Ext, AS*/

/**
 * Chnage password window.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.user.UserChangePassword', {
  extend: 'AS.view.ModalDialog',
  alias: 'widget.as-coreui-user-changepassword',
  requires: [
    'AS.Conditions',
    'AS.I18n'
  ],

  /**
   * @cfg userId to change password for
   */
  userId: undefined,

  /**
   * @override
   */
  initComponent: function () {
    var me = this;

    me.ui = 'as-inset';
    me.title = AS.I18n.get('User_UserChangePassword_Title');

    me.setWidth(AS.view.ModalDialog.SMALL_MODAL);

    me.items = {
      xtype: 'form',
      editableCondition: AS.Conditions.isPermitted('astrum:userschangepw:create'),
      editableMarker: AS.I18n.get('User_UserChangePassword_NoPermission_Error'),
      defaults: {
        anchor: '100%',
        // allow cancel to be clicked w/o validating field to be non-blank
        validateOnBlur: false
      },
      items: [
        {
          xtype: 'as-password',
          name: 'password',
          itemId: 'password',
          fieldLabel: AS.I18n.get('User_UserChangePassword_Password_FieldLabel'),
          allowBlank: false
        },
        {
          xtype: 'as-password',
          fieldLabel: AS.I18n.get('User_UserChangePassword_PasswordConfirm_FieldLabel'),
          allowBlank: false,
          submitValue: false,
          validator: function () {
            var me = this;
            return (me.up('form').down('#password').getValue() === me.getValue()) ? true : AS.I18n.get('User_UserChangePassword_NoMatch_Error');
          }
        }
      ],

      buttonAlign: 'left',
      buttons: [
        { text: AS.I18n.get('User_UserChangePassword_Submit_Button'), action: 'changepassword', formBind: true, bindToEnter: true, ui: 'as-primary' },
        { text: AS.I18n.get('User_UserChangePassword_Cancel_Button'), handler: function () {
          this.up('window').close();
        }}
      ]
    };

    me.maxHeight = Ext.getBody().getViewSize().height - 100;

    me.on({
      resize: function() {
        me.down('#password').focus();
      },
      single: true
    });

    me.callParent();
  }
});
