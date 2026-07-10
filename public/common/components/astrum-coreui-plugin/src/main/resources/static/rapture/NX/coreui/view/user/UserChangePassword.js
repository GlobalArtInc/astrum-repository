
/*global Ext, NX*/

/**
 * Chnage password window.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.user.UserChangePassword', {
  extend: 'NX.view.ModalDialog',
  alias: 'widget.nx-coreui-user-changepassword',
  requires: [
    'NX.Conditions',
    'NX.I18n'
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

    me.ui = 'nx-inset';
    me.title = NX.I18n.get('User_UserChangePassword_Title');

    me.setWidth(NX.view.ModalDialog.SMALL_MODAL);

    me.items = {
      xtype: 'form',
      editableCondition: NX.Conditions.isPermitted('astrum:userschangepw:create'),
      editableMarker: NX.I18n.get('User_UserChangePassword_NoPermission_Error'),
      defaults: {
        anchor: '100%',
        // allow cancel to be clicked w/o validating field to be non-blank
        validateOnBlur: false
      },
      items: [
        {
          xtype: 'nx-password',
          name: 'password',
          itemId: 'password',
          fieldLabel: NX.I18n.get('User_UserChangePassword_Password_FieldLabel'),
          allowBlank: false
        },
        {
          xtype: 'nx-password',
          fieldLabel: NX.I18n.get('User_UserChangePassword_PasswordConfirm_FieldLabel'),
          allowBlank: false,
          submitValue: false,
          validator: function () {
            var me = this;
            return (me.up('form').down('#password').getValue() === me.getValue()) ? true : NX.I18n.get('User_UserChangePassword_NoMatch_Error');
          }
        }
      ],

      buttonAlign: 'left',
      buttons: [
        { text: NX.I18n.get('User_UserChangePassword_Submit_Button'), action: 'changepassword', formBind: true, bindToEnter: true, ui: 'nx-primary' },
        { text: NX.I18n.get('User_UserChangePassword_Cancel_Button'), handler: function () {
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
