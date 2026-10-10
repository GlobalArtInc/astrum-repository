
/*global Ext, AS*/

/**
 * Assertion helper.
 *
 * @since 3.0
 */
Ext.define('AS.Assert', {
  singleton: true,
  requires: [
    'AS.Console'
  ],

  /**
   * Set to true to disable all assertions.
   *
   * @public
   * @property {Boolean}
   */
  disable: false,

  /**
   * @public
   * @param {Boolean} expression
   * @param {String...} message
   */
  assert: function() {
    //<if assert>
    if (this.disable) {
      return;
    }
    var args = Array.prototype.slice.call(arguments),
        expression = args.shift();
    if (!expression) {
      args.unshift('Assertion failure:');
      AS.Console.error.apply(AS.Console, args);
    }
    //</if>
  }
});
