
/*global Ext, AS*/

/**
 * Customized actioncolumn.
 *
 * @since 3.0
 */
Ext.define('AS.ext.grid.column.Action', {
  extend: 'Ext.grid.column.Action',
  alias: 'widget.as-actioncolumn',

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
