
/*global Ext, NX*/

/**
 * External user "Settings" panel.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.user.UserSettingsExternal', {
  extend: 'NX.view.SettingsPanel',
  alias: 'widget.nx-coreui-user-settings-external',

  settingsForm: { xtype: 'nx-coreui-user-settings-external-form' }

});
