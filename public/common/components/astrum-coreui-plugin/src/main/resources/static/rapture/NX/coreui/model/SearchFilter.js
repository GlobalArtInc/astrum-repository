
/*global Ext, NX*/

/**
 * Search filter model.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.model.SearchFilter', {
  extend: 'Ext.data.Model',
  fields: [
    {name: 'id', type: 'string'},
    {name: 'name', type: 'string'},
    {name: 'text', type: 'string'},
    {name: 'description', type: 'string'},
    {name: 'criterias', type: 'auto' /*array*/},
    {name: 'readOnly', type: 'boolean'}
  ]
});
