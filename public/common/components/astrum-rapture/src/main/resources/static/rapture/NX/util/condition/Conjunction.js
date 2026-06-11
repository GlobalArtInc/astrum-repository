
/*global Ext*/

/**
 * A {@link NX.util.condition.Condition} that is satisfied when all AND-ed {@link NX.util.condition.Condition}s
 * are satisfied.
 *
 * @since 3.0
 */
Ext.define('NX.util.condition.Conjunction', {
  extend: 'NX.util.condition.Condition',

  /**
   * Array of conditions to be AND-ed.
   *
   * @cfg {NX.util.condition.Condition[]}
   */
  conditions: undefined,

  /**
   * @override
   * @returns {NX.util.condition.Conjunction}
   */
  bind: function () {
    var me = this;

    if (!me.bounded) {
      me.callParent();
      me.evaluate();
      Ext.each(me.conditions, function (condition) {
        me.mon(condition, {
          satisfied: me.evaluate,
          unsatisfied: me.evaluate,
          scope: me
        });
      });
    }

    return me;
  },

  /**
   * @private
   */
  evaluate: function () {
    var me = this,
        satisfied = true;

    if (me.bounded) {
      Ext.each(me.conditions, function (condition) {
        satisfied = condition.satisfied;
        return satisfied;
      });
      me.setSatisfied(satisfied);
    }
  },

  /**
   * @overrdie
   * @returns {String}
   */
  toString: function () {
    return this.conditions.join(' AND ');
  }

});
