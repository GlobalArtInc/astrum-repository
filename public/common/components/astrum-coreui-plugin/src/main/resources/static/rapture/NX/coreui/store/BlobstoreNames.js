
/*global Ext, NX*/

/**
 * Blobstore format.
 *
 * @since 3.24
 */
Ext.define('NX.coreui.store.BlobstoreNames', {
  extend: 'Ext.data.Store',
  model: 'NX.coreui.model.Blobstore',

  proxy: {
    type: 'direct',

    api: {
      read: 'NX.direct.coreui_Blobstore.readNames'
    },

    reader: {
      type: 'json',
      rootProperty: 'data',
      successProperty: 'success'
    }
  },

  sorters: { property: 'name', direction: 'ASC' }

});
