
/**
 * A {@link AS.util.condition.Condition} that is never satisfied.
 *
 * @since 3.0
 */
Ext.define('AS.util.condition.NeverSatisfied', {
  extend: 'AS.util.condition.Condition',

  /**
   * @override
   * @returns {String}
   */
  toString: function () {
    return this.self.getName() + '{ never satisfied }';
  }

});
