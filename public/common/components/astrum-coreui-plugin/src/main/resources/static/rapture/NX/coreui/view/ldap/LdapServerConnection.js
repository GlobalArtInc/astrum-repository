
/*global Ext, NX*/

/**
 * LDAP Server "Connection" panel.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.ldap.LdapServerConnection', {
  extend: 'NX.view.SettingsPanel',
  alias: 'widget.nx-coreui-ldapserver-connection',

  settingsForm: { xtype: 'nx-coreui-ldapserver-connection-form' }

});
