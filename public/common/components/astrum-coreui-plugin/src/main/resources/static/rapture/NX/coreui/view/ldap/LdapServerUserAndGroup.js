
/*global Ext, NX*/

/**
 * LDAP Server "User & Group" panel.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.ldap.LdapServerUserAndGroup', {
  extend: 'NX.view.SettingsPanel',
  alias: 'widget.nx-coreui-ldapserver-userandgroup',

  settingsForm: { xtype: 'nx-coreui-ldapserver-userandgroup-form' }

});
