
/*global Ext, AS*/

/**
 * Privilege model.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.model.Privilege', {
  extend: 'Ext.data.Model',
  fields: [
    {name: 'id', type: 'string'},
    {name: 'version', type: 'string'},
    {name: 'name', type: 'string', sortType: 'asUCText'},
    {name: 'description', type: 'string', sortType: 'asUCText', convert: Ext.htmlEncode},
    {name: 'type', type: 'string', sortType: 'asUCText'},
    {name: 'readOnly', type: 'boolean'},
    {name: 'properties', type: 'auto' /*object*/},
    {name: 'permission', type: 'string', sortType: 'asUCText', convert: Ext.htmlEncode}
  ]
});
