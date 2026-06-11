
/*global Ext, NX*/

/**
 * A {@link NX.util.condition.Condition} that is satisfied applying a function on a state value.
 *
 * @since 3.0
 */
Ext.define('NX.util.condition.WatchState', {
  extend: 'NX.util.condition.Condition',
  requires: [
    'NX.State'
  ],

  /**
   * @cfg {String}
   *
   * State value key.
   */
  key: undefined,

  /**
   * An optional function to be called when a state value changes. If not specified, a boolean check
   * against value will be performed.
   *
   * @cfg {Function}
   */
  fn: undefined,

  /**
   * @override
   * @returns {NX.util.condition.WatchState}
   */
  bind: function () {
    var me = this,
        controller, listeners;

    if (!me.bounded) {
      if (!Ext.isDefined(me.fn)) {
        me.fn = function (value) {
          return value;
        };
      }
      controller = NX.getApplication().getController('State');
      listeners = { scope: me };
      listeners[me.key.toLowerCase() + 'changed'] = me.evaluate;
      me.mon(controller, listeners);
      me.callParent();
      me.evaluate(NX.State.getValue(me.key));
    }

    return me;
  },

  /**
   * @private
   */
  evaluate: function (value, oldValue) {
    var me = this;

    if (me.bounded) {
      me.setSatisfied(me.fn(value, oldValue));
    }
  },

  /**
   * @override
   * @returns {String}
   */
  toString: function () {
    return this.self.getName() + '{ key=' + this.key + ' }';
  }

});