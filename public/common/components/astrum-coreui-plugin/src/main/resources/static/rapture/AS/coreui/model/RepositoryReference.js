
/*global Ext, AS*/

/**
 * Repository reference model.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.model.RepositoryReference', {
  extend: 'Ext.data.Model',
  fields: [
    {name: 'id', type: 'string', sortType: 'asUCText'},
    {name: 'name', type: 'string', sortType: 'asUCText'},
    {name: 'type', type: 'string', sortType: 'asUCText'},
    {name: 'format', type: 'string', sortType: 'asUCText'},
    {name: 'versionPolicy', type: 'string', sortType: 'asUCText'},
    {name: 'status', type: 'auto' /*object*/},
    {name: 'url', type: 'string', sortType: 'asUCText'},
    {name: 'sortOrder', sortType: 'asInt'}
  ]
});
