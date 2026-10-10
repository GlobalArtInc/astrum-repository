
/*global Ext, AS*/

/**
 * Generic reference (id/name) model.
 *
 * @since 3.14
 */
Ext.define('AS.coreui.model.Format', {
  extend: 'Ext.data.Model',
  fields: [
    {name: 'value', type: 'string', sortType: 'asUCText'}
  ]
});
