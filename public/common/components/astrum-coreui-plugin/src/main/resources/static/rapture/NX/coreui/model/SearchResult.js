
/*global Ext, NX*/

/**
 * Search result model.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.model.SearchResult', {
  extend: 'Ext.data.Model',
  idProperty: 'groupingKey',
  fields: [
    'groupingKey',
    'group',
    'name',
    'format'
  ]
});
