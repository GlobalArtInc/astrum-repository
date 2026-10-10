
/*global Ext*/

/**
 * Global logging helper.
 *
 * @since 3.0
 */
Ext.define('AS.Log', {
  singleton: true,
  requires: [
    'AS.Console'
  ],

  /**
   * Reference to attached logging controller.
   *
   * @private
   * @property {AS.controller.Logging}
   */
  controller: undefined,

  /**
   * Queue of events logged before controller is attached.
   * This is deleted upon attachment after events are passed to the controller.
   *
   * @private
   */
  eventQueue: [],

  /**
   * Attach to the logging controller.
   *
   * @internal
   * @param {AS.controller.Logging} controller
   */
  attach: function (controller) {
    var me = this;
    me.controller = controller;

    // reply queued events and clear
    Ext.each(me.eventQueue, function (event) {
      me.controller.recordEvent(event);
    });
    delete me.eventQueue;
  },

  /**
   * Record a log event.
   *
   * @public
   * @param {String} level
   * @param {String} logger
   * @param {String/Array} message
   */
  recordEvent: function (level, logger, message) {
    var me = this,
        event = {
          timestamp: Date.now(),
          level: level,
          logger: logger,
          message: message
        };

    // if controller is attached, delegate to record the event
    if (me.controller) {
      me.controller.recordEvent(event);
    }
    else {
      // else queue the event and emit to console
      me.eventQueue.push(event);
      AS.Console.recordEvent(event);
    }
  }
});