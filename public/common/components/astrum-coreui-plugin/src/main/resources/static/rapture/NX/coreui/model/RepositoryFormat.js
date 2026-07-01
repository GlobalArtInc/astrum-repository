
/*global Ext, NX*/

/**
 * Repository format model.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.model.RepositoryFormat', {
  extend: 'Ext.data.Model',
  fields: [
    {name: 'id', type: 'string', sortType: 'asUCText'},
    {name: 'name', type: 'string', sortType: 'asUCText'}
  ]
});
