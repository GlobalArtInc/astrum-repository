
/*global Ext, AS*/

/**
 * Repository model.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.model.Repository', {
  extend: 'Ext.data.Model',
  idProperty: 'name',
  fields: [
    {name: 'name', type: 'string', sortType: 'asUCText'},
    {name: 'type', type: 'string', sortType: 'asUCText'},
    {name: 'format', type: 'string', sortType: 'asUCText'},
    {name: 'size', type: 'int'},
    {name: 'recipe', type: 'string', sortType: 'asUCText'},
    {name: 'online', type: 'boolean'},
    {name: 'status', type: 'auto' /*object*/},
    {name: 'attributes', type: 'auto' /*object*/},
    {
      name: 'blobStoreName',
      type: 'string',
      sortType: 'asUCText',
      calculate: function (data) {
        // Note that attributes object must be defined before to be able to access its properties here.
        return data.attributes.storage.blobStoreName;
      },
    },
    {name: 'url', type: 'string', sortType: 'asUCText'},
  ],
});
