
/*global Ext, NX*/

/**
 * Generic reference (id/name) model.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.model.Reference', {
  extend: 'Ext.data.Model',
  fields: [
    {name: 'id', type: 'string', sortType: 'asUCText'},
    {name: 'name', type: 'string', sortType: 'asUCText'}
  ]
});
