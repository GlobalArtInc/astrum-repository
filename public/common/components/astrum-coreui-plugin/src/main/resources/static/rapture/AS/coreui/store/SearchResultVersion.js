
/*global Ext, AS*/

/**
 * Versions / search result store.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.store.SearchResultVersion', {
  extend: 'Ext.data.Store',
  model: 'AS.coreui.model.SearchResultVersion',

  proxy: {
    type: 'direct',

    api: {
      read: 'AS.direct.coreui_Search.readVersions'
    },

    reader: {
      type: 'json',
      rootProperty: 'data',
      successProperty: 'success'
    }
  },

  remoteFilter: true,
  sorters: { property: 'versionOrder', direction: 'DESC' }

});
