
/*global Ext, AS*/

/**
 * Cleanup Policy model.
 *
 * @since 3.14
 */
Ext.define('AS.coreui.model.CleanupPolicy', {
  extend: 'Ext.data.Model',
  idProperty: 'name',
  fields: [
    {name: 'name', type: 'string', sortType: 'asUCText'},
    {name: 'format', type: 'string', sortType: 'asUCText'},
    {name: 'notes', type: 'string', sortType: 'asUCText'},
    {name: 'mode', type: 'string', sortType: 'asUCText'},
    {name: 'sortOrder', sortType: 'asInt'}
  ]
});
