
/*global Ext, AS*/

/**
 * Capability model.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.model.Capability', {
  extend: 'Ext.data.Model',
  fields: [
    {name: 'id', type: 'string', sortType: 'asUCText'},
    {name: 'typeId', type: 'string', sortType: 'asUCText'},
    {name: 'enabled', type: 'boolean'},
    {name: 'notes', type: 'string', sortType: 'asUCText'},
    {name: 'properties', type: 'auto' /*object*/, defaultValue: null },

    {name: 'active', type: 'boolean'},
    {name: 'error', type: 'boolean'},
    {name: 'description', type: 'string', sortType: 'asUCText'},
    {name: 'state', type: 'string', sortType: 'asUCText'},
    {name: 'stateDescription', type: 'string', sortType: 'asUCText'},
    {name: 'status', type: 'string', sortType: 'asUCText'},
    {name: 'typeName', type: 'string', sortType: 'asUCText'},
    {name: 'tags', type: 'auto' /*object*/}
  ]
});
