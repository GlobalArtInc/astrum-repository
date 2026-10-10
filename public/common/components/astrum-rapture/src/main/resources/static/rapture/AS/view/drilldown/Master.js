
/*global Ext, AS*/

/**
 * Master/Detail tabs.
 *
 * @since 3.0
 */
Ext.define('AS.view.drilldown.Master', {
  extend: 'Ext.grid.Panel',
  alias: 'widget.as-drilldown-master',
  requires: [
    'AS.I18n'
  ],

  maskElement: 'body',

  cls: 'as-drilldown-master',
  rowLines: false,

  /**
   * @private
   */
  initComponent: function() {
    var me = this,
        hasAffordance = me.columns.some(function(column) {
          return column.cls === 'as-drilldown-affordance';
        });

    if (!hasAffordance) {
      me.columns.push({
        width: 28,
        hideable: false,
        sortable: false,
        menuDisabled: true,
        resizable: false,
        draggable: false,
        cls: 'as-drilldown-affordance',

        defaultRenderer: function() {
          return Ext.DomHelper.markup({
            tag: 'span',
            cls: 'x-fa fa-angle-right'
          });
        }
      });
    }

    me.callParent();

    me.on('render', this.loadStore, this);
  },

  loadStore: function() {
    this.getStore().load();
  },

  pushColumn: function(newColumn) {
    var columns = this.getColumns(),
        hasAffordance = columns.some(function(column) {
          return column.cls === 'as-drilldown-affordance';
        });

    return this.getHeaderContainer().insert(hasAffordance ? columns.length - 1 : columns.length, newColumn);
  }
});
