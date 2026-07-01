
/*global Ext, NX*/

/**
 * Component model.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.model.ComponentPreview', {
  extend: 'Ext.data.Model',
  fields: [
    {name: 'id', type: 'string', sortType: 'asUCText'},
    {name: 'group', type: 'string'},
    {name: 'name', type: 'string'},
    {name: 'version', type: 'string'}
  ]
});
