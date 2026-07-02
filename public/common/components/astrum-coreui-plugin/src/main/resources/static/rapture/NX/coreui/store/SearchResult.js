
/*global Ext, NX*/

/**
 * Search result store.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.store.SearchResult', {
  extend: 'Ext.data.Store',
  model: 'NX.coreui.model.Component',

  proxy: {
    type: 'direct',

    api: {
      read: 'NX.direct.coreui_Search.read'
    },

    reader: {
      type: 'json',
      rootProperty: 'data',
      successProperty: 'success',
      keepRawData: true
    }
  },

  buffered: true,
  pageSize: 300,

  remoteFilter: true,
  remoteSort: true
});
