
/**
 * A {@link NX.util.condition.Condition} that is never satisfied.
 *
 * @since 3.0
 */
Ext.define('NX.util.condition.NeverSatisfied', {
  extend: 'NX.util.condition.Condition',

  /**
   * @override
   * @returns {String}
   */
  toString: function () {
    return this.self.getName() + '{ never satisfied }';
  }

});
