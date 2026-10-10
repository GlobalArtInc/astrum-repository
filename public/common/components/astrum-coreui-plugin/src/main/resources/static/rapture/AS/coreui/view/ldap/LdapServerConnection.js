
/*global Ext, AS*/

/**
 * LDAP Server "Connection" panel.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.ldap.LdapServerConnection', {
  extend: 'AS.view.SettingsPanel',
  alias: 'widget.as-coreui-ldapserver-connection',

  settingsForm: { xtype: 'as-coreui-ldapserver-connection-form' }

});
