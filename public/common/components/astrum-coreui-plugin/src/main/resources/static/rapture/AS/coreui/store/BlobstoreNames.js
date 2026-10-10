
/*global Ext, AS*/

/**
 * Blobstore format.
 *
 * @since 3.24
 */
Ext.define('AS.coreui.store.BlobstoreNames', {
  extend: 'Ext.data.Store',
  model: 'AS.coreui.model.Blobstore',

  proxy: {
    type: 'direct',

    api: {
      read: 'AS.direct.coreui_Blobstore.readNames'
    },

    reader: {
      type: 'json',
      rootProperty: 'data',
      successProperty: 'success'
    }
  },

  sorters: { property: 'name', direction: 'ASC' }

});
