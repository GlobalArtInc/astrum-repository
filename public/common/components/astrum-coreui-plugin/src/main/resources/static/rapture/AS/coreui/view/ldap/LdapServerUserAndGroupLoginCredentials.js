
/*global Ext, AS*/

/**
 * LDAP Server User & Group login credentials window.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.ldap.LdapServerUserAndGroupLoginCredentials', {
  extend: 'Ext.window.Window',
  alias: 'widget.as-coreui-ldapserver-userandgroup-login-credentials',
  requires: [
    'AS.Icons',
    'AS.I18n'
  ],

  /**
   * @protected
   */
  initComponent: function () {
    var me = this;

    if (!me.message) {
      me.message = AS.I18n.get('Ldap_LdapServerUserAndGroupLoginCredentials_Text');
    }

    Ext.apply(me, {
      ui: 'as-inset',
      title: AS.I18n.get('Ldap_LdapServerUserAndGroupLoginCredentials_Title'),
      layout: 'fit',
      autoShow: true,
      modal: true,
      constrain: true,
      width: 360,
      defaultFocus: 'username',
      items: {
        xtype: 'form',
        defaultType: 'textfield',
        defaults: {
          labelAlign: 'left',
          labelWidth: 140,
          anchor: '100%'
        },
        items: [
          {
            xtype: 'panel',
            layout: 'hbox',
            style: {
              // FIXME: sort out common style here for dialogs
              marginBottom: '10px'
            },
            items: [
              { xtype: 'component', html: AS.Icons.img('authenticate', 'x32') },
              { xtype: 'component', html: AS.I18n.get('Ldap_LdapServerUserAndGroupLoginCredentials_Input_Text') }
            ]
          },
          {
            name: 'username',
            itemId: 'username',
            fieldLabel: AS.I18n.get('Ldap_LdapServerUserAndGroupLoginCredentials_Username_FieldLabel'),
            allowBlank: false,
            validateOnBlur: false // allow cancel to be clicked w/o validating this to be non-blank
          },
          {
            name: 'password',
            itemId: 'password',
            fieldLabel: AS.I18n.get('Ldap_LdapServerUserAndGroupLoginCredentials_Password_FieldLabel'),
            inputType: 'password',
            allowBlank: false,
            validateOnBlur: false // allow cancel to be clicked w/o validating this to be non-blank
          }
        ],

        buttonAlign: 'left',
        buttons: [
          { text: AS.I18n.get('Ldap_LdapServerUserAndGroupLoginCredentials_Submit_Button'), action: 'verifylogin', formBind: true, bindToEnter: true, ui: 'as-primary' },
          { text: AS.I18n.get('Ldap_LdapServerUserAndGroupLoginCredentials_Cancel_Button'), handler: me.close, scope: me }
        ]
      }
    });

    me.callParent();
  }

});
