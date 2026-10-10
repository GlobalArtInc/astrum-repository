
/*global Ext*/

/**
 * Console helper.
 *
 * @since 3.0
 */
Ext.define('AS.Console', {
  singleton: true,

  /**
   * @private
   */
  console: undefined,

  /**
   * Disable all application console output.
   *
   * @public
   * @property {Boolean}
   */
  disable: false,

  /**
   * Set to true to enable console 'trace'.
   *
   * @public
   * @property {Boolean}
   */
  traceEnabled: false,

  /**
   * Set to false to disable console 'debug'.
   *
   * @public
   * @property {Boolean}
   */
  debugEnabled: true,

  /**
   * Set up the console environment.
   */
  constructor: function () {
    this.console = AS.global.console || {};

    // apply default empty functions to console if missing
    Ext.applyIf(this.console, {
      log: Ext.emptyFn,
      info: Ext.emptyFn,
      warn: Ext.emptyFn,
      error: Ext.emptyFn
    });

    // use ?debug to enable
    this.debugEnabled = AS.global.location.href.search("[?&]debug") > -1;

    // use ?debug&trace to enable
    this.traceEnabled = AS.global.location.href.search("[?&]trace") > -1;
  },

  /**
   * Output a message to console at given level.
   *
   * @public
   * @param {String} level
   * @param {Array} args
   */
  log: function (level, args) {
    var c = this.console;
    switch (level) {
      case 'trace':
        if (this.traceEnabled) {
          c.log.apply(c, args);
        }
        break;

      case 'debug':
        if (this.debugEnabled) {
          c.log.apply(c, args);
        }
        break;

      case 'info':
        c.info.apply(c, args);
        break;

      case 'warn':
        c.warn.apply(c, args);
        break;

      case 'error':
        c.error.apply(c, args);
        break;
    }
  },

  /**
   * Outputs a trace message.
   *
   * @public
   * @param {String/Object/Array} message
   */
  trace: function() {
    this.log('trace', Array.prototype.slice.call(arguments));
  },

  /**
   * Outputs a debug message.
   *
   * @public
   * @param {String/Object/Array} message
   */
  debug: function () {
    this.log('debug', Array.prototype.slice.call(arguments));
  },

  /**
   * Outputs an info message.
   *
   * @public
   * @param {String/Object/Array} message
   */
  info: function () {
    this.log('info', Array.prototype.slice.call(arguments));
  },

  /**
   * Outputs a warn message.
   *
   * @public
   * @param {String/Object/Array} message
   */
  warn: function () {
    this.log('warn', Array.prototype.slice.call(arguments));
  },

  /**
   * Outputs an error message.
   *
   * @public
   * @param {String/Object/Array} message
   */
  error: function () {
    this.log('error', Array.prototype.slice.call(arguments));
  },

  /**
   * Helper to record event details to console.
   *
   * @internal
   * @param event
   */
  recordEvent: function(event) {
    this.log(event.level, [event.level, event.logger, event.message.join(' ')]);
  }
});