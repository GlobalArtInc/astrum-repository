
/*global Ext, AS*/

/**
 * Search result model.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.model.SearchResult', {
  extend: 'Ext.data.Model',
  idProperty: 'groupingKey',
  fields: [
    'groupingKey',
    'group',
    'name',
    'format'
  ]
});
