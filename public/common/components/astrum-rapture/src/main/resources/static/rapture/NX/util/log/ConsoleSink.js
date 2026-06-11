
/*global Ext, NX*/

/**
 * Console {@link NX.util.log.Sink}.
 *
 * Emits events to the browser console.
 *
 * @since 3.0
 */
Ext.define('NX.util.log.ConsoleSink', {
  extend: 'NX.util.log.Sink',
  requires: [
    'NX.Console'
  ],

  // default to disabled
  enabled: false,

  /**
   * @override
   */
  receive: function (event) {
    NX.Console.recordEvent(event);
  }
});
