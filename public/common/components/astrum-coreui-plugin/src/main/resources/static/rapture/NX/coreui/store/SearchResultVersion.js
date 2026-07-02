
/*global Ext, NX*/

/**
 * Versions / search result store.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.store.SearchResultVersion', {
  extend: 'Ext.data.Store',
  model: 'NX.coreui.model.SearchResultVersion',

  proxy: {
    type: 'direct',

    api: {
      read: 'NX.direct.coreui_Search.readVersions'
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
