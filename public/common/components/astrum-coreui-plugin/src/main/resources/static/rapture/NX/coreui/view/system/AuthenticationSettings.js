
/*global Ext, NX*/

/**
 * Authentication settings fields.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.system.AuthenticationSettings', {
  extend: 'Ext.form.FieldContainer',
  alias: 'widget.nx-coreui-system-authenticationsettings',
  requires: [
    'NX.I18n'
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
        fieldLabel: NX.I18n.get('System_AuthenticationSettings_Username_FieldLabel'),
        allowBlank: false
      },
      {
        xtype: 'nx-password',
        name: namePrefix + 'uthPassword',
        fieldLabel: NX.I18n.get('System_AuthenticationSettings_Password_FieldLabel')
      },
      {
        name: namePrefix + 'uthNtlmHost',
        fieldLabel: NX.I18n.get('System_AuthenticationSettings_WindowsNtlmHostname_FieldLabel')
      },
      {
        name: namePrefix + 'uthNtlmDomain',
        fieldLabel: NX.I18n.get('System_AuthenticationSettings_WindowsNtlmDomain_FieldLabel')
      }
    ];

    me.callParent();
  }

});
