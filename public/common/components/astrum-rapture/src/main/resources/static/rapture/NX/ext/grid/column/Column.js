
/*global Ext, NX*/

/**
 * @override Ext.grid.column.Column
 *
 * Allow grid columns to toggle from ASC to DESC and back to no sort if allowClearSort is set on the gridpanel.
 *
 * @since 3.14
 */
Ext.define('NX.ext.grid.column.Column', {
  override: 'Ext.grid.column.Column',

  /**
   * @override
   */
  toggleSortState: function() {
    var view = this.up('grid');
    if (view && view.allowClearSort && this.isSortable() && this.sortState === 'DESC') {
      view.getStore().sorters.clear();
      view.getStore().load();
    }
    else {
      this.callParent();
    }
  }
});
