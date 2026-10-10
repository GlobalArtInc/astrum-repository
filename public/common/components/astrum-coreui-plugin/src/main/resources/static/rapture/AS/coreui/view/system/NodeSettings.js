
/*global Ext, AS*/

/**
 * Node Settings Panel Tab.
 *
 * @since 3.6
 */
Ext.define('AS.coreui.view.system.NodeSettings', {
  extend: 'AS.view.SettingsPanel',
  alias: 'widget.as-coreui-system-node-settings',
  title: 'Node Settings Feature',

  settingsForm: { xtype: 'as-coreui-system-node-settings-form' }

});
