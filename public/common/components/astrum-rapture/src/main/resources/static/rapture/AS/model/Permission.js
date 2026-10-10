
/*global Ext, AS*/

/**
 * Permission model.
 *
 * @since 3.0
 */
Ext.define('AS.model.Permission', {
  extend: 'Ext.data.Model',
  requires: [
    'AS.Permissions'
  ],

  fields: [
    { name: 'id', type: 'string' },
    { name: 'permitted', type: 'bool', defaultValue: true }
  ]

});
