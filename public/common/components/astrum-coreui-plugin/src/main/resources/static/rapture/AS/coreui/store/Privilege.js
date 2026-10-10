
/*global Ext, AS*/

/**
 * Privilege store.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.store.Privilege', {
  extend: 'Ext.data.Store',
  model: 'AS.coreui.model.Privilege',

  proxy: {
    type: 'direct',

    api: {
      read: 'AS.direct.coreui_Privilege.read'
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
