
/*global Ext, NX*/

/**
 * Blobstore model.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.model.Blobstore', {
  extend: 'Ext.data.Model',
  idProperty: 'name',
  fields: [
    {name: 'name', type: 'string', sortType: 'asUCText'},
    {name: 'type', type: 'string', sortType: 'asUCText'},
    {name: 'isQuotaEnabled', type:'boolean' },
    {name: 'quotaType', type:'string' },
    {name: 'quotaLimit', type:'int' },
    {name: 'attributes', type: 'auto' /*object*/, defaultValue: null },
    {name: 'blobCount', type: 'int'},
    {name: 'totalSize', type: 'int'},
    {name: 'availableSpace', type: 'int'},
    {name: 'unlimited', type: 'boolean'},
    {name: 'repositoryUseCount', type: 'int'},
    {name: 'blobStoreUseCount', type: 'int'},
    {name: 'inUse', type: 'boolean'},
    {name: 'promotable', type: 'boolean'},
    {name: 'groupName', type: 'string'},
    {name: 'unavailable', type: 'boolean'},
    {
      name: 'state',
      type: 'string',
      calculate: function(data) {
        if (data.unavailable) {
          return NX.I18n.get('Blobstore_BlobstoreList_Failed');
        }
        else {
          return NX.I18n.get('Blobstore_BlobstoreList_Started');
        }
      }
    }
  ]
});
