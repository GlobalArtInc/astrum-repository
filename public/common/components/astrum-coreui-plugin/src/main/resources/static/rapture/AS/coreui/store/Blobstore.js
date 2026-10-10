
/*global Ext, AS*/

/**
 * Blobstore format.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.store.Blobstore', {
  extend: 'Ext.data.Store',
  model: 'AS.coreui.model.Blobstore',

  proxy: {
    type: 'direct',

    api: {
      read: 'AS.direct.coreui_Blobstore.read'
    },

    reader: {
      type: 'json',
      rootProperty: 'data',
      successProperty: 'success'
    }
  },

  sorters: { property: 'name', direction: 'ASC' }

});
