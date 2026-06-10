
/*global Ext, NX*/

/**
 * Customized actioncolumn.
 *
 * @since 3.0
 */
Ext.define('NX.ext.grid.column.Action', {
  extend: 'Ext.grid.column.Action',
  alias: 'widget.nx-actioncolumn',

  /**
   * @override
   */
  initComponent: function () {
    var me = this;

    me.handler = function (grid, ri, ci, item, e, record, row) {
      me.fireEvent('actionclick', me, grid, ri, ci, item, record, row);
    };

    me.callParent();
  }
});
