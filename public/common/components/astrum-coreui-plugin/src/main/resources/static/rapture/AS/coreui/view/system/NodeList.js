
/*global Ext, AS*/

/**
 * Node list.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.view.system.NodeList', {
  extend: 'AS.view.drilldown.Master',
  alias: 'widget.as-coreui-system-nodelist',
  requires: [
    'AS.I18n'
  ],

  config: {
    stateful: true,
    stateId: 'as-coreui-system-nodelist'
  },

  store: 'Node',

  /**
   * @override
   */
  initComponent: function() {
    var me = this;

    me.columns = [
      {
        xtype: 'as-iconcolumn',
        width: 36,
        iconVariant: 'x16',
        iconName: function () {
          return 'node-default';
        }
      },
      {
        header: AS.I18n.get('NodeList_Name_Header'),
        dataIndex: 'friendlyName',
        stateId: 'friendlyName',
        flex: 1,
        renderer: Ext.htmlEncode
      },
      {
        header: AS.I18n.get('NodeList_SocketAddress_Header'),
        dataIndex: 'socketAddress',
        stateId: 'socketAddress',
        flex: 1,
        renderer: Ext.htmlEncode
      },
      {
        header: AS.I18n.get('NodeList_Identity_Header'),
        dataIndex: 'nodeIdentity',
        stateId: 'nodeIdentity',
        flex: 1,
        renderer: Ext.htmlEncode
      }
    ];

    me.plugins = [
      {ptype: 'gridfilterbox', emptyText: AS.I18n.get('NodeList_Filter_EmptyText')}
    ];

    me.callParent();
  }
});
