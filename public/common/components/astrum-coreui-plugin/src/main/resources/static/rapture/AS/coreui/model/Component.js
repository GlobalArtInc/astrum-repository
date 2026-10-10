
/*global Ext, AS*/

/**
 * Component model.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.model.Component', {
  extend: 'Ext.data.Model',
  idProperty: 'groupingKey',
  fields: [
    {name: 'groupingKey', type: 'string', sortType: 'asUCText'},
    {name: 'id', type: 'string', sortType: 'asUCText'},
    {name: 'repositoryName', type: 'string', sortType: 'asUCText'},
    {name: 'group', type: 'string'},
    {name: 'name', type: 'string'},
    {name: 'version', type: 'string'},
    {name: 'format', type: 'string', sortType: 'asUCText'},
    {name: 'attributes', type: 'auto' /*object*/}
  ]
});
