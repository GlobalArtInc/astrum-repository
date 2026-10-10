
/*global Ext, AS*/

/**
 * Search result store.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.store.SearchResult', {
  extend: 'Ext.data.Store',
  model: 'AS.coreui.model.Component',

  proxy: {
    type: 'direct',

    api: {
      read: 'AS.direct.coreui_Search.read'
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
