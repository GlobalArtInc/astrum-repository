
/*global Ext, NX*/

/**
 * @since 3.7
 */
Ext.define('NX.coreui.model.UploadComponentDefinition', {
  extend: 'Ext.data.Model',
  idProperty: 'format',
  fields: [
    {name: 'multipleUpload', type: 'boolean', sortType: 'asUCText'},
    {name: 'format', type: 'string', sortType: 'asUCText'},
    {name: 'assetFields', type: 'auto' /*object*/},
    {name: 'componentFields', type: 'auto' /*object*/},
    {name: 'regexMap', type: 'auto' /*object*/}
  ]
});
