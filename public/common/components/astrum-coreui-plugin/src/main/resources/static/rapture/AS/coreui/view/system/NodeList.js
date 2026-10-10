
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

  columns: [
    {
      xtype: 'as-iconcolumn',
      width: 36,
      iconVariant: 'x16',
      iconName: function () {
        return 'node-default';
      }
    },
    {
      header: 'Node Name',
      dataIndex: 'friendlyName',
      stateId: 'friendlyName',
      flex: 1,
      renderer: Ext.htmlEncode
    },
    {
      header: 'Socket Address',
      dataIndex: 'socketAddress',
      stateId: 'socketAddress',
      flex: 1,
      renderer: Ext.htmlEncode
    },
    {
      header: 'Node Identity',
      dataIndex: 'nodeIdentity',
      stateId: 'nodeIdentity',
      flex: 1,
      renderer: Ext.htmlEncode
    }
  ],

  plugins: [
    {ptype: 'gridfilterbox', emptyText: 'No nodes matched "$filter"'}
  ]
});
