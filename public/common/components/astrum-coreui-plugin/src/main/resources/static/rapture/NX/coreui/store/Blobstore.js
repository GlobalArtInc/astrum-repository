
/*global Ext, NX*/

/**
 * Blobstore format.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.store.Blobstore', {
  extend: 'Ext.data.Store',
  model: 'NX.coreui.model.Blobstore',

  proxy: {
    type: 'direct',

    api: {
      read: 'NX.direct.coreui_Blobstore.read'
    },

    reader: {
      type: 'json',
      rootProperty: 'data',
      successProperty: 'success'
    }
  },

  sorters: { property: 'name', direction: 'ASC' }

});
