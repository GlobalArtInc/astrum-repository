
/*global Ext, NX*/

/**
 * Node list.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.view.system.NodeList', {
  extend: 'NX.view.drilldown.Master',
  alias: 'widget.nx-coreui-system-nodelist',
  requires: [
    'NX.I18n'
  ],

  config: {
    stateful: true,
    stateId: 'nx-coreui-system-nodelist'
  },

  store: 'Node',

  columns: [
    {
      xtype: 'nx-iconcolumn',
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
