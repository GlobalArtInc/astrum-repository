
/*global Ext, AS*/

/**
 * Adds logging support helpers to objects.
 *
 * @since 3.0
 */
Ext.define('AS.LogAware', {
  requires: [
    'AS.Log'
  ],

  /**
   * Log a message at the given level.
   *
   * @param {String} level
   * @param {Array} args
   */
  log: function (level, args) {
    //<if debug>
    AS.Log.recordEvent(level, Ext.getClassName(this), args);
    //</if>
  },

  /**
   * Log a trace message.
   *
   * @public
   * @param {String/Object/Array} message
   */
  logTrace: function () {
    //<if debug>
    this.log('trace', Array.prototype.slice.call(arguments));
    //</if>
  },

  /**
   * Log a debug message.
   *
   * @public
   * @param {String/Object/Array} message
   */
  logDebug: function () {
    //<if debug>
    this.log('debug', Array.prototype.slice.call(arguments));
    //</if>
  },

  /**
   * Log an info message.
   *
   * @public
   * @param {String/Object/Array} message
   */
  logInfo: function () {
    //<if debug>
    this.log('info', Array.prototype.slice.call(arguments));
    //</if>
  },

  /**
   * Log a warn message.
   *
   * @public
   * @param {String/Object/Array} message
   */
  logWarn: function () {
    //<if debug>
    this.log('warn', Array.prototype.slice.call(arguments));
    //</if>
  },

  /**
   * Log an error message.
   *
   * @public
   * @param {String/Object/Array} message
   */
  logError: function () {
    //<if debug>
    this.log('error', Array.prototype.slice.call(arguments));
    //</if>
  }
});