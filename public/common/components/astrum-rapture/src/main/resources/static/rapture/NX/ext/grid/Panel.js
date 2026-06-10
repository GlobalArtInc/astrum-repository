
/*global Ext, NX*/

/**
 * @override Ext.grid.Panel
 *
 * Add a column menu to clear the sort state of a column if allowClearSort is set.
 *
 * @since 3.14
 */
Ext.define('NX.ext.grid.Panel', {
  override: 'Ext.grid.Panel',

  allowClearSort: false,

  initComponent: function() {
    this.callParent();

    if (this.allowClearSort) {
      this.on({
        headermenucreate: function(grid, menu) {
          var store = grid.getStore();
          menu.insert(2, [
            {
              text: 'Clear Sort',
              iconCls: 'x-fa fa-eraser',

              handler: function() {
                store.sorters.clear();
                store.load();
              }
            }
          ]);
        }
      });
    }
  }
});
