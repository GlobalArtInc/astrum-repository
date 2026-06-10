
/*global Ext*/

/**
 * Filter plugin for grids.
 *
 * @since 3.0
 */
Ext.define('NX.ext.grid.plugin.FilterBox', {
  extend: 'NX.ext.grid.plugin.Filtering',
  alias: 'plugin.gridfilterbox',
  requires: [
    'NX.I18n',
    'NX.util.Filter'
  ],

  /**
   * @cfg {String} emptyText Text to be used as grid empty text when no records are matching the filter. If text
   * contains "${filter}" it will be replaced with filter value.
   */

  /**
   * @override
   */
  init: function (grid) {
    var me = this,
        tbar = grid.getDockedItems('toolbar[dock="top"]')[0],
        items = [
          '->',
          {
            xtype: 'nx-searchbox',
            cls: ['nx-searchbox', 'nx-filterbox'],
            emptyText: NX.I18n.get('Grid_Plugin_FilterBox_Empty'),
            iconClass: 'fa-filter',
            searchDelay: 200,
            width: 200,
            listeners: {
              search: me.onSearch,
              searchcleared: me.onSearchCleared,
              scope: me
            }
          }
        ];

    me.callParent(arguments);

    if (tbar) {
      tbar.add(items);
    }
    else {
      grid.addDocked([
        {
          xtype: 'toolbar',
          dock: 'top',
          items: items
        }
      ]);
    }

    me.grid.on('filteringautocleared', me.syncSearchBox, me);
  },

  /**
   * Filter grid.
   *
   * @private
   */
  onSearch: function (searchbox, value) {
    var me = this;

    if (me.emptyText) {
      me.grid.getView().emptyText = NX.util.Filter.buildEmptyResult(value, me.emptyText);
    }
    me.filter(value);
  },

  /**
   * Clear the filter before destroying this plugin.
   *
   * @protected
   */
  destroy: function() {
    var me = this;

    me.clearFilter();

    me.callParent();
  },

  /**
   * Clear filtering on grid.
   *
   * @private
   */
  onSearchCleared: function () {
    var me = this;

    if (me.grid.emptyText) {
      me.grid.getView().emptyText = '<div class="x-grid-empty">' + me.grid.emptyText + '</div>';
    }
    me.clearFilter();
  },

  /**
   * Syncs filtering value with search box.
   *
   * @private
   */
  syncSearchBox: function () {
    var me = this;

    me.grid.down('nx-searchbox').setValue(me.filterValue);
  }

});
