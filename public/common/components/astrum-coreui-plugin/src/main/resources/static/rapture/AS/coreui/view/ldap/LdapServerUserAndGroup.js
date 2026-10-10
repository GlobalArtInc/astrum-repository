
/*global Ext, AS*/

/**
 * LDAP Server "User & Group" panel.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.ldap.LdapServerUserAndGroup', {
  extend: 'AS.view.SettingsPanel',
  alias: 'widget.as-coreui-ldapserver-userandgroup',

  settingsForm: { xtype: 'as-coreui-ldapserver-userandgroup-form' }

});
