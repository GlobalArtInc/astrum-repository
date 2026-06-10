
/*global Ext*/

/**
 * Filter plugin for grids where filtering is handled remotely. 
 *
 * @since 3.0
 */
Ext.define('NX.ext.grid.plugin.RemoteFilterBox', {
  extend: 'Ext.AbstractPlugin',
  alias: 'plugin.remotegridfilterbox',
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
            iconClass: 'fa-filter',
            emptyText: NX.I18n.get('Grid_Plugin_FilterBox_Empty'),
            searchDelay: 200,
            width: 200,
            listeners: {
              search: me.onSearch,
              searchcleared: me.onSearchCleared,
              scope: me
            }
          }
        ];
    me.grid = grid;

    me.callParent(arguments);

    if (tbar) {
      tbar.add(items);
    }
    else {
      grid.addDocked([
        {
          xtype: 'nx-actions',
          dock: 'top',
          items: items
        }
      ]);
    }

    me.grid.on('filteringautocleared', me.syncSearchBox, me);
  },

  /**
   * Syncs filtering value with search box.
   *
   * @private
   */
  syncSearchBox: function () {
    var me = this;

    me.grid.down('nx-searchbox').setValue(me.filterValue);
  },

  /**
   * Clears the present search.
   */
  clearSearch: function() {
    var me = this;

    me.grid.down('nx-searchbox').clearSearch();
  },

  /**
   * @private
   * Filter grid.
   *
   * @private
   * @param {NX.ext.SearchBox} searchBox component
   * @param {String} value to be searched
   */
  onSearch: function(searchBox, value) {
    var grid = searchBox.up('grid'),
        store = grid.getStore(),
        emptyText = grid.getView().emptyTextFilter;

    if (!grid.emptyText) {
      grid.emptyText = grid.getView().emptyText;
    }
    grid.getView().emptyText = NX.util.Filter.buildEmptyResult(value, emptyText);
    grid.getSelectionModel().deselectAll();
    store.addFilter([
      {
        id: 'filter',
        property: 'filter',
        value: value
      }
    ]);
  },

  /**
   * Clear filtering on grid.
   *
   * @private
   * @param {NX.ext.SearchBox} searchBox component
   */
  onSearchCleared: function(searchBox) {
    var grid = searchBox.up('grid'),
        store = grid.getStore();

    if (grid.emptyText) {
      grid.getView().emptyText = grid.emptyText;
    }
    grid.getSelectionModel().deselectAll();
    // we have to remove filter directly as store#removeFilter() does not work when store#remoteFilter = true
    if (store.getFilters().removeAtKey('filter')) {
      if (store.getFilters().length) {
        store.filter();
      }
      else {
        store.clearFilter();
      }
    }
  }

});
