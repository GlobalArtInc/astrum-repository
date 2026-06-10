
/*global Ext, NX*/

/**
 * Permission model.
 *
 * @since 3.0
 */
Ext.define('NX.model.Permission', {
  extend: 'Ext.data.Model',
  requires: [
    'NX.Permissions'
  ],

  fields: [
    { name: 'id', type: 'string' },
    { name: 'permitted', type: 'bool', defaultValue: true }
  ]

});
