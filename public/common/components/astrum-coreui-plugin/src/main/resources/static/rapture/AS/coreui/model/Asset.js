
/*global Ext, AS*/

/**
 * Asset model.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.model.Asset', {
  extend: 'Ext.data.Model',
  fields: [
    {name: 'id', type: 'string', sortType: 'asUCText'},
    {name: 'name', type: 'string'},
    {name: 'format', type: 'string', sortType: 'asUCText'},
    {name: 'contentType', type: 'string', sortType: 'asUCText'},
    {name: 'size', type: 'int'},
    {name: 'repositoryName', type: 'string', sortType: 'asUCText'},
    {name: 'containingRepositoryName', type: 'string', sortType: 'asUCText'},
    {name: 'blobCreated', type: 'date', dateFormat: 'c' },
    {name: 'blobUpdated', type: 'date', dateFormat: 'c' },
    {name: 'lastDownloaded', type: 'date', dateFormat: 'c' },
    {name: 'downloadCount', type: 'int', allowNull: true, defaultValue: null},
    {name: 'blobRef', type: 'string', sortType: 'asUCText'},
    {name: 'componentId', type: 'string', sortType: 'asUCText'},
    {name: 'createdBy', type: 'string'},
    {name: 'createdByIp', type: 'string', sortType: 'asUCText'},
    {name: 'registryUrl', type: 'string'},
    {name: 'attributes', type: 'auto' /*object*/}
  ]
});
