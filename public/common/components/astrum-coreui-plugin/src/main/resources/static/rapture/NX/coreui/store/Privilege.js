
/*global Ext, NX*/

/**
 * Privilege store.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.store.Privilege', {
  extend: 'Ext.data.Store',
  model: 'NX.coreui.model.Privilege',

  proxy: {
    type: 'direct',

    api: {
      read: 'NX.direct.coreui_Privilege.read'
    },

    reader: {
      type: 'json',
      rootProperty: 'data',
      successProperty: 'success'
    }
  },

  buffered: true,
  pageSize: 300,
  trailingBufferZone: 20,
  leadingBufferZone: 50,

  remoteFilter: true,
  remoteSort: true,
  
  sorters: { property: 'name', direction: 'ASC' }
});
