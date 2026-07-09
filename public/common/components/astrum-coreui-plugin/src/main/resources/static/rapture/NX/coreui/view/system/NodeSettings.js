
/*global Ext, NX*/

/**
 * Node Settings Panel Tab.
 *
 * @since 3.6
 */
Ext.define('NX.coreui.view.system.NodeSettings', {
  extend: 'NX.view.SettingsPanel',
  alias: 'widget.nx-coreui-system-node-settings',
  title: 'Node Settings Feature',

  settingsForm: { xtype: 'nx-coreui-system-node-settings-form' }

});
