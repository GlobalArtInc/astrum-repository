
/*global Ext*/

/**
 * Log-level model.
 *
 * @since 3.0
 */
Ext.define('NX.model.LogLevel', {
  extend: 'Ext.data.Model',
  idProperty: 'name',
  fields: [
    { name: 'name', type: 'string' }
  ]
});
