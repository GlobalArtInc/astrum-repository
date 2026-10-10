
/*global Ext, AS*/

/**
 * Nodes feature panel.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.system.Nodes', {
  extend: 'AS.view.drilldown.Drilldown',
  alias: 'widget.as-coreui-system-nodes',
  requires: [
    'AS.I18n',
    'AS.State'
  ],

  /**
   * @override
   */
  initComponent: function() {

    var me = this;
    me.iconName = 'node-default';
    if (AS.State.isClustered()) {
      me.masters = [{xtype: 'as-coreui-system-nodelist'}];
      me.tabs = {
        xtype: 'as-coreui-system-node-settings',
        title: AS.I18n.get('Nodes_NodeSettings_Title'),
        weight: 10
      };
    }
    else {
      // Nodes feature is only available in HA/clustered mode
      // The non-clustered view has been removed as the freeze button was non-functional
      // after ASTRUM-51156 removed the freeze API
      me.html = AS.I18n.get('Nodes_OSS_Message');
      me.skipDetail = true;
    }

    this.callParent();
  }
});
