
/*global Ext*/

/**
 * A {@link AS.util.condition.Condition} that is satisfied when a grid, specified by its selector, exists and has a
 * selection. Optionally, a function could be used to provide additional checking when grid has a selection.
 *
 * @since 3.0
 */
Ext.define('AS.util.condition.GridHasSelection', {
  extend: 'AS.util.condition.Condition',

  /**
   * A grid selector as specified by (@link Ext.ComponentQuery#query}.
   *
   * @cfg {String}
   */
  grid: undefined,

  /**
   * An optional function to be called when grid has a selection to perform additional checks on the
   * passed in model.
   *
   * @cfg {Function}
   */
  fn: undefined,

  /**
   * @override
   * @returns {AS.util.condition.GridHasSelection}
   */
  bind: function () {
    var me = this,
        gridQueryResult = Ext.ComponentQuery.query(me.grid),
        cmp = gridQueryResult && gridQueryResult.length ? gridQueryResult[0] : null;

    if (!me.bounded) {
      cmp.on({
        cellclick: me.cellclick,
        selectionchange: me.selectionchange,
        destroy: me.destroy,
        scope: me
      });
      me.callParent();
    }

    return me;
  },

  /**
   * @private
   */
  cellclick: function (cmp, td, cellIndex, model) {
    this.evaluate(cmp, model);
  },

  /**
   * @private
   */
  selectionchange: function (cmp, selected) {
    this.evaluate(cmp, selected ? selected[0] : null);
  },

  /**
   * @private
   */
  destroy: function (cmp) {
    var me = this;
    if (cmp.getSelectionModel().getSelected()) {
      me.evaluate(cmp, cmp.getSelection());
    }
    else {
      me.evaluate(cmp, null);
    }
  },

  /**
   * @private
   */
  evaluate: function (cmp, selection) {
    var me = this,
        satisfied = false;

    if (cmp && selection) {
      satisfied = true;
      if (Ext.isFunction(me.fn)) {
        satisfied = me.fn(selection) === true;
      }
    }
    me.setSatisfied(satisfied);
  },

  /**
   * @override
   * @returns {String}
   */
  toString: function () {
    return this.self.getName() + '{ grid=' + this.grid + ' }';
  }

});