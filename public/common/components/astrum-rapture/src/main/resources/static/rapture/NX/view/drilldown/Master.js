
/*global Ext, NX*/

/**
 * Master/Detail tabs.
 *
 * @since 3.0
 */
Ext.define('NX.view.drilldown.Master', {
  extend: 'Ext.grid.Panel',
  alias: 'widget.nx-drilldown-master',
  requires: [
    'NX.I18n'
  ],

  maskElement: 'body',

  cls: 'nx-drilldown-master',
  rowLines: false,

  /**
   * @private
   */
  initComponent: function() {
    var me = this,
        hasAffordance = me.columns.some(function(column) {
          return column.cls === 'nx-drilldown-affordance';
        });

    if (!hasAffordance) {
      me.columns.push({
        width: 28,
        hideable: false,
        sortable: false,
        menuDisabled: true,
        resizable: false,
        draggable: false,
        cls: 'nx-drilldown-affordance',

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
          return column.cls === 'nx-drilldown-affordance';
        });

    return this.getHeaderContainer().insert(hasAffordance ? columns.length - 1 : columns.length, newColumn);
  }
});
