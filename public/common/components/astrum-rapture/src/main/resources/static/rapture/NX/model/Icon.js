
/*global Ext*/

/**
 * Icon model.
 *
 * @since 3.0
 */
Ext.define('NX.model.Icon', {
  extend: 'Ext.data.Model',

  idProperty: 'cls',
  fields: [
    { name: 'cls', type: 'string' },
    { name: 'name', type: 'string' },
    { name: 'file', type: 'string' },
    { name: 'variant', type: 'string' },
    { name: 'height', type: 'int' },
    { name: 'width', type: 'int' },
    { name: 'url', type: 'string' },
    { name: 'preload', type: 'boolean' }
  ]
});
