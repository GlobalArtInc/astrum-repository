
/*global Ext, AS*/

/**
 * Capability type model.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.model.CapabilityType', {
  extend: 'Ext.data.Model',
  fields: [
    {name: 'id', type: 'string', sortType: 'asUCText'},
    {name: 'name', type: 'string', sortType: 'asUCText'},
    {name: 'about', type: 'string', sortType: 'asUCText'},
    {name: 'formFields', type: 'auto' /*object*/}
  ]
});
