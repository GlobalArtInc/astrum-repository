
/*global Ext, AS*/

/**
 * A {@link AS.util.condition.Condition} that is satisfied when specified store has records.
 *
 * @since 3.0
 */
Ext.define('AS.util.condition.StoreHasRecords', {
  extend: 'AS.util.condition.Condition',

  /**
   * Id of store to be monitored.
   *
   * @cfg {String}
   */
  store: undefined,

  /**
   * @override
   * @returns {AS.util.condition.StoreHasRecords}
   */
  bind: function () {
    var me = this,
        store;

    if (!me.bounded) {
      store = AS.getApplication().getStore(me.store);
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