
/*global Ext, NX*/

/**
 * Generic reference (id/name) model.
 *
 * @since 3.14
 */
Ext.define('NX.coreui.model.Format', {
  extend: 'Ext.data.Model',
  fields: [
    {name: 'value', type: 'string', sortType: 'asUCText'}
  ]
});
