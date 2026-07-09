
/*global Ext, NX*/

/**
 * Nodes feature panel.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.system.Nodes', {
  extend: 'NX.view.drilldown.Drilldown',
  alias: 'widget.nx-coreui-system-nodes',
  requires: [
    'NX.I18n',
    'NX.State'
  ],

  /**
   * @override
   */
  initComponent: function() {

    var me = this;
    me.iconName = 'node-default';
    if (NX.State.isClustered()) {
      me.masters = [{xtype: 'nx-coreui-system-nodelist'}];
      me.tabs = {
        xtype: 'nx-coreui-system-node-settings',
        title: NX.I18n.get('Nodes_NodeSettings_Title'),
        weight: 10
      };
    }
    else {
      // Nodes feature is only available in HA/clustered mode
      // The non-clustered view has been removed as the freeze button was non-functional
      // after ASTRUM-51156 removed the freeze API
      me.html = NX.I18n.get('Nodes_OSS_Message');
      me.skipDetail = true;
    }

    this.callParent();
  }
});
