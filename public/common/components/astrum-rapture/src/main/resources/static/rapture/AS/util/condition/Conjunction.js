
/*global Ext*/

/**
 * A {@link AS.util.condition.Condition} that is satisfied when all AND-ed {@link AS.util.condition.Condition}s
 * are satisfied.
 *
 * @since 3.0
 */
Ext.define('AS.util.condition.Conjunction', {
  extend: 'AS.util.condition.Condition',

  /**
   * Array of conditions to be AND-ed.
   *
   * @cfg {AS.util.condition.Condition[]}
   */
  conditions: undefined,

  /**
   * @override
   * @returns {AS.util.condition.Conjunction}
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
