
/*global Ext*/

/**
 * Modal to retrieve the 'System Password' of the LDAP server
 *
 * @since 3.24
 */
Ext.define('NX.coreui.view.ldap.LdapSystemPasswordModal', {
  extend: 'NX.view.ModalDialog',
  alias: 'widget.nx-coreui-ldapserver-systempassword-modal',
  requires: [
    'NX.I18n'
  ],

  /**
   * @override
   */
  initComponent: function () {
    var me = this;

    Ext.apply(me, {
      ui: 'nx-inset',
      closable: true,
      width: NX.view.ModalDialog.SMALL_MODAL,

      title: NX.I18n.render(me, 'Title'),

      items: {
        xtype: 'form',
        defaultType: 'textfield',
        defaults: {
          anchor: '100%'
        },
        items: [
          {
            xtype: 'nx-password',
            name: 'authPassword',
            itemId: 'authPassword',
            fieldLabel: NX.I18n.render(me, 'Password_FieldLabel'),
            inputType: 'password',
            helpText: NX.I18n.render(me, 'Password_HelpText'),
            allowBlank: false
          }
        ],

        buttonAlign: 'left',
        buttons: [
          { text: NX.I18n.render(me, 'Button_OK'), action: 'ok', scope: me, formBind: true, bindToEnter: true, ui: 'nx-primary' },
          { text: NX.I18n.render(me, 'Button_Cancel'), handler: me.close, scope: me }
        ]
      }
    });

    me.callParent();
  },
});
