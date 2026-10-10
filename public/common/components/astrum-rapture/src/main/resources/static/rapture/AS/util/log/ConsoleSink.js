
/*global Ext, AS*/

/**
 * Console {@link AS.util.log.Sink}.
 *
 * Emits events to the browser console.
 *
 * @since 3.0
 */
Ext.define('AS.util.log.ConsoleSink', {
  extend: 'AS.util.log.Sink',
  requires: [
    'AS.Console'
  ],

  // default to disabled
  enabled: false,

  /**
   * @override
   */
  receive: function (event) {
    AS.Console.recordEvent(event);
  }
});
