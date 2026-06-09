
/*global Ext, NX*/

/**
 * Assertion helper.
 *
 * @since 3.0
 */
Ext.define('NX.Assert', {
  singleton: true,
  requires: [
    'NX.Console'
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
      NX.Console.error.apply(NX.Console, args);
    }
    //</if>
  }
});
