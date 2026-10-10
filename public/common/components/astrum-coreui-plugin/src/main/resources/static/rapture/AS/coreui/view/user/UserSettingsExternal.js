
/*global Ext, AS*/

/**
 * External user "Settings" panel.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.user.UserSettingsExternal', {
  extend: 'AS.view.SettingsPanel',
  alias: 'widget.as-coreui-user-settings-external',

  settingsForm: { xtype: 'as-coreui-user-settings-external-form' }

});
