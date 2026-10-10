
/*global Ext, AS*/

/**
 * A {@link AS.util.condition.Condition} that calls a function after event(s) fire.
 *
 * @since 3.14
 */
Ext.define('AS.util.condition.MultiListener', {
  extend: 'AS.util.condition.Condition',

  /**
   * @cfg {Array}
   *
   * An array of objects with an observable property and an array of event names
   */
  listenerConfigs: undefined,

  /**
   * The function to be called when an event occurs.
   *
   * @cfg {Function}
   */
  fn: undefined,

  /**
   * @override
   * @returns {AS.util.condition.MultiListener}
   */
  bind: function () {
    var me = this;

    if (!me.bounded) {
      if (!Ext.isFunction(me.fn)) {
        throw "fn must be a valid function";
      }
      if (!Ext.isArray(me.listenerConfigs)) {
        throw "listenerConfigs must be an array";
      }

      me.listenerConfigs.forEach(function(listenerConfig) {
        listenerConfig.events.forEach(function(event) {
          listenerConfig.observable.on(event, me.evaluate, me);
        });
      });

      me.callParent();
      me.evaluate();
    }

    return me;
  },

  /**
   * @private
   */
  evaluate: function () {
    var me = this;

    if (me.bounded) {
      me.setSatisfied(me.fn());
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
