
/*global Ext, AS*/

/**
 * User model.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.model.User', {
  extend: 'Ext.data.Model',
  idProperty: {
    name: 'id',
    convert: function(value, record) {
      return Ext.htmlEncode(record.get('userId'));
    }
  },
  fields: [
    {name: 'userId', type: 'string', sortType: 'asUCText' },
    {name: 'version', type: 'int', sortType: 'asUCText'},
    {name: 'realm', type: 'string', sortType: 'asUCText'},
    {name: 'firstName', type: 'string', sortType: 'asUCText' },
    {name: 'lastName', type: 'string', sortType: 'asUCText' },
    {name: 'email', type: 'string', sortType: 'asUCText'},
    {name: 'status', type: 'string', sortType: 'asUCText'},
    {name: 'roles', type: 'auto' /*array*/},
    {name: 'external', type: 'boolean'},
    {name: 'externalRoles', type: 'auto' /*array*/}
  ]
});
