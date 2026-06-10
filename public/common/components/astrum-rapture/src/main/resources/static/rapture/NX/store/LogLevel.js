
/*global Ext*/

/**
 * Log-level store.
 *
 * @since 3.0
 */
Ext.define('NX.store.LogLevel', {
  extend: 'Ext.data.Store',
  model: 'NX.model.LogLevel',
  data: [
    { name: 'all' },
    { name: 'trace' },
    { name: 'debug' },
    { name: 'info' },
    { name: 'warn' },
    { name: 'error' },
    { name: 'off' }
  ]
});
