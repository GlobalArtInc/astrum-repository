
/*global Ext, NX*/

/**
 * Task type model.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.model.TaskType', {
  extend: 'Ext.data.Model',
  fields: [
    {name: 'id', type: 'string', sortType: 'asUCText'},
    {name: 'name', type: 'string', sortType: 'asUCText'},
    {name: 'exposed', type: 'boolean'},
    {name: 'formFields', type: 'auto' /*array*/}
  ]
});
