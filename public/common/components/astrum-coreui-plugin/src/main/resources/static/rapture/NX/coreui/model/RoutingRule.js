
/*global Ext, NX*/

/**
 * Routing Rule model.
 *
 * @since 3.16
 */
Ext.define('NX.coreui.model.RoutingRule', {
  extend: 'Ext.data.Model',
  idProperty: 'name',
  fields: [
    {name: 'name', type: 'string', sortType: 'asUCText'},
    {name: 'description', type: 'string', sortType: 'asUCText'},
    {name: 'mode', type: 'string', sortType: 'asUCText'},
    {name: 'matchers', type: 'auto', sortType: 'asUCText'}
  ]
});
