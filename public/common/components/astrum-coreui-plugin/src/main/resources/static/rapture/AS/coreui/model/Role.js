
/*global Ext, AS*/

/**
 * Role model.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.model.Role', {
  extend: 'Ext.data.Model',
  fields: [
    {name: 'id', type: 'string', sortType: 'asUCText'},
    {name: 'version', type: 'string', sortType: 'asUCText'},
    {name: 'source', type: 'string', sortType: 'asUCText'},
    {name: 'name', type: 'string', sortType: 'asUCText', convert: Ext.htmlEncode},
    {name: 'description', type: 'string', sortType: 'asUCText', convert: Ext.htmlEncode},
    {name: 'readOnly', type: 'boolean', defaultValue: false},
    {name: 'privileges', type: 'auto'},
    {name: 'roles', type: 'auto'}
  ],
  constructor: function(data, id, raw, convertedData) {
    this.callParent([data, Ext.htmlEncode(id), raw, convertedData]);
  }
});
