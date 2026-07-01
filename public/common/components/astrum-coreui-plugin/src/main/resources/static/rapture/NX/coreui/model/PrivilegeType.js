
/*global Ext, NX*/

/**
 * Privilege type model.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.model.PrivilegeType', {
  extend: 'Ext.data.Model',
  fields: [
    {name: 'id', type: 'string', sortType: 'asUCText'},
    {name: 'name', type: 'string', sortType: 'asUCText'},
    {name: 'about', type: 'string', sortType: 'asUCText'},
    {name: 'formFields', type: 'auto' /*object*/}
  ]
});
