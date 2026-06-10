
/*global Ext*/

/**
 * Log-event model.
 *
 * @since 3.0
 */
Ext.define('NX.model.LogEvent', {
  extend: 'Ext.data.Model',

  fields: [
    { name: 'timestamp', type: 'int' },
    { name: 'logger', type: 'string' },
    { name: 'level', type: 'string' },
    { name: 'message' }
  ],
  identifier: 'sequential'
});
