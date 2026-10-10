
/*global Ext*/

/**
 * State model.
 *
 * @since 3.0
 */
Ext.define('AS.model.State', {
  extend: 'Ext.data.Model',
  idProperty: 'key',
  fields: [
    { name: 'key', type: 'string' },
    { name: 'value', defaultValue: undefined },
    { name: 'hash', type: 'string' }
  ]
});
