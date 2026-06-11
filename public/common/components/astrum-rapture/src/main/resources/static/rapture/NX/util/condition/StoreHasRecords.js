
/*global Ext, NX*/

/**
 * A {@link NX.util.condition.Condition} that is satisfied when specified store has records.
 *
 * @since 3.0
 */
Ext.define('NX.util.condition.StoreHasRecords', {
  extend: 'NX.util.condition.Condition',

  /**
   * Id of store to be monitored.
   *
   * @cfg {String}
   */
  store: undefined,

  /**
   * @override
   * @returns {NX.util.condition.StoreHasRecords}
   */
  bind: function () {
    var me = this,
        store;

    if (!me.bounded) {
      store = NX.getApplication().getStore(me.store);
      me.mon(store, {
        datachanged: me.evaluate,
        beforeload: Ext.pass(me.evaluate, [undefined]),
        scope: me
      });
      me.callParent();
      me.evaluate(store);
    }

    return me;
  },

  /**
   * @private
   */
  evaluate: function (store) {
    var me = this;

    if (me.bounded) {
      me.setSatisfied(Ext.isDefined(store) && (store.getCount() > 0));
    }
  },

  /**
   * @override
   * @returns {String}
   */
  toString: function () {
    return this.self.getName() + '{ store=' + this.store + ' }';
  }

});