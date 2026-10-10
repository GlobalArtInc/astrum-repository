
/*global Ext*/

/**
 * A {@link AS.util.condition.Condition} that is satisfied when any of OR-ed {@link AS.util.condition.Condition}s
 * is satisfied.
 *
 * @since 3.0
 */
Ext.define('AS.util.condition.Disjunction', {
  extend: 'AS.util.condition.Condition',

  /**
   * Array of conditions to be OR-ed.
   *
   * @cfg {AS.util.condition.Condition[]}
   */
  conditions: undefined,

  /**
   * @override
   * @returns {AS.util.condition.Disjunction}
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
        satisfied = false;

    if (me.bounded) {
      Ext.each(me.conditions, function (condition) {
        if(condition.satisfied){
          satisfied = true;
          return false;
        }
        return true;
      });
      me.setSatisfied(satisfied);
    }
  },

  /**
   * @override
   * @returns {String}
   */
  toString: function () {
    return this.conditions.join(' OR ');
  }

});
