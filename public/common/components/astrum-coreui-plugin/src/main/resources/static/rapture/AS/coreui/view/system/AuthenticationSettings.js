
/*global Ext, AS*/

/**
 * Authentication settings fields.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.system.AuthenticationSettings', {
  extend: 'Ext.form.FieldContainer',
  alias: 'widget.as-coreui-system-authenticationsettings',
  requires: [
    'AS.I18n'
  ],

  namePrefix: undefined,

  defaults: {
    xtype: 'textfield'
  },

  /**
   * @override
   */
  initComponent: function () {
    var me = this,
        namePrefix = me.namePrefix ? me.namePrefix + 'A' : 'a';

    me.items = [
      {
        name: namePrefix + 'uthUsername',
        fieldLabel: AS.I18n.get('System_AuthenticationSettings_Username_FieldLabel'),
        allowBlank: false
      },
      {
        xtype: 'as-password',
        name: namePrefix + 'uthPassword',
        fieldLabel: AS.I18n.get('System_AuthenticationSettings_Password_FieldLabel')
      },
      {
        name: namePrefix + 'uthNtlmHost',
        fieldLabel: AS.I18n.get('System_AuthenticationSettings_WindowsNtlmHostname_FieldLabel')
      },
      {
        name: namePrefix + 'uthNtlmDomain',
        fieldLabel: AS.I18n.get('System_AuthenticationSettings_WindowsNtlmDomain_FieldLabel')
      }
    ];

    me.callParent();
  }

});
