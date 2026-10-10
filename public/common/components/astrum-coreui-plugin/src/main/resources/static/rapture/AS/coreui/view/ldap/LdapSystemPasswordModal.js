
/*global Ext*/

/**
 * Modal to retrieve the 'System Password' of the LDAP server
 *
 * @since 3.24
 */
Ext.define('AS.coreui.view.ldap.LdapSystemPasswordModal', {
  extend: 'AS.view.ModalDialog',
  alias: 'widget.as-coreui-ldapserver-systempassword-modal',
  requires: [
    'AS.I18n'
  ],

  /**
   * @override
   */
  initComponent: function () {
    var me = this;

    Ext.apply(me, {
      ui: 'as-inset',
      closable: true,
      width: AS.view.ModalDialog.SMALL_MODAL,

      title: AS.I18n.render(me, 'Title'),

      items: {
        xtype: 'form',
        defaultType: 'textfield',
        defaults: {
          anchor: '100%'
        },
        items: [
          {
            xtype: 'as-password',
            name: 'authPassword',
            itemId: 'authPassword',
            fieldLabel: AS.I18n.render(me, 'Password_FieldLabel'),
            inputType: 'password',
            helpText: AS.I18n.render(me, 'Password_HelpText'),
            allowBlank: false
          }
        ],

        buttonAlign: 'left',
        buttons: [
          { text: AS.I18n.render(me, 'Button_OK'), action: 'ok', scope: me, formBind: true, bindToEnter: true, ui: 'as-primary' },
          { text: AS.I18n.render(me, 'Button_Cancel'), handler: me.close, scope: me }
        ]
      }
    });

    me.callParent();
  },
});
